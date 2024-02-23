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

    let state = serde_json::from_value(response["state"].clone()).unwrap();
    if state != 0 {
        println!("{}", response["msg"]);
        match state {
            1180038 => panic!("Captcha failed"),
            1010076 => panic!("Invalid username or password, try running 'tls-xb login'"),
            _ => panic!("Unknown error state: {}", state),
        }
    }
    client
}
