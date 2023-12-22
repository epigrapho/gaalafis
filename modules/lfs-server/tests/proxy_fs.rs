use crate::{
    common::{app_utils::ClientHelper, rewrite_url},
    scenario::{
        batch_object_exit_directory_attack::batch_object_proxy_exit_directory_attack,
        batch_objects_nominal::batch_objects_nominal_proxy,
    },
};

pub mod common;
pub mod scenario;

/**
 * Integration test for the nominal case
 */
#[tokio::test]
async fn test_batch_objects_nominal() {
    let (app, config) = ClientHelper::new(vec!["proxy", "fs"]);
    let custom_signer_host = config.custom_signer_host.unwrap();
    batch_objects_nominal_proxy(
        app,
        Box::new(move |url, repo| rewrite_url(url, repo, &custom_signer_host)),
    )
    .await;
}

/**
 * Integration test for an attack attempting to download objects outside of objects directory
 */
#[tokio::test]
async fn test_batch_object_exit_directory_attack() {
    let (app, config) = ClientHelper::new(vec!["proxy", "fs"]);
    let custom_signer_host = config.custom_signer_host.unwrap();
    batch_object_proxy_exit_directory_attack(
        app,
        Box::new(move |url, repo| rewrite_url(url, repo, &custom_signer_host)),
    )
    .await;
}
