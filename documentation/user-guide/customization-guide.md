# Customization guide

This guide will help you to customize your own version of the architecture.

## LFS Server

The core component to chose is the LFS server. GAALAFIS provides out of the box several implementations of it, and you can quite easily implement your own. The docker image introduced in the user guide is tagged `epigrapho/gaalafis-lfs-server:0.3.20-proxy-sbs-locks-pg`:

- `epigrapho/gaalafis-lfs-server`: the name of the image
- `0.3.20`: the version of the image
- `proxy`: where does the files are served? The LFS server generate signed links that are sent back to the client. Theses links can point to the LFS server itself that will proxy requests to the actual bucket (`proxy` variant), or to another location, such as the bucket itself (`signer`)
- `sbs`: the storage backend "Single Bucket Storage". GAALAFIS currently provide a single storage backend, S3 or MinIO (same API), and a single bucket is used for all repos. In the future, we plan to implement other backend: one bucket per repo (`mbs`), direct file storage (`fs`)
- `locks`: the image implement the lock api (opt-in)
- `pg`: the locks are stored in a Postgres database

For now, the available images are

- `epigrapho/gaalafis-lfs-server:*.*.*-proxy-sbs`
- `epigrapho/gaalafis-lfs-server:*.*.*-proxy-sbs-locks-pg`
- `epigrapho/gaalafis-lfs-server:*.*.*-signer-sbs`
- `epigrapho/gaalafis-lfs-server:*.*.*-signer-sbs-locks-pg`

## Configuration

| Env variable             | Description                                                                      | Required for                |
| ------------------------ | -------------------------------------------------------------------------------- | --------------------------- |
| `SBS_BUCKET_NAME`        | The name of the bucket                                                           | All sbs variants            |
| `SBS_HOST`               | The internal endpoint of the storage instance                                    | All sbs variants            |
| `SBS_ACCESS_KEY_FILE`    | The access key of the storage instance                                           | All sbs variants            |
| `SBS_SECRET_KEY_FILE`    | The secret key of the storage instance                                           | All sbs variants            |
| `SBS_REGION`             | The region of the storage instance                                               | All sbs variants            |
| `SBS_PUBLIC_REGION`      | The region of the publicly accessible buckets instance (might be behind proxy)   | Signer sbs variants         |
| `SBS_PUBLIC_HOST`        | The endpoint of the publicly accessible buckets instance (might be behind proxy) | Signer sbs variants         |
| `JWT_SECRET_FILE`        | The secret used to sign the JWT tokens                                           | All                         |
| `JWT_EXPIRES_IN`         | The expiration time of the JWT tokens (in seconds)                               | All                         |
| `DATABASE_HOST`          | The internal endpoint of the postgres database                                   | All postgres locks variants |
| `DATABASE_NAME`          | The name of the postgres database                                                | All postgres locks variants |
| `DATABASE_USER`          | The name of the postgres                                                         | All postgres locks variants |
| `DATABASE_PASSWORD_FILE` | The password of the postgres                                                     | All postgres locks variants |

The distinction between the `SBS_HOST` abd `SBS_PUBLIC_HOST` is due to the fact that the LFS server is not aware of the proxy that might be in front of it. The server can access the bucket directly on the private network to perform file manipulation, but when signing links that will be sent to the client, it must be links accessible from the outside. 

Note that when using the proxy variants, you don't even need to expose the bucket to the outside, as the LFS server will proxy the requests to the bucket.

Similarly, the database should not be accessible from the outside. 

## Going further

If you want to implement your own strategies, the LFS server is implemented in rust, and you have only a few traits to implement, and all dependencies are injected top down:

```rust
// ...imports

// TODO: bundle your implementations in a single structure
pub struct InjectedServices {
    fs: MinioSingleBucketStorage,
    token_encoder_decoder: JwtTokenEncoderDecoder,
    locks_provider: Option<PostgresLocksProvider>,
}

impl InjectedServices {
    pub fn new() -> InjectedServices {
        // TODO: read env and build your implementations
        InjectedServices {
            ...
        }
    }
}

// TODO: choose the wanted implementations
impl Services for InjectedServices {
    fn file_storage_meta_requester(&self) -> &(dyn FileStorageMetaRequester + 'static) {
        &self.fs
    }

    fn file_storage_link_signer(&self) -> &(dyn FileStorageLinkSigner + 'static) {
        &self.fs
    }

    fn token_encoder_decoder(&self) -> &(dyn TokenEncoderDecoder + 'static) {
        &self.token_encoder_decoder
    }

    fn locks_provider(&self) -> Option<&(dyn LocksProvider + 'static)> {
        self.locks_provider
            .as_ref()
            .map(|lp| lp as &(dyn LocksProvider))
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let services: Arc<dyn Services + Send + Sync> = Arc::new(InjectedServices::new());

    // TODO: choose the wanted routes
    let app = Router::new()
        .directory_route("/objects/batch", post(post_objects_batch))
        // ...
        .layer(middleware::from_fn(handle_and_filter_error_details))
        .with_state(services);

    // run...
}
```
