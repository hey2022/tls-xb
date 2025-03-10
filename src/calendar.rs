use chrono::{DateTime, FixedOffset, Utc};
use icalendar::{Calendar as ical, Component, Event, EventLike};
use log::debug;
use serde::de::{Deserializer, Error};
use serde::Deserialize;

pub fn date_parser<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let time = s.trim_start_matches("/Date(").trim_end_matches(")/");
    // Bug in chrono preventing parsing "%s%3f%z"
    let (millis, offset) = time.split_at(time.len() - 5);
    let millis = millis.parse::<f64>().unwrap() / 1000.0;
    DateTime::parse_from_str(format!("{millis}{offset}").as_str(), "%s%.3f%z")
        .map_err(Error::custom)
}

#[derive(Deserialize)]
pub struct Calendar {
    pub blocks: Vec<Block>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: u64,
    #[serde(rename = "eName")]
    pub class_name: String,
    #[serde(deserialize_with = "date_parser")]
    pub begin_time: DateTime<FixedOffset>,
    #[serde(deserialize_with = "date_parser")]
    pub end_time: DateTime<FixedOffset>,
}

impl Calendar {
    pub async fn new(
        client: &reqwest::Client,
        begin_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Calendar {
        let begin_time_payload = begin_time.format("%Y-%m-%d").to_string();
        let end_time_payload = end_time.format("%Y-%m-%d").to_string();
        debug!("Calendar range: {begin_time_payload} - {end_time_payload}");
        let payload =
            &serde_json::json!({"beginTime":begin_time_payload,"endTime":end_time_payload});
        let response: serde_json::Value = client
            .post("https://tsinglanstudent.schoolis.cn/api/Schedule/ListScheduleByParent")
            .json(&payload)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        Calendar {
            blocks: serde_json::from_value(response["data"].clone())
                .expect("Failed to parse calendar"),
        }
    }

    pub fn export_ical(&self) -> icalendar::Calendar {
        let mut ical = ical::new();
        ical.name("Tsinglan Class Calendar");
        for block in &self.blocks {
            let event = Event::new()
                .summary(&block.class_name)
                .starts(block.begin_time.with_timezone(&Utc))
                .ends(block.end_time.with_timezone(&Utc))
                .done();
            ical.push(event);
        }
        ical.done()
    }
}
