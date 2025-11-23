use futures::future::join_all;
use log::{debug, trace};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: u64,
    pub learning_task_name: String,
    pub subject_name: String,
    pub class_id: u64,
    pub subject_id: u64,
    pub total_score: f64,
    pub is_in_subject_score: bool,
    pub score: Option<f64>,
    pub class_avg_score: f64,
    pub class_max_score: f64,
    pub eva_projects: Vec<EvaulationProject>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvaulationProject {
    pub name: String,
    pub e_name: String,
    pub id: u32,
    pub parent_pro_id: u32,
    pub proportion: f64,
}

pub async fn get_semester_tasks(client: &reqwest::Client, semester_id: u64) -> Vec<Task> {
    let task_ids = get_task_ids(client, semester_id).await;
    let mut futures = Vec::new();
    for task_id in task_ids {
        let task_handle = get_task_details(client, task_id);
        futures.push(task_handle);
    }
    let task_details = join_all(futures).await;
    trace!(
        "Task list for semester_id {}:\n{:#?}",
        semester_id,
        task_details
    );
    task_details
}

pub async fn get_task_ids(client: &reqwest::Client, semester_id: u64) -> Vec<u64> {
    debug!("Sending task list request for semester_id: {}", semester_id);
    // pageSize=500 is the max amount
    let response: serde_json::Value = client
        .get(format!("https://tsinglanstudent.schoolis.cn/api/LearningTask/GetList?semesterId={semester_id}&pageIndex=1&pageSize=500"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    debug!(
        "Received task list response for semester_id: {}",
        semester_id
    );
    trace!("{:#}", &response);
    let tasks = response["data"]["list"]
        .as_array()
        .expect("Failed to get tasks");
    let ids: Vec<u64> = tasks
        .iter()
        .filter_map(|task| task["id"].as_u64())
        .collect();
    trace!("task_ids: {:?}", &ids);
    ids
}
pub async fn get_task_details(client: &reqwest::Client, task_id: u64) -> Task {
    debug!("Sending task request for task_id: {}", task_id);
    let response: serde_json::Value = client
        .get(format!(
            "https://tsinglanstudent.schoolis.cn/api/LearningTask/GetDetail?learningTaskId={task_id}"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    debug!("Received task response for task_id: {}", task_id);
    trace!("Task {} response:\n{:#}", task_id, &response);
    let task: Task = serde_json::from_value(response["data"].clone()).unwrap();
    trace!("Parsed task {}:\n{:#?}", task_id, &task);
    task
}
