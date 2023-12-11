use axum::http::StatusCode;
use serde::Deserialize;

use crate::api::{
    enums::{from_hash_algo_string, from_transfer_string, HashAlgorithm, Operation, Transfer},
    jwt::RepoTokenPayload,
};

// The requested object identification (oid, and size). This allow for multiple objects to be requested in a single request.
#[derive(Deserialize, PartialEq, Debug)]
pub struct ObjectIdentity {
    pub oid: String,
    pub size: u32,
}

/// A reference to a git pointer. Not used yet. Might be specified in the body
#[derive(Deserialize)]
pub struct Ref {
    // name
}

/// The body of the objects/batch request.
#[derive(Deserialize, PartialEq, Debug)]
pub struct ObjectsBatchRequestPayload {
    pub operation: Operation,
    #[serde(deserialize_with = "from_transfer_string", default)]
    pub transfers: Option<Vec<Transfer>>,
    pub objects: Vec<ObjectIdentity>,
    #[serde(deserialize_with = "from_hash_algo_string")]
    pub hash_algo: HashAlgorithm,
}

impl ObjectsBatchRequestPayload {
    /// Verify that the basic transfer type is accepted by the client. If no transfer type is specified, it's ok, and we assume basic.
    ///
    /// # Examples
    ///
    /// Explicitly specified basic transfer:
    ///
    /// ```
    /// use lfs_info_server::api::{enums::*,objects_batch::body::ObjectsBatchRequestPayload};
    /// let payload = ObjectsBatchRequestPayload {
    ///     operation: Operation::Download,
    ///     transfers: Some(vec![Transfer::Basic]),
    ///     objects: vec![],
    ///     hash_algo: HashAlgorithm::Sha256,
    /// };
    /// payload.assert_transfer_accepted(Transfer::Basic).unwrap();
    /// ```
    ///
    pub fn assert_transfer_accepted(
        &self,
        accepted_transfer: Transfer,
    ) -> Result<(), (StatusCode, String)> {
        // Throw if all objects are not basic
        if let Some(transfers) = &self.transfers {
            for transfer in transfers {
                if &accepted_transfer == transfer {
                    return Ok(());
                }
            }
            return Err((
                StatusCode::NOT_IMPLEMENTED,
                String::from("Only basic transfer is supported"),
            ));
        }

        // No transfer specified, assume basic
        if let Transfer::Basic = accepted_transfer {
            return Ok(());
        }
        Err((
            StatusCode::NOT_IMPLEMENTED,
            String::from("Only basic transfer is supported"),
        ))
    }

