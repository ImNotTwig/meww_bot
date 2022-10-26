use serde_derive::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub prefix: String,
    pub word_blacklist: Vec<String>,
    pub tokens: Tokens,
    pub spam_settings: SpamSettings,
    pub level_system: LevelSystem,
}

#[derive(Debug, Deserialize)]
pub struct Tokens {
    pub discord_token: String,
    pub genius_token: String,
    pub spotify_id: String,
    pub spotify_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct SpamSettings {
    pub antispam: bool,
    pub spam_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct LevelSystem {
    pub levels_on: bool,
    pub xp_per_message: Vec<i32>,
    pub cooldown_in_seconds: i32,
}

pub fn read_config() -> std::io::Result<Config> {
    let content = std::fs::read_to_string("./config.toml");

    let config = match content {
        Ok(file) => file,
        Err(_) => std::fs::read_to_string("./example_config.toml")
            .expect("No config provided, read the readme to see how to set up a config.toml"),
    };

    Ok(toml::from_str(&config)?)
}
