use s3::{creds::Credentials, Region};

use lfs_info_server::{
    server::run_server,
    services::{
        jwt_token_encoder_decoder::JwtTokenEncoderDecoder,
        minio::single_bucket_storage::MinioSingleBucketStorage,
        postgres::postgres_locks_provider::PostgresLocksProvider,
    },
    traits::{
        file_storage::{FileStorageLinkSigner, FileStorageMetaRequester},
        locks::LocksProvider,
        services::Services,
        token_encoder_decoder::TokenEncoderDecoder,
    },
};

/* -------------------------------------------------------------------------- */
/*                            Dependency injection                            */
/* -------------------------------------------------------------------------- */

pub struct InjectedServices {
    fs: MinioSingleBucketStorage,
    token_encoder_decoder: JwtTokenEncoderDecoder,
    locks_provider: Option<PostgresLocksProvider>,
}

impl Default for InjectedServices {
    fn default() -> Self {
        InjectedServices::new()
    }
}

impl InjectedServices {
    fn load_env_var_from_file(key: &str) -> String {
        let path = std::env::var(key).unwrap();
        let file = std::fs::read_to_string(path).unwrap();
        return file.trim().to_string();
    }

    pub fn new() -> InjectedServices {
        // Bucket
        let bucket_name = std::env::var("SBS_BUCKET_NAME").unwrap();
        let credentials = Credentials::new(
            Some(&Self::load_env_var_from_file("SBS_ACCESS_KEY_FILE")),
            Some(&Self::load_env_var_from_file("SBS_SECRET_KEY_FILE")),
            None,
            None,
            None,
        )
        .unwrap();
        let public_sbs_region = std::env::var("SBS_PUBLIC_REGION");
        let public_sbs_host = std::env::var("SBS_PUBLIC_HOST");
        let public_region = match (public_sbs_region, public_sbs_host) {
            (Ok(region), Ok(host)) => Some(Region::Custom {
                region,
                endpoint: host,
            }),
            _ => None,
        };
        let region = Region::from_env("SBS_REGION", Some("SBS_HOST")).unwrap();
        InjectedServices {
            fs: MinioSingleBucketStorage::new(bucket_name, credentials, region, public_region),
            token_encoder_decoder: JwtTokenEncoderDecoder::from_file_env_var(
                "JWT_SECRET_FILE",
                "JWT_EXPIRES_IN",
            ),
            locks_provider: PostgresLocksProvider::try_new_from_env_variables(
                "DATABASE_HOST",
                "DATABASE_NAME",
                "DATABASE_USER",
                "DATABASE_PASSWORD_FILE",
            ),
        }
    }
}

impl Services for InjectedServices {
    fn file_storage_meta_requester(&self) -> &(dyn FileStorageMetaRequester + 'static) {
        &self.fs
    }

    fn file_storage_link_signer(&self) -> &(dyn FileStorageLinkSigner + 'static) {
        &self.fs
    }

    fn token_encoder_decoder(&self) -> &(dyn TokenEncoderDecoder + 'static) {
        &self.token_encoder_decoder
    }

    fn locks_provider(&self) -> Option<&(dyn LocksProvider + 'static)> {
        self.locks_provider
            .as_ref()
            .map(|lp| lp as &(dyn LocksProvider))
    }
}

/* -------------------------------------------------------------------------- */
/*                                   Server                                   */
/* -------------------------------------------------------------------------- */

#[tokio::main]
async fn main() {
    run_server::<InjectedServices>(false, true).await;
}
