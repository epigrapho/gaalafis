pub mod api {
    pub mod repo_query;
    pub mod objects_batch {
        pub mod body;
        pub mod response;
    }
    pub mod locks {
        pub mod body;
        pub mod response;
    }
    pub mod enums;
    pub mod jwt;
}

pub mod controllers {
    pub mod errors;
    pub mod objects {
        pub mod batch;
        pub mod download;
        pub mod upload;
    }
    pub mod locks;
}

pub mod services {
    pub mod fs {
        pub mod local_file_storage;
    }
    pub mod minio {
        pub mod single_bucket_storage;
    }
    pub mod postgres {
        pub mod postgres_lock_row;
        pub mod postgres_locks_provider;
        pub mod sql_query_builder;
    }
    pub mod custom_link_signer;
    pub mod jwt;
    pub mod jwt_token_encoder_decoder;
}

pub mod traits {
    pub mod file_storage;
    pub mod locks;
    pub mod services;
    pub mod token_encoder_decoder;
}
