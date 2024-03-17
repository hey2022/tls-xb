use chrono::{DateTime, FixedOffset};
use serde::de::{Deserializer, Error};
use serde::Deserialize;

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

fn date_parser<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let time = s.trim_start_matches("/Date(").trim_end_matches(")/");
    // Bug in chrono preventing parsing "%s%3f%z"
    let (millis, offset) = time.split_at(time.len() - 5);
    let millis = millis.parse::<f64>().unwrap() / 1000.0;
    DateTime::parse_from_str(format!("{}{}", millis, offset).as_str(), "%s%.3f%z")
        .map_err(Error::custom)
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
