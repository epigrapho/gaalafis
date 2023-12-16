use serde::Serialize;

use super::super::enums::{HashAlgorithm, Transfer};

/// Error details about an object
#[derive(Serialize, Debug)]
pub struct ObjectError {
    message: String,
}

/// Object identification and error details, when object action is not available
#[derive(Serialize, Debug)]
pub struct ObjectWithError {
    oid: String,
    size: u32,
    error: ObjectError,
}

/// Represent the headers explicitly returned by the server, that need to be including in any future request to the server.
#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AuthorizationHeader {
    pub authorization: String,
}

/// An object action represents the next request to do to upload/download an object.
#[derive(Serialize, Debug)]
pub struct ObjectAction {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<AuthorizationHeader>,
    pub expires_in: u64,
}

impl ObjectAction {
    pub fn new(href: String, authorization: Option<&str>, expires_in: u64) -> ObjectAction {
        ObjectAction {
            href,
            header: authorization.map(|authorization| AuthorizationHeader {
                authorization: authorization.to_string(),
            }),
            expires_in,
        }
    }
}

/// For download operation, the api returns how to download the object
#[derive(Serialize, Debug)]
pub struct DownloadActions {
    download: ObjectAction,
}

/// For upload operation, the api returns how to upload the object, and if implemented how to verify that upload was successful
#[derive(Serialize, Debug)]
pub struct UploadActions {
    upload: ObjectAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    verify: Option<ObjectAction>,
}

/// According to the operation, the api returns either download or upload+verify actions
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum ObjectActions {
    Download(DownloadActions),
    Upload(UploadActions),
}

impl ObjectActions {
    pub fn download(download: ObjectAction) -> ObjectActions {
        ObjectActions::Download(DownloadActions { download })
    }
    pub fn upload(upload: ObjectAction, verify: Option<ObjectAction>) -> ObjectActions {
        ObjectActions::Upload(UploadActions { upload, verify })
    }
}

/// The object identification and the actions specifications
#[derive(Serialize, Debug)]
pub struct ObjectWithAvailableActions {
    oid: String,
    size: u32,
    actions: ObjectActions,
}

/// When the server search for the object, either an action is returned, or an error.
/// In any case, we return the object identification and size next to the error or action.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Object {
    ObjectWithAvailableActions(ObjectWithAvailableActions),
    ObjectWithError(ObjectWithError),
}

impl Object {
    fn success(oid: &str, size: u32, actions: ObjectActions) -> Object {
        Object::ObjectWithAvailableActions(ObjectWithAvailableActions {
            oid: oid.to_string(),
            size,
            actions,
        })
    }

    /// Initialize with a not found error
    pub fn not_found(oid: &str, size: u32) -> Object {
        Object::ObjectWithError(ObjectWithError {
            oid: oid.to_string(),
            size,
            error: ObjectError {
                message: String::from("Not found"),
            },
        })
    }

    /// Initialize with an upload and optionally a verify action
    pub fn upload(
        oid: &str,
        size: u32,
        upload: ObjectAction,
        verify: Option<ObjectAction>,
    ) -> Object {
        Object::success(oid, size, ObjectActions::upload(upload, verify))
    }

    /// Initialize with a download action
    pub fn download(oid: &str, size: u32, download: ObjectAction) -> Object {
        Object::success(oid, size, ObjectActions::download(download))
    }

    /// Initialize with a custom error
    pub fn error(oid: &str, size: u32, error: Box<dyn std::error::Error>) -> Object {
        Object::ObjectWithError(ObjectWithError {
            oid: oid.to_string(),
            size,
            error: ObjectError {
                message: error.to_string(),
            },
        })
    }
}

/// The response to the objects/batch request
#[derive(Serialize, Debug)]
pub struct ObjectsBatchSuccessResponse {
    transfer: Transfer,
    objects: Vec<Object>,
    hash_algo: HashAlgorithm,
}

impl ObjectsBatchSuccessResponse {
    /// Initialize with a basic transfer type and sha256 hash algorithm
    ///
    /// # Examples
    ///
    /// ```
    /// use lfs_info_server::api::{enums::*,objects_batch::response::*};
    /// let res = ObjectsBatchSuccessResponse::basic_sha256(vec![
    ///   Object::download("oid1", 1, ObjectAction::new("href1".to_string(), None, 1)),
    ///   Object::upload("oid2", 2, ObjectAction::new("href2".to_string(), None, 2), None),
    ///   Object::not_found("oid3", 3),
    ///   Object::error("oid4", 4, Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test"))),
    /// ]);
    /// ```
    pub fn basic_sha256(objects: Vec<Object>) -> ObjectsBatchSuccessResponse {
        ObjectsBatchSuccessResponse {
            transfer: Transfer::Basic,
            objects,
            hash_algo: HashAlgorithm::Sha256,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_objects_batch_success_response() {
        let (href1, href2) = ("href1".to_string(), "href2".to_string());
        let e = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        let res = ObjectsBatchSuccessResponse::basic_sha256(vec![
            Object::download("oid1", 1, ObjectAction::new(href1, None, 1)),
            Object::upload("oid2", 2, ObjectAction::new(href2, None, 2), None),
            Object::not_found("oid3", 3),
            Object::error("oid4", 4, e),
        ]);
        let json = serde_json::to_string(&res).unwrap();
        assert_eq!(
            json,
            r#"{"transfer":"basic","objects":[{"oid":"oid1","size":1,"actions":{"download":{"href":"href1","expires_in":1}}},{"oid":"oid2","size":2,"actions":{"upload":{"href":"href2","expires_in":2}}},{"oid":"oid3","size":3,"error":{"message":"Not found"}},{"oid":"oid4","size":4,"error":{"message":"test"}}],"hash_algo":"sha256"}"#
        );
    }
}
