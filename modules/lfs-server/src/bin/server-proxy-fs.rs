use lfs_info_server::{
    server::run_server,
    services::{
        custom_link_signer::CustomLinkSigner, fs::local_file_storage::LocalFileStorage,
        jwt_token_encoder_decoder::JwtTokenEncoderDecoder,
    },
    traits::{
        file_storage::{FileStorageLinkSigner, FileStorageMetaRequester, FileStorageProxy},
        services::Services,
        token_encoder_decoder::TokenEncoderDecoder,
    },
};

/* -------------------------------------------------------------------------- */
/*                            Dependency injection                            */
/* -------------------------------------------------------------------------- */

pub struct InjectedServices {
    fs: LocalFileStorage,
    token_encoder_decoder: JwtTokenEncoderDecoder,
    signer: CustomLinkSigner<JwtTokenEncoderDecoder>,
}

impl Default for InjectedServices {
    fn default() -> Self {
        InjectedServices::new()
    }
}

impl InjectedServices {
    pub fn new() -> InjectedServices {
        let root_path = std::env::var("FS_ROOT_PATH").unwrap();
        let jwt_token_encoder_decoder =
            JwtTokenEncoderDecoder::from_file_env_var("JWT_SECRET_FILE", "JWT_EXPIRES_IN");
        let link_token_encoder_decoder = JwtTokenEncoderDecoder::from_file_env_var(
            "CUSTOM_SIGNER_SECRET_FILE",
            "CUSTOM_SIGNER_EXPIRES_IN",
        );

        InjectedServices {
            fs: LocalFileStorage::new(root_path),
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
    run_server::<InjectedServices>(true, false).await;
}
