use serde::Deserialize;

#[derive(Deserialize)]
pub struct Semester {
    pub id: u64,
    pub year: u64,
    pub semester: u64,
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
    let semesters =
        serde_json::from_value(response["data"].clone()).expect("Failed to get semesters");
    semesters
}
