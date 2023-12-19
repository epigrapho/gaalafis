use crate::{
    controllers::{
        errors::handle_and_filter_error_details,
        locks::{list_locks, list_locks_for_verification, post_lock, unlock},
        objects::{batch::post_objects_batch, download::download_object, upload::upload_object},
    },
    server::config::ServerConfig,
    traits::services::Services,
};
use axum::{
    body::HttpBody,
    middleware,
    routing::{get, post, put, MethodRouter},
    Router,
};
use std::sync::Arc;

/**
 * Extension trait to allow for adding both `/path` and `/path/` routes at the same time with
 * the same handler.
 */
pub trait RouterExt<S, B>
where
    B: HttpBody + Send + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn directory_route(self, path: &str, method_router: MethodRouter<S, B>) -> Self;
}

/**
 * Implementation of the extension trait to allow for adding both `/path` and `/path/` routes
 * at the same time with the same handler..
 */
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

/**
 * Run the server with the given configuration and services implementation.
 */
pub fn run_server(
    config: &ServerConfig,
    services: Arc<dyn Services + Send + Sync + 'static>,
) -> Router<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

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
    app.layer(middleware::from_fn(handle_and_filter_error_details))
        .with_state(services)
}
