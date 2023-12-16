use crate::traits::services::Services;
use crate::traits::{
    file_storage::{FileStorageLinkSigner, FileStorageMetaRequester, FileStorageProxy},
    locks::LocksProvider,
    token_encoder_decoder::TokenEncoderDecoder,
};
use std::sync::Arc;

pub struct InjectedServices {
    pub file_storage_meta_requester: Arc<dyn FileStorageMetaRequester + 'static>,
    pub file_storage_proxy: Option<Arc<dyn FileStorageProxy + 'static>>,
    pub file_storage_link_signer: Arc<dyn FileStorageLinkSigner + 'static>,
    pub token_encoder_decoder: Arc<dyn TokenEncoderDecoder + 'static>,
    pub locks_provider: Option<Arc<dyn LocksProvider + 'static>>,
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

    fn locks_provider(&self) -> Option<&(dyn LocksProvider + 'static)> {
        self.locks_provider.as_ref().map(|x| x.as_ref())
    }

    fn file_storage_proxy(&self) -> Option<&(dyn FileStorageProxy + 'static)> {
        self.file_storage_proxy.as_ref().map(|x| x.as_ref())
    }
}
