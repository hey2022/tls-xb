use crate::config::Config;
use serde::Serialize;

#[derive(Serialize)]
struct Payload {
    name: String,
    password: String,
    timestamp: u64,
}

pub async fn login(config: &Config) -> reqwest::Client {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let payload = Payload {
        name: config.name.clone(),
        password: config.password.clone(),
        timestamp: config.timestamp,
    };
    let response: serde_json::Value = client
        .post("https://tsinglanstudent.schoolis.cn/api/MemberShip/Login")
        .json(&payload)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    if response["msg"] != serde_json::value::Value::Null {
        println!("Failed to login, run `tls-xb login` to regenerate keys.");
        std::process::exit(1);
    }
    client
}
