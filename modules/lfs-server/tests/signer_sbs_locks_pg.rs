use crate::{
    common::{app_utils::ClientHelper, init_test_bucket, init_test_database},
    scenario::{batch_objects_nominal::batch_objects_nominal_signer, locks_nominal::locks_nominal},
};

pub mod common;
pub mod scenario;

/**
 * Integration test for the nominal case
 */
#[tokio::test]
async fn test_batch_objects_nominal() {
    let (app, config) = ClientHelper::new(vec!["signer", "sbs", "locks", "pg"]);
    init_test_bucket(&config).await;
    batch_objects_nominal_signer(app).await;
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
