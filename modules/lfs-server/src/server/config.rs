use crate::services::{
    custom_link_signer::CustomLinkSignerConfig, fs::local_file_storage::LocalFileStorageConfig,
    jwt_token_encoder_decoder::JwtTokenEncoderDecoderConfig,
    minio::single_bucket_storage::MinioSingleBucketStorageConfig,
    postgres::postgres_locks_provider::PostgresLocksProviderConfig,
};
use s3::{creds::Credentials, Region};
use std::str::FromStr;

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

const FS_ROOT_PATH_KEY: &str = "FS_ROOT_PATH";
const DATABASE_HOST_KEY: &str = "DATABASE_HOST";
const DATABASE_NAME_KEY: &str = "DATABASE_NAME";
const DATABASE_USER_KEY: &str = "DATABASE_USER";
const DATABASE_PASSWORD_FILE_KEY: &str = "DATABASE_PASSWORD_FILE";
const SBS_BUCKET_NAME_KEY: &str = "SBS_BUCKET_NAME";
const SBS_ACCESS_KEY_FILE_KEY: &str = "SBS_ACCESS_KEY_FILE";
const SBS_SECRET_KEY_FILE_KEY: &str = "SBS_SECRET_KEY_FILE";
const SBS_REGION_KEY: &str = "SBS_REGION";
const SBS_HOST_KEY: &str = "SBS_HOST";
const SBS_PUBLIC_REGION_KEY: &str = "SBS_PUBLIC_REGION";
const SBS_PUBLIC_HOST_KEY: &str = "SBS_PUBLIC_HOST";
const CUSTOM_SIGNER_HOST_KEY: &str = "CUSTOM_SIGNER_HOST";
const JWT_SECRET_FILE_KEY: &str = "JWT_SECRET_FILE";
const JWT_EXPIRES_IN_KEY: &str = "JWT_EXPIRES_IN";
const CUSTOM_SIGNER_SECRET_FILE_KEY: &str = "CUSTOM_SIGNER_SECRET_FILE";
const CUSTOM_SIGNER_EXPIRES_IN_KEY: &str = "CUSTOM_SIGNER_EXPIRES_IN";

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

    // Locks db
    pub database_host: Option<String>,
    pub database_name: Option<String>,
    pub database_user: Option<String>,
    pub database_password: Option<String>,
}

impl ServerConfig {
    /**
     * Parse the CLI arguments.
     *
     * The CLI expect 0, 2 or 4 arguments:
     *   - If no arguments is provided, "proxy fs" is assumed
     *   - If 2 arguments are provided, it is expected to be "<signer|proxy> <fs|sbs>"
     *   - If 4 arguments are provided, it is expected to be "<signer|proxy> <fs|sbs> locks pg"
     */
    pub fn parse_args(self, args: Vec<String>) -> Self {
        self.cli_parse_proxy(&args)
            .cli_parse_fs_impl(&args)
            .cli_parse_locks_impl(&args)
    }

    /**
     * Get the config for a local file storage.
     *
     * The following environment variables are required:
     *   - FS_ROOT_PATH
     */
    pub fn get_local_file_storage_config(&self) -> LocalFileStorageConfig {
        LocalFileStorageConfig {
            root_path: Self::unwrap_config_value(FS_ROOT_PATH_KEY, &self.fs_root_path),
        }
    }

    /**
     * Get the config for a jwt token encoder/decoder.
     *
     * The following environment variables are required:
     *   - JWT_SECRET_FILE
     *   - JWT_EXPIRES_IN
     */
    pub fn get_jwt_token_encoder_decoder_config(&self) -> JwtTokenEncoderDecoderConfig {
        JwtTokenEncoderDecoderConfig {
            secret: Self::unwrap_config_value(JWT_SECRET_FILE_KEY, &self.jwt_secret),
            expires_in: Self::unwrap_config_value(JWT_EXPIRES_IN_KEY, &self.jwt_expires_in),
        }
    }

    /**
     * Get the config for a custom link signer.
     *
     * The following environment variables are required:
     *   - CUSTOM_SIGNER_SECRET_FILE
     *   - CUSTOM_SIGNER_EXPIRES_IN
     */
    pub fn get_custom_signer_encoder_decoder_config(&self) -> JwtTokenEncoderDecoderConfig {
        JwtTokenEncoderDecoderConfig {
            secret: Self::unwrap_config_value(
                CUSTOM_SIGNER_SECRET_FILE_KEY,
                &self.custom_signer_secret,
            ),
            expires_in: Self::unwrap_config_value(
                CUSTOM_SIGNER_EXPIRES_IN_KEY,
                &self.custom_signer_expires_in,
            ),
        }
    }

    /**
     * Get the config for a custom link signer.
     *
     * The following environment variables are required:
     *   - CUSTOM_SIGNER_HOST
     */
    pub fn get_custom_signer_config(&self) -> CustomLinkSignerConfig {
        CustomLinkSignerConfig {
            host: Self::unwrap_config_value(CUSTOM_SIGNER_HOST_KEY, &self.custom_signer_host),
        }
    }

