use axum::{
    body::{Body, HttpBody},
    http::{Method, Request, StatusCode},
    Router,
};
use lfs_info_server::server::{
    config::{FileStorageImplementation, LocksImplementation, ServerConfig},
    injected_services::from_server_config,
    run_server::run_server,
};
use std::sync::Arc;
use tower::{Service, ServiceExt};

fn get_default_server_env(args: Vec<&str>) -> (String, String, ServerConfig) {
    let db_id = uuid::Uuid::new_v4().to_string();
    let bucket_id = uuid::Uuid::new_v4().to_string();
    let fs_root = uuid::Uuid::new_v4().to_string();
    let config = ServerConfig {
        with_locks: false,
        with_proxy: false,
        file_storage_implementation: FileStorageImplementation::LocalFileStorage,
        locks_implementation: LocksImplementation::PostgresLocksProvider,
        fs_root_path: Some(format!("/tmp/IT-{}", fs_root)),
        sbs_bucket_name: Some(bucket_id.clone()),
        sbs_access_key: Some(String::from("minio_access_key")),
        sbs_secret_key: Some(String::from("minio_secret_key")),
        sbs_region: Some(String::from("us-east-1")),
        sbs_host: Some(String::from("http://localhost:9000")),
        sbs_public_region: Some(String::from("us-east-1")),
        sbs_public_host: Some(String::from("http://localhost:9000")),
        jwt_secret: Some(String::from("secret")),
        jwt_expires_in: Some(3600),
        custom_signer_host: Some(String::from("https://example.com")),
        custom_signer_secret: Some(String::from("secret")),
        custom_signer_expires_in: Some(3600),
        database_host: Some(String::from("localhost")),
        database_name: Some(db_id.clone()),
        database_user: Some(String::from("postgres")),
        database_password: Some(String::from("1")),
    };
    (
        db_id,
        bucket_id,
        config.parse_args(args.iter().map(|s| s.to_string()).collect()),
    )
}

pub struct ClientHelper {
    app: Router,
}

impl ClientHelper {
    pub fn new(args: Vec<&str>) -> (ClientHelper, ServerConfig) {
        let (_, _, config) = get_default_server_env(args);
        let services = from_server_config(&config);
        let app = run_server(&config, Arc::new(services));
        (ClientHelper { app }, config)
    }

    async fn send(
        &mut self,
        method: Method,
        uri: &str,
        auth_header_value: &str,
        content_type: &str,
        body: Body,
    ) -> (StatusCode, Option<Vec<u8>>, Option<String>) {
        let res = self
            .app
            .ready()
            .await
            .unwrap()
            .call(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("Content-Type", content_type)
                    .header("Authorization", auth_header_value)
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = res.status();
        let headers = res.headers();
        let content_type = headers
            .get("Content-Type")
            .map(|v| v.to_str().unwrap().to_string());
        let bytes = res.into_body().data().await.map(|b| b.unwrap().to_vec());

        (status, bytes, content_type)
    }

    pub async fn send_json(
        &mut self,
        method: Method,
        uri: &str,
        auth_header_value: &str,
        json_body: &str,
    ) -> (StatusCode, Option<String>) {
        let (status, body, _) = self
            .send(
                method,
                uri,
                auth_header_value,
                "application/json",
                Body::from(String::from(json_body)),
            )
            .await;

        (status, body.map(|b| String::from_utf8(b).unwrap()))
    }

    pub async fn post_json(
        &mut self,
        uri: &str,
        auth_header_value: &str,
        json_body: &str,
    ) -> (StatusCode, Option<String>) {
        self.send_json(Method::POST, uri, auth_header_value, json_body)
            .await
    }

    pub async fn get_json(
        &mut self,
        uri: &str,
        auth_header_value: &str,
    ) -> (StatusCode, Option<String>) {
        self.send_json(Method::GET, uri, auth_header_value, "")
            .await
    }

    pub async fn put_data(
        &mut self,
        uri: &str,
        auth_header_value: &str,
        content_type: &str,
        raw_data: Vec<u8>,
    ) -> (StatusCode, Option<String>) {
        let (status, body, _) = self
            .send(
                Method::PUT,
                uri,
                auth_header_value,
                content_type,
                Body::from(raw_data),
            )
            .await;

        (status, body.map(|b| String::from_utf8(b).unwrap()))
    }

    pub async fn get_data(
        &mut self,
        uri: &str,
        auth_header_value: &str,
    ) -> (StatusCode, Option<String>, Option<String>) {
        let (status, body, content_type) = self
            .send(
                Method::GET,
                uri,
                auth_header_value,
                "application/octet-stream",
                Body::from(vec![]),
            )
            .await;

        (
            status,
            body.map(|b| String::from_utf8(b).unwrap()),
            content_type,
        )
    }

    pub async fn lock(
        &mut self,
        repo: &str,
        path: &str,
        ref_name: &str,
        auth: &str,
    ) -> (StatusCode, Option<String>) {
        self.send_json(
            Method::POST,
            &format!("/locks?repo={}", repo),
            auth,
            &format!(
                "{{\"path\":\"{}\",\"ref\":{{\"name\":\"{}\"}}}}",
                path, ref_name
            ),
        )
        .await
    }
}
