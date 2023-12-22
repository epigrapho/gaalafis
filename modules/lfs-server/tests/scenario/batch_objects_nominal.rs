use crate::common::{
    app_utils::ClientHelper,
    batch_objects::{
        app_download_object, app_upload_object, batch_download, batch_download_missing,
        batch_upload, batch_upload_wrong_token, extract_href, extract_href_auth,
        http_download_object, http_upload_object, UrlRewrite,
    },
};

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
