use lfs_info_server::services::{
    custom_link_signer::CustomLinkSignerConfig, fs::local_file_storage::LocalFileStorageConfig,
    jwt_token_encoder_decoder::JwtTokenEncoderDecoderConfig,
    minio::single_bucket_storage::MinioSingleBucketStorageConfig,
    postgres::postgres_locks_provider::PostgresLocksProviderConfig,
};
use s3::creds::Credentials;
use s3::Region;

#[derive(Default)]
pub enum FileStorageImplementation {
    #[default]
    MinioSingleBucketStorage,
    LocalFileStorage,
}

#[derive(Default)]
pub enum LocksImplementation {
    #[default]
    None,
    PostgresLocksProvider,
}

#[derive(Default)]
pub struct ServerConfig {
    pub with_locks: bool,
    pub with_proxy: bool,

    // Choice of implementation
    pub file_storage_implementation: FileStorageImplementation,
    pub locks_implementation: LocksImplementation,

    // LocalFileStorageConfig
    pub fs_root_path: Option<String>,

    // Minio config
    pub sbs_bucket_name: Option<String>,
    pub sbs_access_key: Option<String>,
    pub sbs_secret_key: Option<String>,
    pub sbs_region: Option<String>,
    pub sbs_host: Option<String>,
    pub sbs_public_region: Option<String>,
    pub sbs_public_host: Option<String>,

    // Jwt
    pub jwt_secret: Option<String>,
    pub jwt_expires_in: Option<u64>,

    // Custom signer
    pub custom_signer_host: Option<String>,
    pub custom_signer_secret: Option<String>,
    pub custom_signer_expires_in: Option<u64>,
}

impl ServerConfig {
    /**
     * Parse the CLI arguments and create a config object from it and from the environment variables.
     *
     * The CLI expect 0, 2 or 4 arguments:
     *   - If no arguments is provided, "proxy fs" is assumed
     *   - If 2 arguments are provided, it is expected to be "<signer|proxy> <fs|sbs>"
     *   - If 4 arguments are provided, it is expected to be "<signer|proxy> <fs|sbs> locks pg"
     */
    pub fn from_args(args: Vec<String>) -> Self {
        ServerConfig::default()
            .cli_parse_proxy(&args)
            .cli_parse_fs_impl(&args)
            .cli_parse_locks_impl(&args)
    }

    pub fn get_local_file_storage_config(&self) -> LocalFileStorageConfig {
        let fs_root_path = Self::unwrap_env_var("FS_ROOT_PATH");
        LocalFileStorageConfig {
            root_path: fs_root_path,
        }
    }

    pub fn get_jwt_token_encoder_decoder_config(&self) -> JwtTokenEncoderDecoderConfig {
        let jwt_secret = Self::unwrap_env_file("JWT_SECRET_FILE");
        let jwt_expires_in = Self::unwrap_u64_env_var("JWT_EXPIRES_IN");
        JwtTokenEncoderDecoderConfig {
            secret: jwt_secret,
            expires_in: jwt_expires_in,
        }
    }

    pub fn get_custom_signer_encoder_decoder_config(&self) -> JwtTokenEncoderDecoderConfig {
        let custom_signer_secret = Self::unwrap_env_file("CUSTOM_SIGNER_SECRET_FILE");
        let custom_signer_expires_in = Self::unwrap_u64_env_var("CUSTOM_SIGNER_EXPIRES_IN");
        JwtTokenEncoderDecoderConfig {
            secret: custom_signer_secret,
            expires_in: custom_signer_expires_in,
        }
    }

    pub fn get_custom_signer_config(&self) -> CustomLinkSignerConfig {
        let custom_signer_host = Self::unwrap_env_var("CUSTOM_SIGNER_HOST");
        CustomLinkSignerConfig {
            host: custom_signer_host,
        }
    }

