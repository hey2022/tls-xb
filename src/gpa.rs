use crate::subject::{Subject, SubjectDetail};
use core::fmt;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum ScoreMappingId {
    Weighted,
    NonWeighted,
}

impl fmt::Display for ScoreMappingId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ScoreMappingId::Weighted => write!(f, "Weighted"),
            ScoreMappingId::NonWeighted => write!(f, "Non-Weighted"),
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScoreMappingConfig {
    display_name: String,
    min_value: f64,
    max_value: f64,
    gpa: f64,
}

pub fn default_score_mapping_lists() -> HashMap<ScoreMappingId, Vec<ScoreMappingConfig>> {
    let mut score_mapping_list = HashMap::new();
    let score_mapping_configs: serde_json::Value =
        serde_json::from_str(include_str!("score_mapping_configs.json")).unwrap();
    let weighted_mapping_list: Vec<ScoreMappingConfig> =
        serde_json::from_value(score_mapping_configs["weighted"].clone()).unwrap();
    let non_weighted_mapping_list: Vec<ScoreMappingConfig> =
        serde_json::from_value(score_mapping_configs["non-weighted"].clone()).unwrap();
    score_mapping_list.insert(ScoreMappingId::NonWeighted, non_weighted_mapping_list);
    score_mapping_list.insert(ScoreMappingId::Weighted, weighted_mapping_list);
    score_mapping_list
}

pub fn get_score_mapping_list_id(subject_detail: &SubjectDetail) -> ScoreMappingId {
    if subject_detail.subject_name.contains("AP")
        || subject_detail.subject_name.contains("A Level")
        || subject_detail.subject_name.contains("AS")
    {
        return ScoreMappingId::Weighted;
    }
    ScoreMappingId::NonWeighted
}

pub fn gpa_from_score(total_score: f64, score_mapping_list: &[ScoreMappingConfig]) -> f64 {
    let total_score = (total_score * 10.0).round() / 10.0;
    for config in score_mapping_list.iter() {
        if config.min_value <= total_score && config.max_value >= total_score {
            return config.gpa;
        }
    }
    f64::NAN
}

pub fn score_level_from_score(
    total_score: f64,
    score_mapping_list: &[ScoreMappingConfig],
) -> String {
    let total_score = (total_score * 10.0).round() / 10.0;
    for config in score_mapping_list.iter() {
        if config.min_value <= total_score && config.max_value >= total_score {
            return config.display_name.clone();
        }
    }
    String::new()
}

pub struct CalculatedGPA {
    pub weighted_gpa: f64,
    pub unweighted_gpa: f64,
    pub gpa_delta: f64,
}

pub fn calculate_gpa(subjects: &[Subject]) -> CalculatedGPA {
    let weighted_gpa = calculate_weighted_gpa(subjects);
    let unweighted_gpa = calculate_unweighted_gpa(subjects);
    let gpa_delta = gpa_delta(subjects);
    CalculatedGPA {
        weighted_gpa,
        unweighted_gpa,
        gpa_delta,
    }
}

pub fn gpa_delta(subjects: &[Subject]) -> f64 {
    let total_weight = subjects
        .iter()
        .filter(|subject| !subject.gpa.is_nan())
        .fold(0.0, |total_weight, subject| total_weight + subject.weight);
    let subject_gpa_delta = 0.3; // Currently is 0.3 for all ScoreMappingConfig when not changing from F
    subject_gpa_delta / total_weight
}

pub fn calculate_weighted_gpa(subjects: &[Subject]) -> f64 {
    let total_gpa: f64 = subjects
        .iter()
        .filter(|subject| !subject.gpa.is_nan())
        .fold(0.0, |total_gpa, subject| {
            total_gpa + subject.gpa * subject.weight
        });
    let total_weight = subjects
        .iter()
        .filter(|subject| !subject.gpa.is_nan())
        .fold(0.0, |total_weight, subject| total_weight + subject.weight);
    total_gpa / total_weight
}

pub fn calculate_unweighted_gpa(subjects: &[Subject]) -> f64 {
    let total_unweighted_gpa: f64 = subjects
        .iter()
        .filter(|subject| !subject.unweighted_gpa.is_nan())
        .fold(0.0, |total_gpa, subject| {
            total_gpa + subject.unweighted_gpa * subject.weight
        });
    let total_unweighted_weight = subjects
        .iter()
        .filter(|subject| !subject.unweighted_gpa.is_nan())
        .fold(0.0, |total_weight, subject| total_weight + subject.weight);
    total_unweighted_gpa / total_unweighted_weight
}

pub async fn get_gpa(client: &reqwest::Client, semester_id: u64) -> f64 {
    let response: serde_json::Value = client
        .get(format!(
            "https://tsinglanstudent.schoolis.cn/api/DynamicScore/GetGpa?semesterId={semester_id}",
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    response["data"].as_f64().unwrap_or(f64::NAN)
}
