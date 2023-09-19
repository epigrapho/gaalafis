use std::process::Command;

pub enum GitoliteError {
    ExecutionError(std::io::Error),
    UnauthorizedError(String),
}

pub fn check_access(repo: &str, user: &str, access: &str) -> Result<(), GitoliteError> {
    let output = Command::new("gitolite")
        .arg("access")
        .arg("-q")
        .arg(repo)
        .arg(user)
        .arg(access)
        .output()
        .map_err(GitoliteError::ExecutionError)?;

    if !output.status.success() {
        return Err(GitoliteError::UnauthorizedError(
            String::from_utf8(output.stdout).unwrap_or("Unknown".to_string()),
        ));
    }

    Ok(())
}
