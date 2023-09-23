use std::fmt::{Display, Error, Formatter};

use crate::{jwt::JwtSignError, config::LoadConfigError, gitolite::GitoliteError};

pub enum CommandError {
    WrongNumberOfParameters(usize),
    InvalidOperation(String),
    LoadEnvError(std::env::VarError),
    LoadConfigError(LoadConfigError),
    JwtSigningError(JwtSignError),
    UnauthorizedError(GitoliteError),
    LoggerError,
}

/* -------------------------------------------------------------------------- */
/*                 End user errors: no sensitive informations                 */
/* -------------------------------------------------------------------------- */

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let message = match self {
            CommandError::InvalidOperation(operation) => {
                format!("Invalid operation: expected 'download' or 'upload', got {}", operation)
            }
            CommandError::WrongNumberOfParameters(n) => format!("Wrong number of parameters, expected 2 or 3, got {}\nUsage: git-lfs-authenticate <repo> <operation> [oid]", n),
            CommandError::LoadConfigError(_) => "Server error".to_string(),
            CommandError::LoadEnvError(_) => "Server error".to_string(),
            CommandError::JwtSigningError(_) => "Server error".to_string(),
            CommandError::UnauthorizedError(_) => "Unauthorized".to_string(),
            CommandError::LoggerError => "Server error".to_string(),
        };
        write!(f, "{}", message)
    }
}

/* -------------------------------------------------------------------------- */
/*                           Logs: full informations                          */
/* -------------------------------------------------------------------------- */

impl Display for LoadConfigError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            LoadConfigError::CurrentExeNotFound(e) => {
                write!(f, "Could not find current exe location: {}", e)
            }
            LoadConfigError::EnvFileNotFound(e) => write!(f, "Error while opening configuration file: {}", e),
            LoadConfigError::PathDecoding => write!(f, "Could not decode path to configuration file"),
            LoadConfigError::InvalidLineInEnvFile(line) => {
                write!(f, "Invalid line in configuration file: {}", line)
            }
            LoadConfigError::MissingKey(key) => write!(f, "Missing key in configuration file: {}", key),
        }
    }
}

impl Display for JwtSignError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            JwtSignError::SystemTime => write!(f, "Could not get current time"),
            JwtSignError::SecretDecoding => write!(f, "Could not decode jwt secret"),
            JwtSignError::JwtSigning => write!(f, "Could not sign jwt"),
        }
    }
}

impl Display for GitoliteError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            GitoliteError::ExecutionError(e) => write!(f, "Error while executing gitolite: {}", e),
            GitoliteError::UnauthorizedError(e) => write!(f, "Unauthorized access prevented: {}", e),
        }
    }
}

impl CommandError {
    pub fn log(&self) -> String {
        match self {
            CommandError::LoadConfigError(e) => format!("LoadConfigError: {}", e),
            CommandError::LoadEnvError(e) => format!("LoadEnvError: {}", e),
            CommandError::JwtSigningError(e) => format!("JwtSigningError: {}", e),
            CommandError::UnauthorizedError(e) => format!("UnauthorizedError: {}", e),
            _ => format!("{}", self),
        }
    }
}
