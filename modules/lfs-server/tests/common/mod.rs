use lfs_info_server::server::config::ServerConfig;
use s3::creds::Credentials;
use s3::{Bucket, BucketConfiguration, Region};

pub mod app_utils;
pub mod http_utils;

#[macro_export]
macro_rules! assert_match {
    ($value:expr, $pattern:expr) => {{
        let regex = regex::Regex::new($pattern).expect("Invalid regex pattern");
        assert!(
            regex.is_match($value),
            "Assertion failed: '{}' does not match pattern '{}'",
            $value,
            $pattern
        );
    }};
}

pub async fn init_test_bucket(config: &ServerConfig) {
    let region = Region::Custom {
        region: config.sbs_region.clone().unwrap(),
        endpoint: config.sbs_host.clone().unwrap(),
    };
    let credentials = Credentials::new(
        Some(&config.sbs_access_key.clone().unwrap()),
        Some(&config.sbs_secret_key.clone().unwrap()),
        None,
        None,
        None,
    )
    .unwrap();
    Bucket::create_with_path_style(
        &config.sbs_bucket_name.clone().unwrap(),
        region.clone(),
        credentials.clone(),
        BucketConfiguration::private(),
    )
    .await
    .unwrap();
}

/**
 * Rewrite url as the nginx proxy would typically do in a production architecture:
 * Transform "<custom_signer_host>/<repo>/<path>" into "/<path>?repo=<repo>"
 */
pub fn rewrite_url(url: &str, repo: &str, custom_signer_host: &str) -> String {
    let host = format!("{}/{}", custom_signer_host, repo);
    format!("{}?repo={}", &url.replace(&host, ""), repo)
}
