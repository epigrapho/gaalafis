use crate::common::app_utils::ClientHelper;
use crate::common::batch_objects::{
    app_upload_object, assert_batch_download_secret_file_fail, assert_download_secret_file_fail,
    batch_upload, extract_href_auth, UrlRewrite,
};

pub async fn batch_object_proxy_exit_directory_attack(
    mut app: ClientHelper,
    url_rewrite: Box<UrlRewrite>,
) {
    // 1) Upload a first file so the directory exists
    let json = batch_upload(&mut app, r#"(.*)"#).await;
    let (href, auth) = extract_href_auth(&json, "upload");
    let href = url_rewrite(&href, "testing");
    app_upload_object(&mut app, &href, &auth).await;

    // 2) Create a file in a secret location
    tokio::fs::create_dir_all("/tmp/secret").await.unwrap();
    tokio::fs::write("/tmp/secret/my_secret.txt", "secret")
        .await
        .unwrap();

    // 3) Try to batch download the secret file
    assert_batch_download_secret_file_fail(&mut app).await;

    // 4) It shall have been refused by now, but if there were a token, the download itself should fail too
    assert_download_secret_file_fail(&mut app, url_rewrite).await;
}
