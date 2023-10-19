use super::{
    file_storage::{FileStorageLinkSigner, FileStorageMetaRequester, FileStorageProxy},
    locks::LocksProvider,
    token_encoder_decoder::TokenEncoderDecoder,
};

pub trait Services: Send + Sync {
    fn file_storage_meta_requester(&self) -> &(dyn FileStorageMetaRequester + 'static);
    fn file_storage_link_signer(&self) -> &(dyn FileStorageLinkSigner + 'static);
    fn token_encoder_decoder(&self) -> &(dyn TokenEncoderDecoder + 'static);
    fn locks_provider(&self) -> Option<&(dyn LocksProvider + 'static)> {
        None
    }

    fn file_storage_proxy(&self) -> Option<&(dyn FileStorageProxy + 'static)> {
        None
    }
}
