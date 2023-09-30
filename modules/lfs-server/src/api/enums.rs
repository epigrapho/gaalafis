use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// the way the object is hashed to create the oid
#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum HashAlgorithm {
    Sha256,
    Unknown,
}

pub fn from_hash_algo_string<'de, D>(deserializer: D) -> Result<HashAlgorithm, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let hash_algo: String = String::deserialize(deserializer)?;
    match hash_algo.as_str() {
        "sha256" => Ok(HashAlgorithm::Sha256),
        _ => Ok(HashAlgorithm::Unknown),
    }
}

/// The requested / actual operation
/// If Upload is requested on an existing object, the server will specify that the operation is actually download.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Download,
    Upload,
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Download => write!(f, "download"),
            Operation::Upload => write!(f, "upload"),
        }
    }
}

/// The transfer type requested by the client
/// For now, only basic is supported.
/// If no transfer type is specified, the server will assume basic, but if the client specify that it only accepts a different transfer type, the server will return an error.
#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Transfer {
    Basic,
    Unknown,
}

pub fn from_transfer_string<'de, D>(deserializer: D) -> Result<Option<Vec<Transfer>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let transfers: Option<Vec<String>> = Option::deserialize(deserializer)?;
    match transfers {
        Some(transfers) => {
            let transfers: Vec<Transfer> = transfers
                .into_iter()
                .map(|transfer| match transfer.as_str() {
                    "basic" => Transfer::Basic,
                    _ => Transfer::Unknown,
                })
                .collect();
            Ok(Some(transfers))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct TestHoldingTransfers {
        #[serde(deserialize_with = "from_transfer_string", skip_serializing_if = "Option::is_none", default)]
        transfers: Option<Vec<Transfer>>,
    }
    
    #[test]
    fn test_deserialize_transfer_basic() {
        let json = "{\"transfers\":[\"basic\"]}";
        let transfer: TestHoldingTransfers = serde_json::from_str(json).unwrap();
        assert_eq!(transfer, TestHoldingTransfers { transfers: Some(vec![Transfer::Basic]) });
    }

    #[test]
    fn test_deserialize_transfer_unknown() {
        let json = "{\"transfers\":[\"foo\"]}";
        let transfer: TestHoldingTransfers = serde_json::from_str(json).unwrap();
        assert_eq!(transfer, TestHoldingTransfers { transfers: Some(vec![Transfer::Unknown]) });
    }

    #[test]
    fn test_deserialize_transfer_none() {
        let json = "{}";
        let transfer: TestHoldingTransfers = serde_json::from_str(json).unwrap();
        assert_eq!(transfer, TestHoldingTransfers { transfers: None });
    }

    #[test]
    fn test_serialize_transfer_basic() {
        let transfers = TestHoldingTransfers { transfers: Some(vec![Transfer::Basic]) };
        let json = serde_json::to_string(&transfers).unwrap();
        assert_eq!(json, "{\"transfers\":[\"basic\"]}");
    }

    #[test]
    fn test_serialize_transfer_unknown() {
        let transfers = TestHoldingTransfers { transfers: Some(vec![Transfer::Unknown]) };
        let json = serde_json::to_string(&transfers).unwrap();
        assert_eq!(json, "{\"transfers\":[\"unknown\"]}");
    }

    #[test]
    fn test_serialize_transfer_none() {
        let transfers = TestHoldingTransfers { transfers: None };
        let json = serde_json::to_string(&transfers).unwrap();
        assert_eq!(json, "{}");
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct TestHoldingHashAlgo {
        #[serde(deserialize_with = "from_hash_algo_string")]
        hash_algo: HashAlgorithm,
    }

    #[test]
    fn test_deserialize_hash_algo_sha256() {
        let json= "{\"hash_algo\":\"sha256\"}";
        let hash_algo: TestHoldingHashAlgo = serde_json::from_str(json).unwrap();
        assert_eq!(hash_algo, TestHoldingHashAlgo { hash_algo: HashAlgorithm::Sha256 });
    }

    #[test]
    fn test_deserialize_hash_algo_unknown() {
        let json= "{\"hash_algo\":\"foo\"}";
        let hash_algo: TestHoldingHashAlgo = serde_json::from_str(json).unwrap();
        assert_eq!(hash_algo, TestHoldingHashAlgo { hash_algo: HashAlgorithm::Unknown });
    }

    #[test]
    fn test_serialize_hash_algo_sha256() {
        let hash_algo = TestHoldingHashAlgo { hash_algo: HashAlgorithm::Sha256 };
        let json = serde_json::to_string(&hash_algo).unwrap();
        assert_eq!(json, "{\"hash_algo\":\"sha256\"}");
    }

    #[test]
    fn test_serialize_hash_algo_unknown() {
        let hash_algo = TestHoldingHashAlgo { hash_algo: HashAlgorithm::Unknown };
        let json = serde_json::to_string(&hash_algo).unwrap();
        assert_eq!(json, "{\"hash_algo\":\"unknown\"}");
    }
}
