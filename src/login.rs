use serde::{Deserialize, Serialize};
use std::fs;
use std::time::SystemTime;

pub async fn login() -> reqwest::Client {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let payload = get_payload().await;
    client
        .post("https://tsinglanstudent.schoolis.cn/api/MemberShip/Login")
        .json(&payload)
        .send()
        .await
        .unwrap();
    client
}

#[derive(Deserialize)]
struct Config {
    name: String,
    password: String,
}

#[derive(Serialize)]
struct Payload {
    name: String,
    password: String,
    timestamp: u64,
}

async fn get_payload() -> Payload {
    let config_str = fs::read_to_string("./config.toml").expect("config.toml should be located at the project root");
    let config: Config = toml::from_str(&config_str).expect("config.toml should contain name and password key/value pairs");
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let payload = Payload {
        name: config.name,
        password: get_hashed_password(config.password, timestamp).await,
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
