use serde::{Deserialize, Serialize};

use std::{env, fs::File};

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub port: u16,
}

#[allow(dead_code)]
impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let cfg = match (
            File::open("app.yml"),
            File::open("/etc/chat/app.yml"),
            env::var("CHAT_CONFIG"),
        ) {
            (Ok(f), _, _) => serde_yml::from_reader(f),
            (_, Ok(f), _) => serde_yml::from_reader(f),
            (_, _, Ok(f)) => serde_yml::from_reader(File::open(f)?),
            _ => anyhow::bail!("Could not find app.yml"),
        };

        anyhow::Ok(cfg?)
    }
}
