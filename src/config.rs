use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Config {
    pub name: String,
    pub password: String,
}

pub async fn get_config() -> Config {
    println!(
        "Getting config.toml from {}...",
        confy::get_configuration_file_path("tls", "config")
            .unwrap()
            .to_str()
            .unwrap()
    );
    let config: Config = confy::load("tls-xb", "config").unwrap();
    config
}
