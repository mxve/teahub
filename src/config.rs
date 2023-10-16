use serde_derive::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct CGitea {
    pub token: String,
    pub user: String,
    pub url: String,
    pub keep_private: bool,
}

#[derive(Deserialize, Debug)]
pub struct CGitHub {
    pub token: String,
    pub user: String,
    pub include_starred: bool,
    pub include_private: bool,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub gitea: CGitea,
    pub github: CGitHub,
}

pub fn load_config(path: PathBuf) -> Config {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => panic!("Error reading file: {:?}", error),
    };

    match toml::from_str(&content) {
        Ok(config) => config,
        Err(error) => panic!("Error parsing TOML: {:?}", error),
    }
}
