pub async fn get_gpa(client: &reqwest::Client, semester_id: u64) -> f64 {
    let response: serde_json::Value = client
        .get(format!(
            "https://tsinglanstudent.schoolis.cn/api/DynamicScore/GetGpa?semesterId={}",
            semester_id
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let gpa = response["data"].as_f64().unwrap_or(f64::NAN);
    gpa
}
