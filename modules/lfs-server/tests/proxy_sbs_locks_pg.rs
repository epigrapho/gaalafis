use crate::{
    common::{app_utils::ClientHelper, init_test_bucket, init_test_database, rewrite_url},
    scenario::{batch_objects_nominal::batch_objects_nominal_proxy, locks_nominal::locks_nominal},
};

pub mod common;
pub mod scenario;

/**
 * Integration test for the nominal case
 */
#[tokio::test]
async fn test_batch_objects_nominal() {
    let (app, config) = ClientHelper::new(vec!["proxy", "sbs", "locks", "pg"]);
    init_test_bucket(&config).await;
    let custom_signer_host = config.custom_signer_host.unwrap();
    batch_objects_nominal_proxy(
        app,
        Box::new(move |url, repo| rewrite_url(url, repo, &custom_signer_host)),
    )
    .await;
}

/**
 * Integration test for the nominal case of locks
 */
#[tokio::test]
async fn test_locks_nominal() {
    let (app, config) = ClientHelper::new(vec!["proxy", "fs", "locks", "pg"]);
    init_test_database(&config.database_name.unwrap()).await;
    locks_nominal(app).await;
}
