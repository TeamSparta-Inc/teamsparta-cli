use serde::Deserialize;
use serde_json::Result;
use std::{collections::HashMap, fs};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub mongo_dump: HashMap<String, MongoDumpInstruction>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MongoDumpInstruction {
    pub source_uri: String,
    pub target_uri: String,
    pub db_name: String,
    pub excludes: Vec<String>,
    pub family: HashMap<String, Vec<String>>,
}

impl Config {
    pub fn new() -> Result<Config> {
        let config_file =
            fs::read_to_string("/etc/sprt/config.json").expect("config file did not found");
        let config: Config = serde_json::from_str(&config_file)
            .expect("config.json file properties lacks for this program");

        Ok(config)
    }
}
