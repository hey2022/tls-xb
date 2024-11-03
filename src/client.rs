use crate::config::Config;
use base64::Engine as _;
use image::{DynamicImage, ImageFormat};
use serde::Serialize;
use std::io::Cursor;
use text_io::read;

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
    if state != 0 {
        println!("{}", response["msg"]);
        match state {
            1180038 => panic!("Captcha failed"),
            1010076 => panic!("Invalid username or password, try running 'tls-xb login'"),
            _ => panic!("Unknown error state: {state}"),
        }
    }
    client
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
    let image =
        image::load(Cursor::new(decoded_captcha), ImageFormat::Png).expect("Failed to load image");
    let image = DynamicImage::ImageRgba8(image.to_rgba8());
    let conf = viuer::Config::default();
    print!("\x1B[2J"); // clear terminal screen
    viuer::print(&image, &conf).expect("Failed to print image");

    print!("\nCaptcha: ");
    let captcha = read!();
    captcha
}
