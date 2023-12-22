use crate::common::app_utils::ClientHelper;
use crate::common::batch_objects::{
    app_upload_object, assert_batch_download_secret_file_fail, assert_download_secret_file_fail,
    http_full_upload, UrlRewrite,
};

pub async fn batch_object_proxy_exit_directory_attack(
    mut app: ClientHelper,
    url_rewrite: Box<UrlRewrite>,
) {
    // 1) Upload a first file "test2.txt" to repo "testing"
    let auth = "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib2lkIjoidGVzdDIudHh0Iiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmcifQ.tQcY2kU9XqL5sACoEaVM7j3B1HoYg44vqR1eWHAL_oU";
    app_upload_object(&mut app, "/objects/access/test2.txt?repo=testing", auth).await;

    // 2) Upload a second file "secret.txt" to repo "secret"
    let auth = "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib2lkIjoic2VjcmV0LnR4dCIsIm9wZXJhdGlvbiI6InVwbG9hZCIsInJlcG8iOiJzZWNyZXQifQ.JzsocNgP4nPTsHsQbS5lTUXNmMmkK8jArd9pWoCBBe4";
    app_upload_object(&mut app, "/objects/access/secret.txt?repo=secret", auth).await;

    // 3) Try to batch download the secret file
    assert_batch_download_secret_file_fail(&mut app).await;

    // 4) It shall have been refused by now, but if there were a token, the download itself should fail too
    assert_download_secret_file_fail(&mut app, url_rewrite).await;
}

pub async fn batch_object_signer_exit_directory_attack(mut app: ClientHelper) {
    // 1) Upload a first file "test2.txt" to repo "testing"
    let auth = "eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoiYWRtaW4tdGVzdGVyIn0.bFZTK0MdnBlJLLkXXXKmwVMBLHSIMqeBhziVys-PBSI";
    http_full_upload(&mut app, "testing", "test2.txt", auth).await;

    // 2) Upload a second file "secret.txt" to repo "secret"
    let auth = "eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InNlY3JldCIsInVzZXIiOiJhZG1pbi10ZXN0ZXIifQ.ijtHr6f1QHhXP7z1kVS7zy1AH6---2OGvNzHL_czI0k";
    http_full_upload(&mut app, "secret", "secret.txt", auth).await;

    // 3) Try to batch download the secret file
    assert_batch_download_secret_file_fail(&mut app).await;
}
