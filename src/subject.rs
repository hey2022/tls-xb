use crate::gpa::*;
use chrono::{DateTime, Duration, FixedOffset};
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;

pub async fn get_subject_ids(client: &reqwest::Client, semester_id: u64) -> Vec<u64> {
    let response: serde_json::Value = client
        .get(format!("https://tsinglanstudent.schoolis.cn/api/LearningTask/GetStuSubjectListForSelect?semesterId={}", semester_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let subjects = response["data"].as_array().expect("Failed to get subjects");
    let ids: Vec<u64> = subjects
        .iter()
        .filter_map(|subject| subject["id"].as_u64())
        .unique()
        .collect();
    ids
}

pub struct Subject {
    pub subject_name: String,
    pub subject_id: u64,
    pub class_id: u64,
    pub total_score: f64,
    pub evaluation_projects: Vec<EvaluationProject>,
    pub score_mapping_list_id: ScoreMappingId,
    pub gpa: f64,
    pub score_level: String,
    pub elective: bool,
    pub weight: f64,
}

pub async fn get_subject(
    client: &reqwest::Client,
    semester_id: u64,
    subject_id: u64,
    score_mapping_lists: &HashMap<ScoreMappingId, Vec<ScoreMappingConfig>>,
    elective_class_ids: &[u64],
) -> Subject {
    let subject_detail = get_subject_detail(client, semester_id, subject_id).await;
    let evaluation_projects = get_subject_evaluation_projects(client, &subject_detail).await;
    let total_score = get_subject_score(&evaluation_projects).await;
    let score_mapping_list_id = get_score_mapping_list_id(&subject_detail);
    let score_mapping_list = &score_mapping_lists[&score_mapping_list_id];
    let gpa = gpa_from_score(total_score, score_mapping_list);
    let score_level = score_level_from_score(total_score, score_mapping_list);
    let elective = elective_class_ids.contains(&subject_detail.class_id);
    let weight = if elective { 0.5 } else { 1.0 };
    Subject {
        subject_name: subject_detail.subject_name,
        subject_id,
        class_id: subject_detail.class_id,
        total_score,
        evaluation_projects,
        score_mapping_list_id,
        gpa,
        score_level,
        elective,
        weight,
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectDetail {
    pub subject_name: String,
    class_id: u64,
    subject_id: u64,
    school_semester_id: u64,
}

async fn get_subject_detail(
    client: &reqwest::Client,
    semester_id: u64,
    subject_id: u64,
) -> SubjectDetail {
    let response: serde_json::Value = client
        .get(format!("https://tsinglanstudent.schoolis.cn/api/LearningTask/GetList?semesterId={}&subjectId={}&pageIndex=1&pageSize=1", semester_id, subject_id))
        .send()
        .await.unwrap()
        .json()
        .await.unwrap();
    let task_id = response["data"]["list"][0]["id"]
        .as_u64()
        .expect("Failed to get task id");
    let response: serde_json::Value = client
        .get(format!(
            "https://tsinglanstudent.schoolis.cn/api/LearningTask/GetDetail?learningTaskId={}",
            task_id
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let subject_detail: SubjectDetail =
        serde_json::from_value(response["data"].clone()).expect("Failed to get subject detail");
    subject_detail
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationProject {
    pub evaluation_project_e_name: String,
    pub proportion: f64,
    pub score: f64,
    pub score_level: String,
    pub gpa: f64,
    pub score_is_null: bool,
    #[serde(default)]
    pub evaluation_project_list: Vec<EvaluationProject>,
}

async fn get_subject_evaluation_projects(
    client: &reqwest::Client,
    subject_detail: &SubjectDetail,
) -> Vec<EvaluationProject> {
    let response: serde_json::Value = client
        .get(format!("https://tsinglanstudent.schoolis.cn/api/DynamicScore/GetDynamicScoreDetail?classId={}&subjectId={}&semesterId={}",
                     subject_detail.class_id, subject_detail.subject_id, subject_detail.school_semester_id))
        .send()
        .await.unwrap()
        .json()
        .await.unwrap();
    let evaluation_projects: Vec<EvaluationProject> =
        serde_json::from_value(response["data"]["evaluationProjectList"].clone())
            .expect("Failed to get evaluation projects");
    evaluation_projects
}

async fn get_subject_score(evaluation_projects: &[EvaluationProject]) -> f64 {
    let mut total_score = 0.0;
    let mut total_proportion = 0.0;
    for evaluation_project in evaluation_projects {
        total_score += evaluation_project.proportion * evaluation_project.score;
        if !evaluation_project.score_is_null {
            total_proportion += evaluation_project.proportion;
        }
    }

    total_score / total_proportion
}

pub async fn get_elective_class_ids(
    client: &reqwest::Client,
    begin_time: DateTime<FixedOffset>,
) -> Vec<u64> {
    let begin_time_payload = begin_time.format("%Y-%m-%d").to_string();
    // 8 days = 6 days per cycle + 2 weekends
    let end_time_payload = (begin_time + Duration::days(8))
        .format("%Y-%m-%d")
        .to_string();
    let payload = &serde_json::json!({"beginTime":begin_time_payload,"endTime":end_time_payload});
    let response: serde_json::Value = client
        .post("https://tsinglanstudent.schoolis.cn/api/Schedule/ListScheduleByParent")
        .json(&payload)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let blocks = response["data"].as_array().unwrap();
    let elective_class_ids = blocks
        .iter()
        .filter(|block| block["classInfo"]["classEName"].to_string().contains("Ele"))
        .filter_map(|block| block["classInfo"]["id"].as_u64())
        .unique()
        .collect();
    elective_class_ids
}
