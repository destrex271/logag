use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

use crate::storage::traits::StorageBackend;

pub const CONFIG_PATH: &str = "config/";

#[derive(Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub storage_backend: StorageBackend,
    pub database_connection_string: String,
}

impl GlobalConfig {
    pub fn load_config(file_path: String) -> GlobalConfig {
        let path = Path::new(file_path.as_str());
        let display = path.display();
        let content: String = match File::open(&path) {
            Err(err) => panic!("unable to open file {}: {}", display, err),
            Ok(mut file) => {
                let mut s = String::new();
                match file.read_to_string(&mut s) {
                    Err(err) => panic!("unable to read from config {}: {}", display, err),
                    Ok(_) => s.clone(),
                }
            }
        };

        match toml::from_str(content.as_str()) {
            Ok(config) => config,
            Err(err) => panic!("unable to parse config {}: {}", display, err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::StorageBackend;

    #[test]
    fn test_global_config_serde_roundtrip() {
        let config = GlobalConfig {
            storage_backend: StorageBackend::Postgres,
            database_connection_string: "postgres://localhost:5432/test".into(),
        };
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: GlobalConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized.database_connection_string,
            config.database_connection_string
        );
        assert_eq!(deserialized.storage_backend, config.storage_backend);
    }
}
