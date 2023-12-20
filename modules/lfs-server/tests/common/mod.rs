use lfs_info_server::server::config::ServerConfig;
use s3::creds::Credentials;
use s3::{Bucket, BucketConfiguration, Region};
use tokio_postgres::NoTls;

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

#[macro_export]
macro_rules! assert_response_match {
    ($response_future: expr, $expected_status: expr, $expected_body_pattern: expr) => {{
        let (status, body) = $response_future.await;
        assert_eq!(
            status, $expected_status,
            "Unexpected status code: received {}, expected {}",
            status, $expected_status
        );
        assert_match!(body.as_ref().unwrap(), $expected_body_pattern);
    }};
}

#[macro_export]
macro_rules! assert_response_eq {
    ($response_future: expr, $expected_status: expr, $expected_body: expr) => {{
        let (status, body) = $response_future.await;
        assert_eq!(
            status, $expected_status,
            "Unexpected status code: received {}, expected {}",
            status, $expected_status
        );
        assert_eq!(body.as_ref().unwrap(), $expected_body);
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

pub async fn init_test_database(db_name: &str) {
    // 1) create database
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=1", NoTls)
            .await
            .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    tracing::info!("creating database {}", db_name);
    let stmt = client
        .prepare(format!("CREATE DATABASE \"{}\";", db_name).as_str())
        .await
        .unwrap();
    client.execute(&stmt, &[]).await.unwrap();

    // 2) create table
    let (client, connection) = tokio_postgres::connect(
        format!("host=localhost user=postgres password=1 dbname={}", db_name).as_str(),
        NoTls,
    )
    .await
    .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let stmt = client
        .prepare(
            "CREATE TABLE locks (
                    id SERIAL PRIMARY KEY,
                    path TEXT NOT NULL,
                    ref_name TEXT NOT NULL,
                    repo TEXT NOT NULL,
                    owner TEXT NOT NULL,
                    locked_at TIMESTAMP NOT NULL DEFAULT NOW()
                )",
        )
        .await
        .unwrap();
    client.execute(&stmt, &[]).await.unwrap();
}

/**
 * Rewrite url as the nginx proxy would typically do in a production architecture:
 * Transform "<custom_signer_host>/<repo>/<path>" into "/<path>?repo=<repo>"
 */
pub fn rewrite_url(url: &str, repo: &str, custom_signer_host: &str) -> String {
    let host = format!("{}/{}", custom_signer_host, repo);
    format!("{}?repo={}", &url.replace(&host, ""), repo)
}
