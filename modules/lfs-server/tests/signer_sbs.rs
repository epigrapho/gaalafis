use crate::{
    common::{app_utils::ClientHelper, init_test_bucket},
    scenario::{
        batch_object_exit_directory_attack::batch_object_signer_exit_directory_attack,
        batch_objects_nominal::batch_objects_nominal_signer,
    },
};

pub mod common;
pub mod scenario;

/**
 * Integration test for the nominal case
 */
#[tokio::test]
async fn test_batch_objects_nominal() {
    let (app, config) = ClientHelper::new(vec!["signer", "sbs"]);
    init_test_bucket(&config).await;
    batch_objects_nominal_signer(app).await;
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
