use crate::{calendar::Calendar, gpa::*, round_score, task::Task};
use chrono::Duration;
use itertools::Itertools;
use log::{debug, trace};
use serde::Deserialize;
use std::collections::HashMap;

pub async fn get_subject_ids(client: &reqwest::Client, semester_id: u64) -> Vec<u64> {
    debug!(
        "Sending request for subject_ids for semester_id: {}",
        semester_id
    );
    let response: serde_json::Value = client
        .get(format!("https://tsinglanstudent.schoolis.cn/api/LearningTask/GetStuSubjectListForSelect?semesterId={semester_id}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    debug!(
        "Received subject_ids response for semester_id: {}",
        semester_id
    );
    let subjects = response["data"].as_array().expect("Failed to get subjects");
    let ids: Vec<u64> = subjects
        .iter()
        .filter_map(|subject| subject["id"].as_u64())
        .unique()
        .collect();
    trace!("subject_ids: {:?}", &ids);
    ids
}
pub struct Subject {
    pub subject_name: String,
    pub subject_id: u64,
    pub class_id: u64,
    pub total_score: f64,
    pub extra_credit: f64,
    pub in_gpa: bool, // Unreliable method of determining whether subject counts into gap
    // pub evaluation_projects: Vec<EvaluationProject>,
    // pub score_mapping_list_id: ScoreMappingId,
    // pub score_mapping_list: Vec<ScoreMappingConfig>,
    pub gpa: f64,
    pub max_gpa: f64,
    pub unweighted_gpa: f64,
    pub unweighted_max_gpa: f64,
    pub score_level: String,
    pub elective: bool,
    pub weight: f64,
}

pub struct EvaluationProject {
    pub evaluation_project_e_name: String,
    pub proportion: f64,
    pub score: f64,
    pub score_level: String,
    pub gpa: f64,
    pub score_is_null: bool,
    pub tasks: Vec<Task>,
    pub adjusted_proportion: f64,
}

impl Subject {
    pub fn new(subject_id: u64, tasks: &[Task]) -> Self {
        let subject_tasks: Vec<Task> = tasks
            .iter()
            .filter(|task| task.subject_id == subject_id)
            .cloned()
            .collect();
        trace!("Subject {} tasks:\n{:#?}", subject_id, subject_tasks);
        // Assume the subject details from the first task is the same as the other tasks
        let first_task = subject_tasks.iter().next().unwrap();
        let subject_name = first_task.subject_name.clone();
        let subject_id = first_task.subject_id.clone();
        let class_id = first_task.class_id.clone();
        dbg!(&subject_name);
        dbg!(&subject_id);
        dbg!(&class_id);
        let evaluation_projects = Self::calculate_evaluation_projects(&subject_tasks);
        todo!()
    }

    fn calculate_evaluation_projects(subject_tasks: &[Task]) -> Vec<EvaluationProject> {
        dbg!(&subject_tasks);
        todo!()
    }
}

impl EvaluationProject {
    pub fn new(evaluation_project_id: u32, tasks: &[Task]) {
        todo!()
        // let evaluation_project_tasks = tasks.iter().filter(|task| task.eva_projects);
    }
}
