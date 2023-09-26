use crate::jwt::JwtPayload;
use command_errors::CommandError;
use config::Config;
use gitolite::check_access;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use response::AuthResponse;

mod command_errors;
mod config;
mod gitolite;
mod jwt;
mod response;

fn expect_between_3_and_4_parameters(args: &[String]) -> Result<(), CommandError> {
    if args.len() < 3 || args.len() > 4 {
        return Err(CommandError::WrongNumberOfParameters(args.len()));
    }
    Ok(())
}

fn get_gitolite_operation(operation: &str) -> Result<String, CommandError> {
    match operation {
        "download" => Ok("R".to_string()),
        "upload" => Ok("W".to_string()),
        _ => Err(CommandError::InvalidOperation(operation.to_string())),
    }
}

fn _main() -> Result<String, CommandError> {
    let args: Vec<String> = std::env::args().collect();
    log::info!("Running git-lfs-authenticate with arguments: {:?}", args);
    expect_between_3_and_4_parameters(&args)?;

    let config = Config::load_config_file().map_err(CommandError::LoadConfigError)?;

    let repo = args[1].trim().to_string();
    let jwt_operation = &args[2]; // download or upload
    let gitolite_operation = get_gitolite_operation(&args[2])?; // R or W
    let user = std::env::var("GL_USER").map_err(CommandError::LoadEnvError)?;

    check_access(&repo, &user, &gitolite_operation).map_err(CommandError::UnauthorizedError)?;

    let token = JwtPayload::new(&repo, &user, jwt_operation)
        .sign(&config.jwt_secret, &config.expires_in)
        .map_err(CommandError::JwtSigningError)?;

    let response = AuthResponse::new(config.url(&repo), token, config.expires_in);
    log::info!("Successfully ran git-lfs-authenticate with arguments: {:?}", args);
    Ok(response.to_string())
}

fn init_log_file() -> Result<(), CommandError> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log/output.log")
        .map_err(|_| CommandError::LoggerError)?;

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .map_err(|_| CommandError::LoggerError)?;

    log4rs::init_config(config).map_err(|_| CommandError::LoggerError)?;
    Ok(())
}

fn main() {
    // Error loading the logger are quite critical, but we do not want to leak any info to user
    // So we just print it as a "Server error" and leave. We have no backup way of logging it.
    if let Err(e) = init_log_file() {
        eprintln!("{e}");
        std::process::exit(1);
    }

    match _main() {
        Ok(response) => {
            println!("{response}");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{e}");
            log::error!("{}", e.log());
            std::process::exit(1);
        }
    }
}