    pub fn get_minio_single_bucket_storage_config(&self) -> MinioSingleBucketStorageConfig {
        let sbs_bucket_name = Self::unwrap_env_var("SBS_BUCKET_NAME");
        let sbs_access_key = Self::unwrap_env_file("SBS_ACCESS_KEY_FILE");
        let sbs_secret_key = Self::unwrap_env_file("SBS_SECRET_KEY_FILE");
        Self::unwrap_env_var("SBS_REGION");
        Self::unwrap_env_var("SBS_HOST");

        MinioSingleBucketStorageConfig {
            bucket_name: sbs_bucket_name,
            credentials: Credentials::new(
                Some(&sbs_access_key),
                Some(&sbs_secret_key),
                None,
                None,
                None,
            )
            .unwrap(),
            direct_access_region: Region::from_env("SBS_REGION", Some("SBS_HOST")).unwrap(),
            public_access_region: if !self.with_proxy {
                match (
                    std::env::var("SBS_PUBLIC_REGION"),
                    std::env::var("SBS_PUBLIC_HOST"),
                ) {
                    (Ok(_), Ok(_)) => Some(
                        Region::from_env("SBS_PUBLIC_REGION", Some("SBS_PUBLIC_HOST")).unwrap(),
                    ),
                    _ => Some(Region::from_env("SBS_REGION", Some("SBS_HOST")).unwrap()),
                }
            } else {
                None
            },
        }
    }

    pub fn get_postgres_locks_provider_config(&self) -> PostgresLocksProviderConfig {
        let database_host = Self::unwrap_env_var("DATABASE_HOST");
        let database_name = Self::unwrap_env_var("DATABASE_NAME");
        let database_user = Self::unwrap_env_var("DATABASE_USER");
        let database_password = Self::unwrap_env_file("DATABASE_PASSWORD_FILE");
        PostgresLocksProviderConfig {
            host: database_host,
            dbname: database_name,
            username: database_user,
            password: database_password,
        }
    }

    fn unwrap_env_var(key: &str) -> String {
        match std::env::var(key) {
            Ok(value) => value,
            Err(_) => panic!("Missing environment variable: {}", key),
        }
    }

    fn unwrap_u64_env_var(key: &str) -> u64 {
        let var = Self::unwrap_env_var(key);
        match var.parse::<u64>() {
            Ok(value) => value,
            Err(_) => panic!("Invalid environment variable. Expected integer for {}", key),
        }
    }

    fn unwrap_env_file(key: &str) -> String {
        let path = Self::unwrap_env_var(key);
        match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => panic!("Failed to read file described by env variable {}", key),
        }
    }

    fn cli_parse_proxy(mut self, args: &Vec<String>) -> Self {
        if args.is_empty() || args[0] == "proxy" {
            self.with_proxy = true;
        } else if args[0] == "signer" {
            self.with_proxy = false;
        } else {
            panic!("Invalid arguments: {}", args.join(", "));
        }
        self
    }

    fn cli_parse_fs_impl(mut self, args: &Vec<String>) -> Self {
        if args.is_empty() || args.len() > 1 && args[1] == "fs" {
            self.file_storage_implementation = FileStorageImplementation::LocalFileStorage;
        } else if args.len() > 1 && args[1] == "sbs" {
            self.file_storage_implementation = FileStorageImplementation::MinioSingleBucketStorage;
        } else {
            panic!("Invalid arguments: {}", args.join(", "));
        }
        self
    }

    fn cli_parse_locks_impl(mut self, args: &Vec<String>) -> Self {
        if args.len() <= 2 {
            self.with_locks = false;
            self.locks_implementation = LocksImplementation::None;
        } else if args.len() == 4 && args[2] == "locks" && args[3] == "pg" {
            self.with_locks = true;
            self.locks_implementation = LocksImplementation::PostgresLocksProvider;
        } else {
            panic!("Invalid arguments: {}", args.join(", "));
        }
        self
    }
}
