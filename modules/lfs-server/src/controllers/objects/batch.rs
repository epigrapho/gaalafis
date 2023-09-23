use axum::{
    extract::{Json, Query, State},
    http::{HeaderMap, StatusCode},
};
use std::sync::Arc;

use crate::{
    api::{
        jwt::{Jwt,RepoTokenPayload},
        enums::{Operation, HashAlgorithm, Transfer},
        objects_batch::{
            body::ObjectsBatchRequestPayload,
            response::{ObjectsBatchSuccessResponse, Object},
        },
        repo_query::QueryRepo,
    },
    traits::{
        file_storage::{FileStorageLinkSigner, FileStorageMetaRequester, FileStorageMetaResult},
        services::Services,
    },
};

// Request the ability to transfer a batch of objects
// Might be download or upload
// Implements the Batch api defined at https://github.com/git-lfs/git-lfs/blob/main/docs/api/batch.md
// Expect a token in the header
// Available at /objects/batch?repo=a/b/c
pub async fn post_objects_batch(
    headers: HeaderMap,
    query: Query<QueryRepo>,
    State(services): State<Arc<impl Services>>,
    Json(payload): Json<ObjectsBatchRequestPayload>,
) -> Result<Json<ObjectsBatchSuccessResponse>, (StatusCode, String)> {
    // 1) Extract and validate
    let token_decoder = services.token_decoder();
    let jwt = Jwt::from_headers(headers, token_decoder)?;
    let jwt_payload = RepoTokenPayload::new(&jwt)?;
    
    payload.assert_jwt_access_level_higher_than_requested(&jwt_payload)?;
    
    query.assert_repo_match_token(&jwt_payload)?;
    
    payload.assert_hash_algo(HashAlgorithm::Sha256)?;
    
    payload.assert_transfer_accepted(Transfer::Basic)?;
    
    // 7) For each object, verify if it exists on storage server
    let services = Arc::clone(&services);
    
    let repo = &query.repo;
    let number_of_objects = payload.objects.len();
    let mut objects: Vec<Object> = Vec::with_capacity(number_of_objects);
    for object in payload.objects.iter() {
        let oid = &object.oid[..];
        let size = object.size;
        let result = services
            .file_storage_meta_requester()
            .get_meta_result(repo, oid)
            .await;

        let FileStorageMetaResult { exists, .. } = result;
        let signer = services.file_storage_link_signer();

        let object = if exists {
            let download =  signer.get_presigned_link(result).await;
            match download {
                Ok(download) => Object::download(oid, size, download),
                Err(error) => Object::error(oid, size, error),
            }
        } else if let Operation::Upload = payload.operation {
            let actions = signer.post_presigned_link(result, size).await;
            match actions {
                Ok(actions) => Object::upload(oid, size, actions.0, actions.1),
                Err(error) => Object::error(oid, size, error),
            }
        } else {
            Object::not_found(oid, size)
        };

        objects.push(object);
    }
    
    // 8) Return the result
    let response = ObjectsBatchSuccessResponse::basic_sha256(objects);
    
    Ok(Json(response))
}
