use crate::{
    server::config::{FileStorageImplementation, LocksImplementation, ServerConfig},
    services::{
        custom_link_signer::CustomLinkSigner, fs::local_file_storage::LocalFileStorage,
        injected_services::InjectedServices, jwt_token_encoder_decoder::JwtTokenEncoderDecoder,
        minio::single_bucket_storage::MinioSingleBucketStorage,
        postgres::postgres_locks_provider::PostgresLocksProvider,
    },
    traits::{
        file_storage::{FileStorageLinkSigner, FileStorageMetaRequester, FileStorageProxy},
        locks::LocksProvider,
    },
};
use std::sync::Arc;

struct FileBackendServices(
    Arc<dyn FileStorageMetaRequester + 'static>,
    Option<Arc<dyn FileStorageProxy + 'static>>,
    Arc<dyn FileStorageLinkSigner + 'static>,
);

/**
 * Get the CustomLinkSigner implementation from the given configuration.
 */
fn get_custom_signer_implementation(
    config: &ServerConfig,
) -> Arc<dyn FileStorageLinkSigner + 'static> {
    Arc::new(CustomLinkSigner::from_config(
        &config.get_custom_signer_config(),
        JwtTokenEncoderDecoder::from_config(config.get_custom_signer_encoder_decoder_config()),
    ))
}

/**
 * Get the single bucket storage implementation from the given configuration.
 * Get in order the FileStorageMetaRequester, the FileStorageProxy, FileStorageLinkSigner implementations
 *
 * In proxy mode, FileStorageMetaRequester and FileStorageProxy are both a reference to an instance of MinioSingleBucketStorage
 *      and the signer is a CustomLinkSigner
 * In signer mode, FileStorageMetaRequester and FileStorageLinkSigner are both a reference to an instance of MinioSingleBucketStorage
 *      and the proxy is unset (not a proxy)
 */
fn get_sbs_implementation(config: &ServerConfig) -> FileBackendServices {
    let fs = Arc::new(MinioSingleBucketStorage::from_config(
        config.get_minio_single_bucket_storage_config(),
    ));

    if config.with_proxy {
        let custom_signer = get_custom_signer_implementation(config);
        FileBackendServices(fs.clone(), Some(fs), custom_signer)
    } else {
        FileBackendServices(fs.clone(), None, fs)
    }
}

/**
 * Get the file storage implementation from the given configuration.
 */
fn get_fs_implementation(config: &ServerConfig) -> FileBackendServices {
    let fs = Arc::new(LocalFileStorage::from_config(
        config.get_local_file_storage_config(),
    ));
    let custom_signer = get_custom_signer_implementation(config);
    FileBackendServices(fs.clone(), Some(fs), custom_signer)
}

/**
 * Create the services from the given configuration. Might panic when some environment variables
 * are missing.
 */
pub fn from_server_config(config: &ServerConfig) -> InjectedServices {
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
        FileStorageImplementation::MinioSingleBucketStorage => get_sbs_implementation(config),
        FileStorageImplementation::LocalFileStorage => get_fs_implementation(config),
    };

    // Get the locks provider implementation
    let locks_provider: Option<Arc<dyn LocksProvider>> = match config.locks_implementation {
        LocksImplementation::PostgresLocksProvider => Some(Arc::new(
            PostgresLocksProvider::from_config(config.get_postgres_locks_provider_config()),
        )),
        LocksImplementation::None => None,
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
