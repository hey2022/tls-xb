use log::info;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::prompt_input;

#[derive(Deserialize, Serialize, Default)]
pub struct Login {
    pub name: String,
    pub password: String,
    pub timestamp: u64,
}

pub fn login() -> Login {
    let name = prompt_input!("Username: ");
    let password = rpassword::prompt_password("Password: ").unwrap();
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let hashed_password = get_hashed_password(password, timestamp);
    Login {
        name,
        password: hashed_password,
        timestamp,
    }
}

fn get_hashed_password(password: String, timestamp: u64) -> String {
    let timestamp = timestamp.to_string();
    let hash = format!("{:X}", md5::compute(password));
    let combined = hash + &timestamp;
    let combined_hash = format!("{:X}", md5::compute(combined));
    combined_hash
}

pub fn save_login(config: &Login) {
    confy::store("tls-xb", "login", config).expect("Failed to save login");
}

pub fn get_login() -> Login {
    info!(
        "Getting login.toml from {}",
        confy::get_configuration_file_path("tls-xb", "login")
            .unwrap()
            .to_str()
            .unwrap()
    );
    confy::load("tls-xb", "login").expect("Failed to get login")
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Config {
    pub colors: ColorScheme,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ColorScheme {
    pub a_color: String,
    pub b_color: String,
    pub c_color: String,
    pub d_color: String,
    pub f_color: String,
    pub text_color: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme {
            a_color: "green".to_string(),
            b_color: "blue".to_string(),
            c_color: "yellow".to_string(),
            d_color: "red".to_string(),
            f_color: "red".to_string(),
            text_color: "white".to_string(),
        }
    }
}

pub fn get_config() -> Config {
    info!(
        "Getting config.toml from {}",
        confy::get_configuration_file_path("tls-xb", "config")
            .unwrap()
            .to_str()
            .unwrap()
    );
    confy::load("tls-xb", "config").expect("Failed to get config")
}

pub fn save_config(config: &Config) {
    confy::store("tls-xb", "config", config).expect("Failed to save config");
}
