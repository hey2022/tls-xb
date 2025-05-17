use crate::{config::Login, prompt_input};
use base64::Engine as _;
use serde::Serialize;

#[derive(Serialize)]
struct Payload {
    name: String,
    password: String,
    timestamp: u64,
}

pub enum LoginError {
    IncorrectCaptcha(String),
    IncorrectLogin(String),
    ErrorCode((String, i32)),
}

pub async fn login(config: &Login) -> Result<reqwest::Client, LoginError> {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let payload = Payload {
        name: config.name.clone(),
        password: config.password.clone(),
        timestamp: config.timestamp,
    };
    let captcha = get_captcha(&client).await;
    let response: serde_json::Value = client
        .post(format!(
            "https://tsinglanstudent.schoolis.cn/api/MemberShip/Login?captcha={captcha}",
        ))
        .json(&payload)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let state = serde_json::from_value(response["state"].clone()).unwrap();
    match state {
        0 => Ok(client),
        1180038 => Err(LoginError::IncorrectCaptcha(response["msg"].to_string())),
        13 | 1010076 => Err(LoginError::IncorrectLogin(response["msg"].to_string())),
        _ => Err(LoginError::ErrorCode((response["msg"].to_string(), state))),
    }
}

pub async fn get_captcha(client: &reqwest::Client) -> String {
    let reponse: serde_json::Value = client
        .get("https://tsinglanstudent.schoolis.cn/api/MemberShip/GetStudentCaptchaForLogin")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let encoded_captcha: String = serde_json::from_value(reponse["data"].clone()).unwrap();
    if encoded_captcha.is_empty() {
        return encoded_captcha;
    }

    let decoded_captcha = base64::engine::general_purpose::STANDARD
        .decode(encoded_captcha.trim_start_matches("data:image/png;base64,"))
        .expect("Failed to decode base64 data");
    let image = image::load_from_memory(&decoded_captcha).expect("Failed to load image");
    let conf = viuer::Config::default();
    print!("\x1B[2J"); // clear terminal screen
    viuer::print(&image, &conf).expect("Failed to print image");

    prompt_input!("\nCaptacha: ")
}
