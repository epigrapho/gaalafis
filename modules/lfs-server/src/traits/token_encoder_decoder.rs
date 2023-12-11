use std::collections::BTreeMap;

pub trait TokenEncoderDecoder: Sync + Send {
    fn encode_token(
        &self,
        claims: &mut BTreeMap<&str, String>,
    ) -> Result<String, Box<dyn std::error::Error>>;
    fn decode_token(
        &self,
        token: &str,
    ) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>>;
}
