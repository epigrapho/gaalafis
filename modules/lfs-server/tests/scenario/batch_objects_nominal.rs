use crate::assert_match;
use crate::common::app_utils::ClientHelper;
use crate::common::http_utils::fetch_url;
use axum::body::Bytes;
use http::Method;
use serde_json::Value;

type UrlRewrite = dyn Fn(&str, &str) -> String;

async fn batch_download_missing(app: &mut ClientHelper) {
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

async fn batch_upload_wrong_token(app: &mut ClientHelper) {
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

async fn batch_download(app: &mut ClientHelper, pattern: &str) -> String {
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

async fn batch_upload(app: &mut ClientHelper, pattern: &str) -> String {
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

async fn app_upload_object(app: &mut ClientHelper, href: &str, auth: &str) {
    let data = b"test of some data from integration test".to_vec();
    let (status, json) = app.put_data(href, auth, "custom/my-mime-type", data).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(json, None);
}

async fn http_upload_object(href: &str) {
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

async fn app_download_object(app: &mut ClientHelper, href: &str, auth: &str) {
    let (status, data, content_type) = app.get_data(href, auth).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(data.unwrap(), "test of some data from integration test");
    assert_eq!(content_type, Some("custom/my-mime-type".to_string()));
}

async fn http_download_object(href: &str) {
    let (status, data, content_type) = fetch_url(href, Method::GET, vec![], None).await.unwrap();
    assert_eq!(status, http::StatusCode::OK);
    assert_eq!(data, Bytes::from("test of some data from integration test"));
    assert_eq!(content_type, Some("custom/my-mime-type".to_string()));
}

fn extract_href_auth(json: &str, action_name: &str) -> (String, String) {
    let res: Value = serde_json::from_str(json).unwrap();
    let action = &res["objects"][0]["actions"][action_name];
    let auth = action["header"]["Authorization"].as_str().unwrap();
    let href = action["href"].as_str().unwrap();
    (href.to_string(), auth.to_string())
}

fn extract_href(json: &str, action_name: &str) -> String {
    let res: Value = serde_json::from_str(json).unwrap();
    res["objects"][0]["actions"][action_name]["href"]
        .as_str()
        .unwrap()
        .to_string()
}

pub async fn batch_objects_nominal_proxy(mut app: ClientHelper, url_rewrite: Box<UrlRewrite>) {
    // 1) Try to download a non existent file
    batch_download_missing(&mut app).await;

    // 2) Get a link to upload the file, but with the wrong token
    batch_upload_wrong_token(&mut app).await;

    // 3) Get a link to upload the file
    let json = batch_upload(
        &mut app,
        r#"\{"transfer":"basic","objects":\[\{"oid":"test2\.txt","size":123,"actions":\{"upload":\{"href":"https://example.com/testing/objects/access/test2.txt","header":\{"Authorization":"Bearer ([a-zA-Z0-9_\.\-]*)"},"expires_in":3600}}}\],"hash_algo":"sha256"}"#
    ).await;

    // 4) Parse json and get back the link and the token
    let (href, auth) = extract_href_auth(&json, "upload");
    let href = url_rewrite(&href, "testing");

    // 5) Upload the file
    app_upload_object(&mut app, &href, &auth).await;

    // 6) Get a link to download the file
    let json = batch_download(
        &mut app,
        r#"\{"transfer":"basic","objects":\[\{"oid":"test2\.txt","size":123,"actions":\{"download":\{"href":"https://example.com/testing/objects/access/test2.txt","header":\{"Authorization":"Bearer ([a-zA-Z0-9_\.\-]*)"},"expires_in":3600}}}\],"hash_algo":"sha256"}"#,
    ).await;

    // 7) reparse the actions
    let (href, auth) = extract_href_auth(&json, "download");
    let href = url_rewrite(&href, "testing");

    // 8) Download the content of the file
    app_download_object(&mut app, &href, &auth).await;
}

pub async fn batch_objects_nominal_signer(mut app: ClientHelper) {
    // 1) Try to download a non existent file
    batch_download_missing(&mut app).await;

    // 2) Get a link to upload the file, but with the wrong token
    batch_upload_wrong_token(&mut app).await;

    // 3) Get a link to upload the file
    let json = batch_upload(
        &mut app,
        r#"\{"transfer":"basic","objects":\[\{"oid":"test2\.txt","size":123,"actions":\{"upload":\{"href":"http://localhost:9000/(.*)","expires_in":3600}}}\],"hash_algo":"sha256"}"#
    ).await;

    // 4) Parse json and get back the link and the token
    let href = extract_href(&json, "upload");

    // 5) Upload the file
    http_upload_object(&href).await;

    // 6) Get a link to download the file
    let json = batch_download(
        &mut app,
        r#"\{"transfer":"basic","objects":\[\{"oid":"test2\.txt","size":123,"actions":\{"download":\{"href":"http://localhost:9000/(.*)","expires_in":3600}}}\],"hash_algo":"sha256"}"#,
    ).await;

    // 7) reparse the actions
    let href = extract_href(&json, "download");

    // 8) Download the content of the file
    http_download_object(&href).await;
}
