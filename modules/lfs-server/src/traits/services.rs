use super::{file_storage::{FileStorageMetaRequester, FileStorageLinkSigner}, token_decoder::TokenDecoder};

pub trait Services {
    type TFileStorageMetaRequester: FileStorageMetaRequester;
    type TFileStorageLinkSigner: FileStorageLinkSigner;
    type TTokenDecoder: TokenDecoder;

    fn file_storage_meta_requester(&self) -> &Self::TFileStorageMetaRequester;
    fn file_storage_link_signer(&self) -> &Self::TFileStorageLinkSigner;
    fn token_decoder(&self) -> &Self::TTokenDecoder;
}
