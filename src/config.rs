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
    confy::store("tls-xb", "login", config).expect("Failed to get login");
}

pub fn get_login() -> Login {
    info!(
        "Getting login.toml from {}",
        confy::get_configuration_file_path("tls-xb", "login")
            .unwrap()
            .to_str()
            .unwrap()
    );
    confy::load("tls-xb", "login").expect("Failed to get config")
}
