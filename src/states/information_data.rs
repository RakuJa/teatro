use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{env, fs};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InformationEntry {
    pub position: usize,
    pub data: String,
}

impl InformationEntry {
    pub fn write_to_file(path: &str, data: &Vec<Self>) -> anyhow::Result<()> {
        let serialized = serde_json::to_string_pretty(data)?;
        let mut file = fs::File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> anyhow::Result<Vec<Self>> {
        let contents = fs::read_to_string(path)?;
        let data: Vec<Self> = serde_json::from_str(&contents).expect("Deserialization failed");
        Ok(data)
    }
}

fn default_path() -> String {
    env::var("DATA_PATH").unwrap_or_else(|_| "data.json".to_string())
}
