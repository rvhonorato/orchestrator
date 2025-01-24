use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub services: Vec<Service>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    name: String,
    upload_url: String,
    download_url: String,
}

impl Config {
    pub fn new(input_file: &str) -> Result<Config, Box<dyn Error>> {
        let contents = fs::read_to_string(input_file)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
    pub fn get_download_url(&self, service_name: &str) -> Option<&str> {
        self.services
            .iter()
            .find(|service| service.name == service_name)
            .map(|service| service.download_url.as_str())
    }
    pub fn get_upload_url(&self, service_name: &str) -> Option<&str> {
        self.services
            .iter()
            .find(|service| service.name == service_name)
            .map(|service| service.upload_url.as_str())
    }
}
