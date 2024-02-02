use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub name: String,
    pub password: String,
}

pub async fn get_config() -> Config {
    let config_str = fs::read_to_string("./config.toml")
        .expect("config.toml should be located at the project root");
    let config: Config = toml::from_str(&config_str)
        .expect("config.toml should contain name and password key/value pairs");
    config
}
