use crate::calendar::date_parser;
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use chrono::Utc;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Semester {
    pub id: u64,
    pub year: u64,
    pub semester: u64,
    pub is_now: bool,
    #[serde(deserialize_with = "date_parser")]
    pub start_date: DateTime<FixedOffset>,
    #[serde(deserialize_with = "date_parser")]
    pub end_date: DateTime<FixedOffset>,
}

pub async fn get_semesters(client: &reqwest::Client) -> Vec<Semester> {
    let response: serde_json::Value = client
        .get("https://tsinglanstudent.schoolis.cn/api/School/GetSchoolSemesters")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    serde_json::from_value(response["data"].clone()).expect("Failed to get semesters")
}

pub fn get_current_semester(semesters: &[Semester]) -> Option<&Semester> {
    semesters.iter().find(|s| s.is_now)
}
