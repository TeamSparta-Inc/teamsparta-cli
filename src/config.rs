use serde::Deserialize;
use serde_json::Result;
use std::{collections::HashMap, fs};

#[derive(Deserialize)]
pub struct Config {
    pub mongo_dump: MongoDumpServiceConfig,
    pub watch: WatchServiceConfig,
}
#[derive(Deserialize)]
pub struct MongoDumpServiceConfig {
    pub online: MongoDumpInstruction,
    pub swc: MongoDumpInstruction,
    pub hhv2: MongoDumpInstruction,
    pub nbc: MongoDumpInstruction,
    pub scc: MongoDumpInstruction,
    pub chang: MongoDumpInstruction,
    pub intellipick: MongoDumpInstruction,
    pub backoffice: MongoDumpInstruction,
    pub backoffice_bootcamp: MongoDumpInstruction,
}
#[derive(Deserialize)]
pub struct MongoDumpInstruction {
    pub uri: String,
    pub excludes: Vec<String>,
    pub target_port: u32,
}
#[derive(Deserialize)]
pub struct WatchServiceConfig {
    pub slack: SlackInstruction,
    pub pipeline: Codepipeline,
}
#[derive(Deserialize)]
pub struct Codepipeline {
    pub online: String,
    pub online_test: String,
    pub swc: String,
    pub swc_test: String,
    pub hhv2: String,
    pub hhv2_test: String,
    pub nbc: String,
    pub nbc_test: String,
    pub intellipick: String,
    pub intellipick_test: String,
    pub h99: String,
    pub h99_test: String,
}

#[derive(Deserialize)]
pub struct SlackInstruction {
    pub webhook_url: String,
    pub user_id: String,
    pub known_users: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Result<Config> {
        let config_file =
            fs::read_to_string("./src/config.json").expect("config file did not found");
        let config: Config = serde_json::from_str(&config_file)
            .expect("config.json file properties lacks for this program");

        Ok(config)
    }
}
