use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub services: HashMap<String, Service>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub upload_url: String,
    pub download_url: String,
}

impl Config {
    pub fn new() -> Result<Config, Box<dyn Error>> {
        let mut services = HashMap::new();

        // Iterate over all environment variables
        for (key, value) in env::vars() {
            // Look for service environment variables with the pattern SERVICE_<NAME>_UPLOAD_URL and SERVICE_<NAME>_DOWNLOAD_URL
            if key.starts_with("SERVICE_") {
                let parts: Vec<&str> = key.split('_').collect();
                if parts.len() >= 3 {
                    let service_name = parts[1]; // Extract the service name from the variable
                    let service_type = parts[2]; // "UPLOAD" or "DOWNLOAD"

                    // Use the service name as a key to store the service info
                    let service = services
                        .entry(service_name.to_string().to_ascii_lowercase())
                        .or_insert(Service {
                            name: service_name.to_string().to_ascii_lowercase(),
                            upload_url: String::new(),
                            download_url: String::new(),
                        });

                    // Assign the corresponding URL based on the type
                    if service_type == "UPLOAD" {
                        service.upload_url = value;
                    } else if service_type == "DOWNLOAD" {
                        service.download_url = value;
                    }
                }
            }
        }

        let config = Config { services };
        info!("{:?}", config);
        Ok(config)
    }

    pub fn get_download_url(&self, service_name: &str) -> Option<&str> {
        self.services
            .get(service_name)
            .map(|service| service.download_url.as_str())
    }

    pub fn get_upload_url(&self, service_name: &str) -> Option<&str> {
        self.services
            .get(service_name)
            .map(|service| service.upload_url.as_str())
    }
}
