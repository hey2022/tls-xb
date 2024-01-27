use serde::Deserialize;

pub async fn get_subject_ids(client: &reqwest::Client, semester_id: u64) -> Vec<u64> {
    let response: serde_json::Value = client
        .get(format!("https://tsinglanstudent.schoolis.cn/api/LearningTask/GetStuSubjectListForSelect?semesterId={}", semester_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let subjects = response["data"]
        .as_array()
        .expect("Failed to get subjects");
    let ids: Vec<u64> = subjects
        .iter()
        .filter_map(|subject| subject["id"].as_u64())
        .collect();
    ids
}

pub struct Subject {
    pub subject_name: String,
    subject_id: u64,
    class_id: u64,
    pub total_score: f64,
    pub evaluation_projects: Vec<EvaluationProject>,
}

pub async fn get_subject(client: &reqwest::Client, semester_id: u64, subject_id: u64) -> Subject {
    let subject_detail = get_subject_detail(&client, semester_id, subject_id).await;
    let evaluation_projects =
        get_subject_evaluation_projects(&client, semester_id, &subject_detail).await;
    let subject = Subject {
        subject_name: subject_detail.subject_name,
        subject_id: subject_detail.subject_id,
        class_id: subject_detail.class_id,
        total_score: get_subject_score(&evaluation_projects).await,
        evaluation_projects,
    };
    subject
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubjectDetail {
    subject_name: String,
    class_id: u64,
    subject_id: u64,
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
    score_is_null: bool,
    pub proportion: f64,
    pub score: f64,
}

async fn get_subject_evaluation_projects(
    client: &reqwest::Client,
    semester_id: u64,
    subject_detail: &SubjectDetail,
) -> Vec<EvaluationProject> {
    let response: serde_json::Value = client
        .get(format!("https://tsinglanstudent.schoolis.cn/api/DynamicScore/GetDynamicScoreDetail?classId={}&subjectId={}&semesterId={}",
                     subject_detail.class_id, subject_detail.subject_id, semester_id))
        .send()
        .await.unwrap()
        .json()
        .await.unwrap();
    let evaluation_projects: Vec<EvaluationProject> =
        serde_json::from_value(response["data"]["evaluationProjectList"].clone())
            .expect("Failed to get evaluation projects");
    evaluation_projects
}

async fn get_subject_score(evauluation_projects: &[EvaluationProject]) -> f64 {
    let mut total_score = 0.0;
    let mut total_proportion = 0.0;
    for evaluation_project in evauluation_projects {
        total_score += evaluation_project.proportion * evaluation_project.score;
        if !evaluation_project.score_is_null {
            total_proportion += evaluation_project.proportion;
        }
    }
    let subject_score = total_score / total_proportion;
    subject_score
}
