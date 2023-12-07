use serde::Deserialize;
use serde_json::Result;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub mongo_dump: MongoDumpServiceConfig,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct MongoDumpInstruction {
    pub uri: String,
    pub excludes: Vec<String>,
    pub target_port: u32,
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
