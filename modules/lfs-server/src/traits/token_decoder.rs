use std::collections::BTreeMap;

pub trait TokenDecoder {
    fn decode_token(&self, token: &str) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>>;
}
