use crate::gpa::*;
use chrono::{DateTime, Duration, FixedOffset};
use itertools::Itertools;
use serde::Deserialize;
use std::{collections::HashMap, f64::NAN};

pub async fn get_subject_ids(client: &reqwest::Client, semester_id: u64) -> Vec<u64> {
    let response: serde_json::Value = client
        .get(format!("https://tsinglanstudent.schoolis.cn/api/LearningTask/GetStuSubjectListForSelect?semesterId={semester_id}"))
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
    pub score_mapping_list: Vec<ScoreMappingConfig>,
    pub gpa: f64,
    pub max_gpa: f64,
    pub unweighted_gpa: f64,
    pub score_level: String,
    pub elective: bool,
    pub weight: f64,
}

pub async fn get_subject(
    client: &reqwest::Client,
    semester_id: u64,
    subject_id: u64,
    score_mapping_lists: &HashMap<ScoreMappingId, Vec<ScoreMappingConfig>>,
) -> Subject {
    let subject_detail = get_subject_detail(client, semester_id, subject_id).await;
    let evaluation_projects = get_subject_evaluation_projects(client, &subject_detail).await;
    let total_score = get_subject_score(&evaluation_projects);
    let score_mapping_list_id = get_score_mapping_list_id(&subject_detail);
    let score_mapping_list = score_mapping_lists[&score_mapping_list_id].clone();
    let gpa = gpa_from_score(total_score, &score_mapping_list);
    let max_gpa = gpa_from_score(100.0, &score_mapping_list);
    let unweighted_gpa = gpa_from_score(
        total_score,
        &score_mapping_lists[&ScoreMappingId::NonWeighted],
    );
    let score_level = score_level_from_score(total_score, &score_mapping_list);
    Subject {
        subject_name: subject_detail.subject_name,
        subject_id,
        class_id: subject_detail.class_id,
        total_score,
        evaluation_projects,
        score_mapping_list_id,
        score_mapping_list,
        gpa,
        max_gpa,
        unweighted_gpa,
        score_level,
        elective: false,
        weight: 1.0,
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
        .get(format!("https://tsinglanstudent.schoolis.cn/api/LearningTask/GetList?semesterId={semester_id}&subjectId={subject_id}&pageIndex=1&pageSize=1"))
        .send()
        .await.unwrap()
        .json()
        .await.unwrap();
    let task_id = response["data"]["list"][0]["id"]
        .as_u64()
        .expect("Failed to get task id");
    let response: serde_json::Value = client
        .get(format!(
            "https://tsinglanstudent.schoolis.cn/api/LearningTask/GetDetail?learningTaskId={task_id}",
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
    pub learning_task_and_exam_list: Vec<LearningTask>,
    #[serde(default)]
    pub evaluation_project_list: Vec<EvaluationProject>,
    #[serde(skip)]
    pub adjusted_proportion: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningTask {
    pub name: String,
    pub score: Option<f64>,
    pub total_score: f64,
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
    let mut evaluation_projects: Vec<EvaluationProject> =
        serde_json::from_value(response["data"]["evaluationProjectList"].clone())
            .expect("Failed to get evaluation projects");
    let total_proportion: f64 = evaluation_projects
        .iter()
        .filter(|evaluation_project| !evaluation_project.score_is_null)
        .map(|evaluation_project| evaluation_project.proportion)
        .sum();
    for evaluation_project in &mut evaluation_projects {
        evaluation_project.adjusted_proportion =
            evaluation_project.proportion / total_proportion * 100.0;
        let total_proportion: f64 = evaluation_project
            .evaluation_project_list
            .iter()
            .filter(|evaluation_project| !evaluation_project.score_is_null)
            .map(|evaluation_project| evaluation_project.proportion)
            .sum();
        for sub_evaluation_project in &mut evaluation_project.evaluation_project_list {
            sub_evaluation_project.adjusted_proportion = sub_evaluation_project.proportion
                / total_proportion
                * evaluation_project.adjusted_proportion;
        }
    }
    evaluation_projects
}

fn get_subject_score(evaluation_projects: &[EvaluationProject]) -> f64 {
    evaluation_projects
        .iter()
        .filter(|evaluation_project| !evaluation_project.score_is_null)
        .map(|evaluation_project| {
            evaluation_project.score * evaluation_project.adjusted_proportion / 100.0
        })
        .reduce(|a, b| a + b)
        .unwrap_or(NAN)
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

pub fn adjust_weights(subject: &mut Subject, elective_class_ids: &[u64]) {
    let elective = elective_class_ids.contains(&subject.class_id);
    if elective || subject.subject_name == "C-Humanities" {
        subject.elective = elective;
        subject.weight = 0.5;
    }
}
