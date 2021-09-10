use serde::{Deserialize, Serialize};
use serde_json::{from_reader, Value};

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct SettingsJson {
    pub tags: Vec<String>,
    pub wip: bool,
    pub number: Option<u64>,
}

impl SettingsJson {
    pub fn from_file(filename: &PathBuf) -> SettingsJson {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        return from_reader(reader).unwrap();
    }

    pub fn from_json(json: &Value) -> SettingsJson {
        return SettingsJson {
            wip: json["wip"].as_bool().unwrap(),
            number: json["number"].as_u64(),
            tags: json["tags"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect::<Vec<String>>(),
        };
    }
}
