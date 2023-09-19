use std::collections::HashMap;

pub enum LoadConfigError {
    CurrentExeNotFound(std::io::Error),
    EnvFileNotFound(std::io::Error),
    PathDecoding,
    InvalidLineInEnvFile(String),
    MissingKey(String),
}

#[derive(Debug)]
pub struct Config {
    base_url: String,
    pub jwt_secret: String,
    pub expires_in: u64,
}

impl Config {
    fn get_config_file_location() -> Result<Option<String>, LoadConfigError> {
        let mut path = std::env::current_exe().map_err(LoadConfigError::CurrentExeNotFound)?;
        path.pop();
        path.push(".env");
        Ok(path.to_str().map(|s| s.to_string()))
    }

    fn parse_env_file(content: String) -> Result<HashMap<String, String>, LoadConfigError> {
        let mut config_hash: HashMap<String, String> = HashMap::new();
        for line in content.lines() {
            let mut line_split = line.split('=');
            let key = match line_split.next() {
                Some(key) => key.to_string(),
                None => return Err(LoadConfigError::InvalidLineInEnvFile(line.to_string())),
            };
            let value = match line_split.next() {
                Some(value) => value.to_string(),
                None => return Err(LoadConfigError::InvalidLineInEnvFile(line.to_string())),
            };
            config_hash.insert(key, value);
        }
        Ok(config_hash)
    }

    fn get_or_error(map: &HashMap<String, String>, key: &str) -> Result<String, LoadConfigError> {
        match map.get(key) {
            Some(value) => Ok(value.to_string()),
            None => Err(LoadConfigError::MissingKey(key.to_string())),
        }
    }

    fn get_or_default_u64(map: &HashMap<String, String>, key: &str, default: u64) -> u64 {
        match map.get(key) {
            Some(value) => value.parse::<u64>().unwrap_or(default),
            None => default,
        }
    }

    pub fn load_config_file() -> Result<Config, LoadConfigError> {
        let path = Self::get_config_file_location()?;
        let path = match path {
            Some(path) => path,
            None => return Err(LoadConfigError::PathDecoding),
        };
        let config_content =
            std::fs::read_to_string(path).map_err(LoadConfigError::EnvFileNotFound)?;
        let config_map = Self::parse_env_file(config_content)?;

        let base_url = Self::get_or_error(&config_map, "BASE_URL")?;
        let jwt_secret_file = Self::get_or_error(&config_map, "JWT_SECRET_FILE")?;
        let jwt_secret = std::fs::read_to_string(jwt_secret_file)
            .map_err(LoadConfigError::EnvFileNotFound)?
            .trim()
            .to_string();
        let expires_in = Self::get_or_default_u64(&config_map, "EXPIRES_IN", 30 * 60);

        Ok(Config {
            base_url,
            jwt_secret,
            expires_in,
        })
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}
