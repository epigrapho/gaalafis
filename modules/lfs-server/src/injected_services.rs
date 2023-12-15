use crate::config::{FileStorageImplementation, ServerConfig};
use lfs_info_server::{
    services::{
        custom_link_signer::CustomLinkSigner, fs::local_file_storage::LocalFileStorage,
        jwt_token_encoder_decoder::JwtTokenEncoderDecoder,
        minio::single_bucket_storage::MinioSingleBucketStorage,
        postgres::postgres_locks_provider::PostgresLocksProvider,
    },
    traits::{
        file_storage::{FileStorageLinkSigner, FileStorageMetaRequester, FileStorageProxy},
        locks::LocksProvider,
        services::Services,
        token_encoder_decoder::TokenEncoderDecoder,
    },
};
use std::sync::Arc;

pub struct InjectedServices {
    pub file_storage_meta_requester: Arc<dyn FileStorageMetaRequester + 'static>,
    pub file_storage_proxy: Option<Arc<dyn FileStorageProxy + 'static>>,
    pub file_storage_link_signer: Arc<dyn FileStorageLinkSigner + 'static>,
    pub token_encoder_decoder: Arc<dyn TokenEncoderDecoder + 'static>,
    pub locks_provider: Option<Arc<dyn LocksProvider + 'static>>,
}

struct FileBackendServices(
    Arc<dyn FileStorageMetaRequester + 'static>,
    Option<Arc<dyn FileStorageProxy + 'static>>,
    Arc<dyn FileStorageLinkSigner + 'static>,
);

impl InjectedServices {
    fn get_custom_signer_implementation(
        config: &ServerConfig,
    ) -> Arc<dyn FileStorageLinkSigner + 'static> {
        Arc::new(CustomLinkSigner::from_config(
            &config.get_custom_signer_config(),
            JwtTokenEncoderDecoder::from_config(config.get_custom_signer_encoder_decoder_config()),
        ))
    }

    fn get_sbs_implementation(config: &ServerConfig) -> FileBackendServices {
        let fs = Arc::new(MinioSingleBucketStorage::from_config(
            config.get_minio_single_bucket_storage_config(),
        ));

        if config.with_proxy {
            let custom_signer = Self::get_custom_signer_implementation(config);
            FileBackendServices(fs.clone(), Some(fs), custom_signer)
        } else {
            FileBackendServices(fs.clone(), None, fs)
        }
    }

    fn get_fs_implementation(config: &ServerConfig) -> FileBackendServices {
        let fs = Arc::new(LocalFileStorage::from_config(
            config.get_local_file_storage_config(),
        ));
        let custom_signer = Self::get_custom_signer_implementation(config);
        FileBackendServices(fs.clone(), Some(fs), custom_signer)
    }

    pub fn new(config: &ServerConfig) -> Self {
        // Token encoder decoder (only jwt for now)
        let token_encoder_decoder = Arc::new(JwtTokenEncoderDecoder::from_config(
            config.get_jwt_token_encoder_decoder_config(),
        ));

        // Get the file storage, file proxy and link signer implementations
        let FileBackendServices(
            file_storage_meta_requester,
            file_storage_proxy,
            file_storage_link_signer,
        ) = match &config.file_storage_implementation {
            FileStorageImplementation::MinioSingleBucketStorage => {
                Self::get_sbs_implementation(config)
            }
            FileStorageImplementation::LocalFileStorage => Self::get_fs_implementation(config),
        };

        // Get the locks provider implementation
        let locks_provider: Option<Arc<dyn LocksProvider>> = match config.locks_implementation {
            crate::config::LocksImplementation::PostgresLocksProvider => Some(Arc::new(
                PostgresLocksProvider::from_config(config.get_postgres_locks_provider_config()),
            )),
            crate::config::LocksImplementation::None => None,
        };

        // Bundle everything into a struct
        InjectedServices {
            file_storage_meta_requester,
            file_storage_proxy,
            file_storage_link_signer,
            token_encoder_decoder,
            locks_provider,
        }
    }
}

impl Services for InjectedServices {
    fn file_storage_meta_requester(&self) -> &(dyn FileStorageMetaRequester + 'static) {
        self.file_storage_meta_requester.as_ref()
    }

    fn file_storage_link_signer(&self) -> &(dyn FileStorageLinkSigner + 'static) {
        self.file_storage_link_signer.as_ref()
    }

    fn token_encoder_decoder(&self) -> &(dyn TokenEncoderDecoder + 'static) {
        self.token_encoder_decoder.as_ref()
    }

    fn file_storage_proxy(&self) -> Option<&(dyn FileStorageProxy + 'static)> {
        self.file_storage_proxy.as_ref().map(|x| x.as_ref())
    }

    fn locks_provider(&self) -> Option<&(dyn LocksProvider + 'static)> {
        self.locks_provider.as_ref().map(|x| x.as_ref())
    }
}
