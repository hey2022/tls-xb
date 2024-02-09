use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;
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
    io::stdout().flush().expect("Unable to flush stdout");
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
    let hash = format!("{:x}", md5::compute(password)).to_uppercase();
    let combined = hash + &timestamp;
    let combined_hash = format!("{:x}", md5::compute(combined)).to_uppercase();
    combined_hash
}

pub fn get_config() -> Config {
    println!(
        "Getting config.toml from {}...",
        confy::get_configuration_file_path("tls-xb", "config")
            .unwrap()
            .to_str()
            .unwrap()
    );
    let config: Config = confy::load("tls-xb", "config").expect("Failed to get config");
    config
}
