use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use std::{env, time};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub services: HashMap<String, Service>,
    pub db_path: String,
    pub data_path: String,
    pub max_age: Duration,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub upload_url: String,
    pub download_url: String,
    pub runs_per_user: u16,
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
                    let service_vars = parts[2..].join("_"); // Join the rest for type

                    // Use the service name as a key to store the service info
                    let service = services
                        .entry(service_name.to_string().to_ascii_lowercase())
                        .or_insert(Service {
                            name: service_name.to_string().to_ascii_lowercase(),
                            upload_url: String::new(),
                            download_url: String::new(),
                            runs_per_user: u16::MIN,
                        });

                    // Assign the corresponding vars to the config
                    match service_vars.as_str() {
                        "UPLOAD_URL" => service.upload_url = value,
                        "DOWNLOAD_URL" => service.download_url = value,
                        "RUNS_PER_USER" => {
                            service.runs_per_user = value.parse::<u16>().unwrap_or(5)
                        }
                        _ => continue, // Skip if it's not a recognized type
                    };
                }
            }
        }

        let wd = env::current_dir().unwrap().display().to_string();

        let db_path = match env::var("DB_PATH") {
            Ok(p) => p,
            Err(_) => {
                let db_path = format!("{}/db.sqlite", wd.clone());
                warn!("DB_PATH not defined, using {:?}", db_path);
                db_path
            }
        };

        let data_path = match env::var("DATA_PATH") {
            Ok(p) => p,
            Err(_) => {
                let data_path = format!("{}/data", wd);
                warn!("DATA_PATH not defined, using {:?}", data_path);
                data_path
            }
        };

        let max_age = match env::var("MAX_AGE") {
            Ok(v) => {
                let time: u64 = v.parse().unwrap();
                time::Duration::from_secs(time)
            }
            Err(_) => {
                let duration = time::Duration::from_secs(864000);
                warn!("MAX_AGE not defined, using {:?}", duration);
                duration
            }
        };

        let config = Config {
            services,
            db_path,
            data_path,
            max_age,
        };
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
