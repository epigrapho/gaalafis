use crate::config::ServerConfig;
use axum::body::HttpBody;
use axum::routing::{get, post, put, MethodRouter};
use axum::{middleware, Router};
use lfs_info_server::{
    controllers::{
        errors::handle_and_filter_error_details,
        locks::{list_locks, list_locks_for_verification, post_lock, unlock},
        objects::{batch::post_objects_batch, download::download_object, upload::upload_object},
    },
    traits::services::Services,
};
use std::net::SocketAddr;
use std::sync::Arc;

pub trait RouterExt<S, B>
where
    B: HttpBody + Send + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn directory_route(self, path: &str, method_router: MethodRouter<S, B>) -> Self;
}

impl<S, B> RouterExt<S, B> for Router<S, B>
where
    B: HttpBody + Send + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn directory_route(self, path: &str, method_router: MethodRouter<S, B>) -> Self {
        self.route(path, method_router.clone())
            .route(&format!("{path}/"), method_router)
    }
}

pub async fn run_server(config: ServerConfig, services: Arc<dyn Services + Send + Sync + 'static>) {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Create services
    // let s: TServices = TServices::default();

    // Bundle services
    // let services: Arc<dyn Services + Send + Sync + 'static> = Arc::new(s);

    // build our application with a route
    let app = Router::new();

    // Objects module
    //   - `POST /objects/batch?repo=a/b/c`
    let app = app.directory_route("/objects/batch", post(post_objects_batch));

    // Proxy module
    //   - `PUT /objects/access/<oid>?repo=a/b/c`
    //   - `GET /objects/access/<oid>?repo=a/b/c`
    let app = if config.with_proxy {
        app.directory_route("/objects/access/:oid", put(upload_object))
            .directory_route("/objects/access/:oid", get(download_object))
    } else {
        app
    };

    // Locks module
    //   - `POST /locks?repo=abc`
    //   - `GET /locks?repo=abc`
    //   - `POST /locks/:id/unlock?repo=abc`
    //   - `POST /locks/verify?repo=abc`
    let app = if config.with_locks {
        app.directory_route("/locks", post(post_lock))
            .directory_route("/locks", get(list_locks))
            .directory_route("/locks/:id/unlock", post(unlock))
            .directory_route("/locks/verify", post(list_locks_for_verification))
    } else {
        app
    };

    // Error handling and services injection
    let app = app
        .layer(middleware::from_fn(handle_and_filter_error_details))
        .with_state(services);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
