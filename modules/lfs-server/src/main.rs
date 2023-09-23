use axum::{middleware, routing::post, Router};
use s3::{creds::Credentials, Region};
use std::net::SocketAddr;
use std::sync::Arc;

use lfs_info_server::{
    controllers::{errors::handle_and_filter_error_details, objects::batch::post_objects_batch},
    services::{minio::single_bucket_storage::MinioSingleBucketStorage, jwt_token_decoder::JwtTokenDecoder},
    traits::services::Services,
};

/* -------------------------------------------------------------------------- */
/*                            Dependency injection                            */
/* -------------------------------------------------------------------------- */

pub struct InjectedServices {
    fs: MinioSingleBucketStorage,
    token_decoder: JwtTokenDecoder,
}

impl InjectedServices {
    fn load_env_var_from_file(key: &str) -> String {
        let path = std::env::var(key).unwrap();
        let file = std::fs::read_to_string(path).unwrap();
        return file.trim().to_string();
    }

    pub fn new() -> InjectedServices {
        let bucket_name = std::env::var("SBS_BUCKET_NAME").unwrap();
        let credentials = Credentials::new(
            Some(&Self::load_env_var_from_file("SBS_ACCESS_KEY_FILE")),
            Some(&Self::load_env_var_from_file("SBS_SECRET_KEY_FILE")),
            None,
            None,
            None,
        )
        .unwrap();
        let region = Region::from_env("SBS_REGION", Some("SBS_HOST")).unwrap();
        InjectedServices {
            fs: MinioSingleBucketStorage::new(bucket_name, region, credentials),
            token_decoder: JwtTokenDecoder::from_file_env_var("JWT_SECRET_FILE"),
        }
    }
}

impl Services for InjectedServices {
    type TFileStorageMetaRequester = MinioSingleBucketStorage;
    type TFileStorageLinkSigner = MinioSingleBucketStorage;
    type TTokenDecoder = JwtTokenDecoder;

    fn file_storage_meta_requester(&self) -> &Self::TFileStorageMetaRequester {
        &self.fs
    }
    
    fn file_storage_link_signer(&self) -> &Self::TFileStorageLinkSigner {
        &self.fs
    }

    fn token_decoder(&self) -> &Self::TTokenDecoder {
        &self.token_decoder
    }
}

/* -------------------------------------------------------------------------- */
/*                                   Server                                   */
/* -------------------------------------------------------------------------- */

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Bundle services
    let services = Arc::new(InjectedServices::new());

    // build our application with a route
    let app = Router::new()
        // `POST /objects/batch?repo=a/b/c`
        .route("/objects/batch", post(post_objects_batch))
        // Error handling
        .layer(middleware::from_fn(handle_and_filter_error_details))
        .with_state(services);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
