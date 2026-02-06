use std::{collections::HashMap, fs::File, io::BufReader};

use affinidi_tdk::secrets_resolver::secrets::Secret;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub mediator_did: String,
    pub our_did: String,
    pub did_secrets: Vec<Secret>,
    pub passwords: HashMap<String, String>,
}

impl Config {
    /// Attempts to load a configuration from `config_path`
    pub fn load_config(config_path: &str) -> Result<Self> {
        let file = File::open(config_path)?;
        let reader = BufReader::new(file);

        Ok(serde_json::from_reader(reader)?)
    }

    /// Attempts to save to a local file at `config_path`
    pub fn save(&self, config_path: &str) -> Result<()> {
        let file = File::create(config_path)?;
        serde_json::to_writer_pretty(file, &self)?;

        Ok(())
    }
}
