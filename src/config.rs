use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;
use std::time::SystemTime;

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub name: String,
    pub password: String,
    pub timestamp: u64,
}

pub fn login() -> Config {
    print!("Username: ");
    io::stdout().flush().expect("Unable to flush stdout");
    let mut name = String::new();
    std::io::stdin()
        .read_line(&mut name)
        .expect("Failed to read line");
    name = name.trim().to_string();
    let password = rpassword::prompt_password("Password: ").unwrap();
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let hashed_password = get_hashed_password(password, timestamp);
    Config {
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

pub fn save_config(config: &Config) {
    confy::store("tls-xb", "config", config).expect("Failed to get config");
}

pub fn get_config() -> Config {
    let config: Config = confy::load("tls-xb", "config").expect("Failed to get config");
    config
}
