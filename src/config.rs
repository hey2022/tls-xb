use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use text_io::read;

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub name: String,
    pub password: String,
    pub timestamp: u64,
}

pub fn login() {
    print!("Username: ");
    let name = read!();
    let password = rpassword::prompt_password("Password: ").unwrap();
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let hashed_password = get_hashed_password(password, timestamp);
    let config = Config {
        name,
        password: hashed_password,
        timestamp,
    };
    confy::store("tls-xb", "config", config).unwrap();
}

fn get_hashed_password(password: String, timestamp: u64) -> String {
    let timestamp = timestamp.to_string();
    let hash = format!("{:X}", md5::compute(password));
    let combined = hash + &timestamp;
    let combined_hash = format!("{:X}", md5::compute(combined));
    combined_hash
}

pub fn get_config() -> Config {
    let config: Config = confy::load("tls-xb", "config").expect("Failed to get config");
    config
}
