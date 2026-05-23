use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GlobalConfig {
    pub database_connection_string: String,
}

impl GlobalConfig {
    fn load_config(file_path: String) -> GlobalConfig {
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