    /**
     * Get the config for a minio single bucket storage.
     *
     * The following environment variables are required:
     *   - SBS_BUCKET_NAME
     *   - SBS_ACCESS_KEY_FILE
     *   - SBS_SECRET_KEY_FILE
     *   - SBS_REGION or SBS_HOST
     *
     * In proxy mode, the following environment variables are accepted.
     * If not provided, it will fallback on the SBS_REGION and SBS_HOST values:
     *   - SBS_PUBLIC_REGION or SBS_PUBLIC_HOST
     */
    pub fn get_minio_single_bucket_storage_config(&self) -> MinioSingleBucketStorageConfig {
        let sbs_bucket_name = Self::unwrap_config_value(SBS_BUCKET_NAME_KEY, &self.sbs_bucket_name);
        let sbs_access_key =
            Self::unwrap_config_value(SBS_ACCESS_KEY_FILE_KEY, &self.sbs_access_key);
        let sbs_secret_key =
            Self::unwrap_config_value(SBS_SECRET_KEY_FILE_KEY, &self.sbs_secret_key);

        let region = Self::get_region(self.sbs_region.clone(), self.sbs_host.clone());
        let region = match region {
            Some(region) => region,
            None => panic!(
                "Missing environment variable: {} or {}",
                SBS_REGION_KEY, SBS_HOST_KEY
            ),
        };

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
            direct_access_region: region.clone(),
            public_access_region: if !self.with_proxy {
                Some(
                    match Self::get_region(
                        self.sbs_public_region.clone(),
                        self.sbs_public_host.clone(),
                    ) {
                        Some(region) => region,
                        None => region,
                    },
                )
            } else {
                None
            },
        }
    }

    /**
     * Get the config for a postgres locks provider.
     *
     * The following environment variables are required:
     *   - DATABASE_HOST
     *   - DATABASE_NAME
     *   - DATABASE_USER
     *   - DATABASE_PASSWORD_FILE
     */
    pub fn get_postgres_locks_provider_config(&self) -> PostgresLocksProviderConfig {
        PostgresLocksProviderConfig {
            host: Self::unwrap_config_value(DATABASE_HOST_KEY, &self.database_host),
            dbname: Self::unwrap_config_value(DATABASE_NAME_KEY, &self.database_name),
            username: Self::unwrap_config_value(DATABASE_USER_KEY, &self.database_user),
            password: Self::unwrap_config_value(
                DATABASE_PASSWORD_FILE_KEY,
                &self.database_password,
            ),
        }
    }

    /**
     * Get the region from the region or host environment variables.
     *
     * If both are provided, a custom region is returned with the host as endpoint.
     * If only the host is provided, a custom region is returned with us-east-1 as region.
     * If only the region is provided, the corresponding region is returned.
     * If none are provided, None is returned.
     */
    fn get_region(region: Option<String>, host: Option<String>) -> Option<Region> {
        match (region, host) {
            (Some(region), Some(endpoint)) => Some(Region::Custom { region, endpoint }),
            (None, Some(host)) => Some(Region::Custom {
                region: "us-east-1".to_string(),
                endpoint: host,
            }),
            (Some(region), None) => Some(Region::from_str(region.as_str()).unwrap()),
            (None, None) => None,
        }
    }

    /**
     * Parse the environment variables and set the config values.
     * This do not verify that the values exists, but if they exist
     * they should be valid. (files shall exists, u64 shall be parsable, etc.)
     */
    pub fn parse_env(mut self) -> Self {
        self.fs_root_path = std::env::var(FS_ROOT_PATH_KEY).ok();
        self.database_host = std::env::var(DATABASE_HOST_KEY).ok();
        self.database_name = std::env::var(DATABASE_NAME_KEY).ok();
        self.database_user = std::env::var(DATABASE_USER_KEY).ok();
        self.database_password = Self::read_env_file(DATABASE_PASSWORD_FILE_KEY);
        self.sbs_bucket_name = std::env::var(SBS_BUCKET_NAME_KEY).ok();
        self.sbs_access_key = Self::read_env_file(SBS_ACCESS_KEY_FILE_KEY);
        self.sbs_secret_key = Self::read_env_file(SBS_SECRET_KEY_FILE_KEY);
        self.sbs_region = std::env::var(SBS_REGION_KEY).ok();
        self.sbs_host = std::env::var(SBS_HOST_KEY).ok();
        self.sbs_public_region = std::env::var(SBS_PUBLIC_REGION_KEY).ok();
        self.sbs_public_host = std::env::var(SBS_PUBLIC_HOST_KEY).ok();
        self.jwt_secret = Self::read_env_file(JWT_SECRET_FILE_KEY);
        self.jwt_expires_in = std::env::var(JWT_EXPIRES_IN_KEY)
            .ok()
            .map(|v| v.parse::<u64>().unwrap());
        self.custom_signer_host = std::env::var(CUSTOM_SIGNER_HOST_KEY).ok();
        self.custom_signer_secret = Self::read_env_file(CUSTOM_SIGNER_SECRET_FILE_KEY);
        self.custom_signer_expires_in = std::env::var(CUSTOM_SIGNER_EXPIRES_IN_KEY)
            .ok()
            .map(|v| v.parse::<u64>().unwrap());
        self
    }

    /**
     * Unwrap a config value or panic giving the key name if the value is missing.
     */
    fn unwrap_config_value<T: Clone>(key: &str, value: &Option<T>) -> T {
        match value {
            Some(v) => v.clone(),
            None => panic!("Missing environment variable: {}", key),
        }
    }

    /**
     * Read a file described by an environment variable.
     * If the variable is not set, return None.
     * If the variable is set, read the file and return its content.
     * If the file cannot be read, panic giving the key name.
     */
    fn read_env_file(key: &str) -> Option<String> {
        match std::env::var(key) {
            Ok(path) => match std::fs::read_to_string(path) {
                Ok(content) => Some(content),
                Err(_) => panic!("Failed to read file described by env variable {}", key),
            },
            Err(_) => None,
        }
    }

    /**
     * Parse the proxy/signer CLI argument.
     */
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

    /**
     * Parse the fs implementation CLI argument.
     */
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

    /**
     * Parse the locks CLI arguments.
     */
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
