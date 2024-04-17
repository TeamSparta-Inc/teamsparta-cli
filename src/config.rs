use serde::Deserialize;
use serde_json::Result;
use std::{
    collections::HashMap,
    fs::{self, create_dir_all, OpenOptions},
    io::{Read, Write},
};
const CONFIG_SUFFIX: &str = "sprt/config.json";
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
        let home_dir = dirs::home_dir().expect("failed to get home dir");
        let config_path = home_dir.join(CONFIG_SUFFIX);
        let mut config_content = String::new();

        if !config_path.exists() {
            let parent = config_path.parent().expect("failed to get parent dir path");

            create_dir_all(parent).expect("failed to create dir");
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(&config_path)
                .expect("failed to create new config");

            writeln!(file, r#"{{"mongo_dump":{{}}}}"#).unwrap()
        }

        OpenOptions::new()
            .read(true)
            .open(&config_path)
            .expect("could not open config file")
            .read_to_string(&mut config_content)
            .expect("failed to read as string config file");

        fs::read_to_string(&config_path).expect("config file did not found");
        let config: Config = serde_json::from_str(&config_content)
            .expect("config.json file properties lacks for this program");

        Ok(config)
    }
}
