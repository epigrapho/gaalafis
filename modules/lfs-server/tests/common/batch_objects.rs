use crate::common::app_utils::ClientHelper;
use crate::common::http_utils::fetch_url;
use crate::{assert_match, assert_response_eq};
use axum::body::Bytes;
use axum::http::StatusCode;
use http::Method;
use serde_json::Value;

pub type UrlRewrite = dyn Fn(&str, &str) -> String;

pub async fn batch_download_missing(app: &mut ClientHelper) {
    let (status, json) = app
        .post_json(
            "/objects/batch?repo=testing",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZG93bmxvYWQiLCJyZXBvIjoidGVzdGluZyIsInVzZXIiOiJhZG1pbi10ZXN0ZXIifQ.rNfKZOwgCVN-EQj7BA1ef3q2_D-aVM2nofbEdlxPShU",
            "{\"operation\":\"download\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}",
        )
        .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(json.unwrap(), "{\"transfer\":\"basic\",\"objects\":[{\"oid\":\"test2.txt\",\"size\":123,\"error\":{\"message\":\"Not found\"}}],\"hash_algo\":\"sha256\"}");
}

pub async fn batch_upload_wrong_token(app: &mut ClientHelper) {
    let (status, json) = app
        .post_json(
            "/objects/batch?repo=testing",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZG93bmxvYWQiLCJyZXBvIjoidGVzdGluZyIsInVzZXIiOiJhZG1pbi10ZXN0ZXIifQ.rNfKZOwgCVN-EQj7BA1ef3q2_D-aVM2nofbEdlxPShU",
            "{\"operation\":\"upload\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}",
        )
        .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
    assert_eq!(json.unwrap(), "{\"message\":\"Forbidden\"}");
}

pub async fn batch_download(app: &mut ClientHelper, pattern: &str) -> String {
    let (status, json) = app
        .post_json(
            "/objects/batch?repo=testing",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZG93bmxvYWQiLCJyZXBvIjoidGVzdGluZyIsInVzZXIiOiJhZG1pbi10ZXN0ZXIifQ.rNfKZOwgCVN-EQj7BA1ef3q2_D-aVM2nofbEdlxPShU",
            "{\"operation\":\"download\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}",
        )
        .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_match!(json.as_ref().unwrap(), pattern);
    json.unwrap()
}

pub async fn batch_upload(app: &mut ClientHelper, pattern: &str) -> String {
    let (status, json) = app
        .post_json(
            "/objects/batch?repo=testing",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoiYWRtaW4tdGVzdGVyIn0.bFZTK0MdnBlJLLkXXXKmwVMBLHSIMqeBhziVys-PBSI",
            "{\"operation\":\"upload\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}",
        )
        .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_match!(json.as_ref().unwrap(), pattern);
    json.unwrap()
}

pub async fn app_upload_object(app: &mut ClientHelper, href: &str, auth: &str) {
    let data = b"test of some data from integration test".to_vec();
    let (status, json) = app.put_data(href, auth, "custom/my-mime-type", data).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(json, None);
}

pub async fn http_upload_object(href: &str) {
    let data = b"test of some data from integration test".to_vec();
    let (status, data, content_type) = fetch_url(
        href,
        Method::PUT,
        data,
        Some(String::from("custom/my-mime-type")),
    )
    .await
    .unwrap();
    assert_eq!(status, http::StatusCode::OK);
    assert_eq!(data, Bytes::from(""));
    assert_eq!(content_type, None);
}

pub async fn app_download_object(app: &mut ClientHelper, href: &str, auth: &str) {
    let (status, data, content_type) = app.get_data(href, auth).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(data.unwrap(), "test of some data from integration test");
    assert_eq!(content_type, Some("custom/my-mime-type".to_string()));
}

pub async fn http_download_object(href: &str) {
    let (status, data, content_type) = fetch_url(href, Method::GET, vec![], None).await.unwrap();
    assert_eq!(status, http::StatusCode::OK);
    assert_eq!(data, Bytes::from("test of some data from integration test"));
    assert_eq!(content_type, Some("custom/my-mime-type".to_string()));
}

pub fn extract_href_auth(json: &str, action_name: &str) -> (String, String) {
    let res: Value = serde_json::from_str(json).unwrap();
    let action = &res["objects"][0]["actions"][action_name];
    let auth = action["header"]["Authorization"].as_str().unwrap();
    let href = action["href"].as_str().unwrap();
    (href.to_string(), auth.to_string())
}

pub fn extract_href(json: &str, action_name: &str) -> String {
    let res: Value = serde_json::from_str(json).unwrap();
    res["objects"][0]["actions"][action_name]["href"]
        .as_str()
        .unwrap()
        .to_string()
}

pub async fn assert_batch_download_secret_file_fail(app: &mut ClientHelper) {
    let res = app
        .post_json(
            "/objects/batch?repo=testing",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZG93bmxvYWQiLCJyZXBvIjoidGVzdGluZyIsInVzZXIiOiJhZG1pbi10ZXN0ZXIifQ.rNfKZOwgCVN-EQj7BA1ef3q2_D-aVM2nofbEdlxPShU",
            "{\"operation\":\"download\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"../../../secret/my_secret.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}",
        );
    assert_response_eq!(
        res,
        StatusCode::OK,
        "{\"transfer\":\"basic\",\"objects\":[{\"oid\":\"../../../secret/my_secret.txt\",\"size\":123,\"error\":{\"message\":\"Not found\"}}],\"hash_algo\":\"sha256\"}"
    );
}

pub async fn assert_download_secret_file_fail(
    app: &mut ClientHelper,
    url_rewrite: Box<UrlRewrite>,
) {
    let (status, data, _) = app
        .get_data(
            &url_rewrite(
                "https://example.com/testing/objects/access/../../../secret/my_secret.txt",
                "testing",
            ),
            "eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiIxNzAzMjQ4NTM3Iiwib2lkIjoiLi4vLi4vLi4vc2VjcmV0L215X3NlY3JldC50eHQiLCJvcGVyYXRpb24iOiJkb3dubG9hZCIsInJlcG8iOiJ0ZXN0aW5nIn0.ff_245iF21DAtsEirWMx1Gg7-4ConCQp0ckS7APWK9k",
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(data.unwrap(), "{\"message\":\"Not found\"}");
}
