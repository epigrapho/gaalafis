use crate::{
    common::{app_utils::ClientHelper, init_test_bucket},
    scenario::batch_objects_nominal::batch_objects_nominal_signer,
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
