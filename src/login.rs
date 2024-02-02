use super::config::Config;
use serde::Serialize;
use std::time::SystemTime;

pub async fn login(config: &Config) -> reqwest::Client {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let payload = get_payload(config.name.clone(), config.password.clone()).await;
    client
        .post("https://tsinglanstudent.schoolis.cn/api/MemberShip/Login")
        .json(&payload)
        .send()
        .await
        .unwrap();
    client
}

#[derive(Serialize)]
struct Payload {
    name: String,
    password: String,
    timestamp: u64,
}

async fn get_payload(name: String, password: String) -> Payload {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let payload = Payload {
        name,
        password: get_hashed_password(password, timestamp).await,
        timestamp,
    };
    payload
}

async fn get_hashed_password(password: String, timestamp: u64) -> String {
    let timestamp = timestamp.to_string();
    let hash = format!("{:x}", md5::compute(password)).to_uppercase();
    let combined = hash + &timestamp;
    let combined_hash = format!("{:x}", md5::compute(combined)).to_uppercase();
    combined_hash
}
