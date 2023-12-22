use crate::{
    common::{app_utils::ClientHelper, init_test_bucket, init_test_database},
    scenario::{
        batch_object_exit_directory_attack::batch_object_signer_exit_directory_attack,
        batch_objects_nominal::batch_objects_nominal_signer, locks_nominal::locks_nominal,
    },
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

/**
 * Integration test for an attack attempting to download objects outside of objects directory
 */
#[tokio::test]
async fn test_batch_object_exit_directory_attack() {
    let (app, config) = ClientHelper::new(vec!["signer", "sbs"]);
    init_test_bucket(&config).await;
    batch_object_signer_exit_directory_attack(app).await;
}
