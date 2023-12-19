mod common;

use crate::common::ClientHelper;
use axum::http::StatusCode;
use serde_json::Value;

/**
* Rewrite url as the nginx proxy would typically do in a production architecture
*/
fn rewrite(url: &str, repo: &str, custom_signer_host: &str) -> String {
    let host = format!("{}/{}", custom_signer_host, repo);
    format!("{}?repo={}", &url.replace(&host, ""), repo)
}

/**
 * Itegration test for the nominal case
 */
#[tokio::test]
async fn test_batch() {
    let (mut app, config) = ClientHelper::new(vec!["proxy", "fs"]);
    let custom_signer_host = &config.custom_signer_host.unwrap();

    // 1) Try to download a non existent file
    let (status, json) = app
        .post_json(
            "/objects/batch?repo=testing",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZG93bmxvYWQiLCJyZXBvIjoidGVzdGluZyIsInVzZXIiOiJhZG1pbi10ZXN0ZXIifQ.rNfKZOwgCVN-EQj7BA1ef3q2_D-aVM2nofbEdlxPShU",
            "{\"operation\":\"download\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}",
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.unwrap(), "{\"transfer\":\"basic\",\"objects\":[{\"oid\":\"test2.txt\",\"size\":123,\"error\":{\"message\":\"Not found\"}}],\"hash_algo\":\"sha256\"}");

    // 2) Get a link to upload the file, but with the wrong token
    let (status, json) = app
        .post_json(
            "/objects/batch?repo=testing",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZG93bmxvYWQiLCJyZXBvIjoidGVzdGluZyIsInVzZXIiOiJhZG1pbi10ZXN0ZXIifQ.rNfKZOwgCVN-EQj7BA1ef3q2_D-aVM2nofbEdlxPShU",
            "{\"operation\":\"upload\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}",
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(json.unwrap(), "{\"message\":\"Forbidden\"}");

    // 3) Get a link to upload the file
    let (status, json) = app
        .post_json(
            "/objects/batch?repo=testing",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoiYWRtaW4tdGVzdGVyIn0.bFZTK0MdnBlJLLkXXXKmwVMBLHSIMqeBhziVys-PBSI",
            "{\"operation\":\"upload\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}",
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_match!(
        json.as_ref().unwrap(),
        r#"\{"transfer":"basic","objects":\[\{"oid":"test2\.txt","size":123,"actions":\{"upload":\{"href":"https://example.com/testing/objects/access/test2.txt","header":\{"Authorization":"Bearer ([a-zA-Z0-9_\.\-]*)"},"expires_in":3600}}}\],"hash_algo":"sha256"}"#
    );

    // 4) Parse json and get back the link and the token
    let res: Value = serde_json::from_str(&json.unwrap()).unwrap();
    let action = &res["objects"][0]["actions"]["upload"];
    let auth = action["header"]["Authorization"].as_str().unwrap();
    let href = rewrite(
        action["href"].as_str().unwrap(),
        "testing",
        custom_signer_host,
    );

    // 5) Upload the file
    let data = b"test of some data from integration test".to_vec();
    let (status, json) = app.put_data(&href, auth, "custom/my-mime-type", data).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json, None);

    // 6) Get a link to download the file
    let (status, json) = app
        .post_json(
            "/objects/batch?repo=testing",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZG93bmxvYWQiLCJyZXBvIjoidGVzdGluZyIsInVzZXIiOiJhZG1pbi10ZXN0ZXIifQ.rNfKZOwgCVN-EQj7BA1ef3q2_D-aVM2nofbEdlxPShU",
            "{\"operation\":\"download\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}",
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_match!(
        json.as_ref().unwrap(),
        r#"\{"transfer":"basic","objects":\[\{"oid":"test2\.txt","size":123,"actions":\{"download":\{"href":"https://example.com/testing/objects/access/test2.txt","header":\{"Authorization":"Bearer ([a-zA-Z0-9_\.\-]*)"},"expires_in":3600}}}\],"hash_algo":"sha256"}"#
    );

    // 7) reparse the actions
    let res: Value = serde_json::from_str(&json.unwrap()).unwrap();
    let action = &res["objects"][0]["actions"]["download"];
    let href = rewrite(
        action["href"].as_str().unwrap(),
        "testing",
        custom_signer_host,
    );
    let auth = action["header"]["Authorization"].as_str().unwrap();

    // 8) Download the content of the file
    let (status, data, content_type) = app.get_data(&href, auth).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(data.unwrap(), "test of some data from integration test");
    assert_eq!(content_type, Some("custom/my-mime-type".to_string()));
}