    /// Verify that the hash algo is sha256, as it's the only one supported.
    ///
    /// # Examples
    ///
    /// ```
    /// use lfs_info_server::api::{enums::*,objects_batch::body::ObjectsBatchRequestPayload};
    /// let payload = ObjectsBatchRequestPayload {
    ///    operation: Operation::Download,
    ///    transfers: Some(vec![Transfer::Basic]),
    ///    objects: vec![],
    ///    hash_algo: HashAlgorithm::Sha256,
    /// };
    /// payload.assert_hash_algo(HashAlgorithm::Sha256).unwrap();
    /// ```
    ///
    pub fn assert_hash_algo(&self, algo: HashAlgorithm) -> Result<(), (StatusCode, String)> {
        if algo != self.hash_algo {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                String::from("Invalid hash algo, only sha256 is supported"),
            ));
        }

        Ok(())
    }

    /// Verify that the access level of the token is higher than the requested operation.
    pub fn assert_jwt_access_level_higher_than_requested(
        &self,
        jwt_payload: &RepoTokenPayload,
    ) -> Result<(), (StatusCode, String)> {
        if jwt_payload.has_write_access() {
            Ok(())
        } else if let Operation::Upload = &self.operation {
            Err((
                StatusCode::FORBIDDEN,
                "You only have read access to this repository".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;

    use crate::api::{
        enums::{HashAlgorithm, Operation, Transfer},
        jwt::RepoTokenPayload,
        objects_batch::body::ObjectsBatchRequestPayload,
    };

    #[test]
    fn test_assert_transfer_accepted_unspecified_transfer_ok() {
        let payload = ObjectsBatchRequestPayload {
            operation: Operation::Download,
            transfers: None,
            objects: vec![],
            hash_algo: HashAlgorithm::Sha256,
        };
        payload.assert_transfer_accepted(Transfer::Basic).unwrap();
    }

    #[test]
    fn test_assert_transfer_accepted_multiple_transfer_accepted() {
        let payload = ObjectsBatchRequestPayload {
            operation: Operation::Download,
            transfers: Some(vec![Transfer::Unknown, Transfer::Basic]),
            objects: vec![],
            hash_algo: HashAlgorithm::Sha256,
        };
        payload.assert_transfer_accepted(Transfer::Basic).unwrap();
    }

    #[test]
    fn test_assert_transfer_accepted_basic_not_accepted() {
        let payload = ObjectsBatchRequestPayload {
            operation: Operation::Download,
            transfers: Some(vec![Transfer::Unknown]),
            objects: vec![],
            hash_algo: HashAlgorithm::Sha256,
        };
        let (status, _) = payload
            .assert_transfer_accepted(Transfer::Basic)
            .unwrap_err();
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED)
    }

    #[test]
    fn test_assert_hash_algo_unknown_algo() {
        let payload = ObjectsBatchRequestPayload {
            operation: Operation::Download,
            transfers: Some(vec![Transfer::Basic]),
            objects: vec![],
            hash_algo: HashAlgorithm::Unknown,
        };
        let (status, _) = payload.assert_hash_algo(HashAlgorithm::Sha256).unwrap_err();
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY)
    }

    #[test]
    fn test_assert_jwt_access_level_higher_than_requested_upload_rights_to_download() {
        let payload = ObjectsBatchRequestPayload {
            operation: Operation::Download,
            transfers: Some(vec![Transfer::Basic]),
            objects: vec![],
            hash_algo: HashAlgorithm::Unknown,
        };
        let jwt_payload = RepoTokenPayload::new_for_test("foo", "upload");
        payload
            .assert_jwt_access_level_higher_than_requested(&jwt_payload)
            .unwrap();
    }

    #[test]
    fn test_assert_jwt_access_level_higher_than_requested_download_rights_to_upload_fail() {
        let payload = ObjectsBatchRequestPayload {
            operation: Operation::Upload,
            transfers: Some(vec![Transfer::Basic]),
            objects: vec![],
            hash_algo: HashAlgorithm::Unknown,
        };
        let jwt_payload = RepoTokenPayload::new_for_test("foo", "download");
        let (status, _) = payload
            .assert_jwt_access_level_higher_than_requested(&jwt_payload)
            .unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[test]
    fn deserialize_objects_batch_request_payload() {
        let serialized = "{\"operation\":\"download\",\"transfers\":[\"basic\"],\"objects\":[{\"oid\":\"oid1\",\"size\":1},{\"oid\":\"oid2\",\"size\":2}],\"hash_algo\":\"sha256\"}";
        let deserialized: ObjectsBatchRequestPayload = serde_json::from_str(serialized).unwrap();
        assert_eq!(
            deserialized,
            ObjectsBatchRequestPayload {
                operation: Operation::Download,
                transfers: Some(vec![Transfer::Basic]),
                objects: vec![
                    crate::api::objects_batch::body::ObjectIdentity {
                        oid: "oid1".to_string(),
                        size: 1,
                    },
                    crate::api::objects_batch::body::ObjectIdentity {
                        oid: "oid2".to_string(),
                        size: 2,
                    },
                ],
                hash_algo: HashAlgorithm::Sha256,
            }
        );
    }

    #[test]
    #[should_panic]
    fn deserialize_unknown_operation() {
        let serialized = "{\"operation\":\"foo\",\"transfers\":[\"basic\"],\"objects\":[],\"hash_algo\":\"sha256\"}";
        let _deserialized: ObjectsBatchRequestPayload = serde_json::from_str(serialized).unwrap();
    }

    #[test]
    fn deserialize_unknown_transfer() {
        let serialized = "{\"operation\":\"download\",\"transfers\":[\"foo\"],\"objects\":[],\"hash_algo\":\"sha256\"}";
        let deserialized: ObjectsBatchRequestPayload = serde_json::from_str(serialized).unwrap();
        assert_eq!(
            deserialized,
            ObjectsBatchRequestPayload {
                operation: Operation::Download,
                transfers: Some(vec![Transfer::Unknown]),
                objects: vec![],
                hash_algo: HashAlgorithm::Sha256,
            }
        );
    }

    #[test]
    fn deserialize_unknown_hash_algo() {
        let serialized = "{\"operation\":\"download\",\"transfers\":[\"basic\"],\"objects\":[],\"hash_algo\":\"foo\"}";
        let deserialized: ObjectsBatchRequestPayload = serde_json::from_str(serialized).unwrap();
        assert_eq!(
            deserialized,
            ObjectsBatchRequestPayload {
                operation: Operation::Download,
                transfers: Some(vec![Transfer::Basic]),
                objects: vec![],
                hash_algo: HashAlgorithm::Unknown,
            }
        );
    }

    #[test]
    fn deserialize_no_transfer() {
        let serialized = "{\"operation\":\"download\",\"objects\":[],\"hash_algo\":\"sha256\"}";
        let deserialized: ObjectsBatchRequestPayload = serde_json::from_str(serialized).unwrap();
        assert_eq!(
            deserialized,
            ObjectsBatchRequestPayload {
                operation: Operation::Download,
                transfers: None,
                objects: vec![],
                hash_algo: HashAlgorithm::Sha256,
            }
        );
    }

    #[test]
    fn test_assert_jwt_access_level_higher_than_requested_download_rights_to_download() {
        let payload = ObjectsBatchRequestPayload {
            operation: Operation::Download,
            transfers: Some(vec![Transfer::Basic]),
            objects: vec![],
            hash_algo: HashAlgorithm::Unknown,
        };
        let jwt_payload = RepoTokenPayload::new_for_test("foo", "download");
        payload
            .assert_jwt_access_level_higher_than_requested(&jwt_payload)
            .unwrap();
    }

    #[test]
    fn test_assert_jwt_access_level_higher_than_requested_upload_rights_to_upload() {
        let payload = ObjectsBatchRequestPayload {
            operation: Operation::Upload,
            transfers: Some(vec![Transfer::Basic]),
            objects: vec![],
            hash_algo: HashAlgorithm::Unknown,
        };
        let jwt_payload = RepoTokenPayload::new_for_test("foo", "upload");
        payload
            .assert_jwt_access_level_higher_than_requested(&jwt_payload)
            .unwrap();
    }

    #[test]
    fn test_assert_jwt_access_level_higher_than_requested_download_rights_to_upload() {
        let payload = ObjectsBatchRequestPayload {
            operation: Operation::Upload,
            transfers: Some(vec![Transfer::Basic]),
            objects: vec![],
            hash_algo: HashAlgorithm::Unknown,
        };
        let jwt_payload = RepoTokenPayload::new_for_test("foo", "download");
        let (status, msg) = payload
            .assert_jwt_access_level_higher_than_requested(&jwt_payload)
            .unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(msg, "You only have read access to this repository");
    }
}
