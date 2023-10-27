use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use s3::{creds::Credentials, Region};
use std::net::SocketAddr;
use std::sync::Arc;

use lfs_info_server::{
    controllers::{
        errors::handle_and_filter_error_details,
        objects::{batch::post_objects_batch, upload::upload_object, download::download_object},
    },
    services::{
        custom_link_signer::CustomLinkSigner, jwt_token_encoder_decoder::JwtTokenEncoderDecoder,
        minio::single_bucket_storage::MinioSingleBucketStorage,
    },
    traits::{file_storage::{FileStorageProxy, FileStorageMetaRequester, FileStorageLinkSigner}, services::Services, token_encoder_decoder::TokenEncoderDecoder},
    server::RouterExt,
};

/* -------------------------------------------------------------------------- */
/*                            Dependency injection                            */
/* -------------------------------------------------------------------------- */

pub struct InjectedServices {
    fs: MinioSingleBucketStorage,
    token_encoder_decoder: JwtTokenEncoderDecoder,
    signer: CustomLinkSigner<JwtTokenEncoderDecoder>,
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
        let jwt_token_encoder_decoder =
            JwtTokenEncoderDecoder::from_file_env_var("JWT_SECRET_FILE", "JWT_EXPIRES_IN");
        let link_token_encoder_decoder = JwtTokenEncoderDecoder::from_file_env_var(
            "CUSTOM_SIGNER_SECRET_FILE",
            "CUSTOM_SIGNER_EXPIRES_IN",
        );

        InjectedServices {
            fs: MinioSingleBucketStorage::new(
                bucket_name,
                credentials,
                region,
                public_region,
            ),
            token_encoder_decoder: jwt_token_encoder_decoder,
            signer: CustomLinkSigner::from_env_var(
                "CUSTOM_SIGNER_HOST",
                link_token_encoder_decoder,
            ),
        }
    }
}

impl Services for InjectedServices {
    fn file_storage_meta_requester(&self) -> &(dyn FileStorageMetaRequester + 'static) {
        &self.fs
    }

    fn file_storage_link_signer(&self) -> &(dyn FileStorageLinkSigner + 'static) {
        &self.signer
    }

    fn token_encoder_decoder(&self) -> &(dyn TokenEncoderDecoder + 'static) {
        &self.token_encoder_decoder
    }

    fn file_storage_proxy(&self) -> Option<&(dyn FileStorageProxy + 'static)> {
        Some(&self.fs)
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
    let services: Arc<dyn Services + Send + Sync + 'static> = Arc::new(InjectedServices::new());

    // build our application with a route
    let app = Router::new()
        // `POST /objects/batch?repo=a/b/c`
        .directory_route("/objects/batch", post(post_objects_batch))
        // `PUT /objects/access/<oid>?repo=a/b/c`
        .directory_route("/objects/access/:oid", put(upload_object))
        // `GET /objects/access/<oid>?repo=a/b/c`
        .directory_route("/objects/access/:oid", get(download_object))
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
