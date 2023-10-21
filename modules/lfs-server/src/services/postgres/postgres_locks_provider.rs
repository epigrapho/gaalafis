use std::vec;

use async_trait::async_trait;
use deadpool_postgres::{
    Config, GenericClient, ManagerConfig, Object, Pool, RecyclingMethod, Runtime,
};
use futures_util::{pin_mut, TryStreamExt};
use tokio_postgres::{types::ToSql, NoTls, Row, RowStream};

use crate::traits::locks::{Lock, LocksProvider, LocksProviderError};

use super::sql_query_builder::SqlQueryBuilder;

pub struct PostgresLocksProvider {
    pool: Pool,
}

impl PostgresLocksProvider {
    pub fn new(host: &str, dbname: &str, username: &str, password: &str) -> Self {
        let mut cfg = Config::new();
        cfg.host = Some(String::from(host));
        cfg.dbname = Some(String::from(dbname));
        cfg.password = Some(String::from(password));
        cfg.user = Some(String::from(username));

        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

        Self { pool }
    }

    fn try_get_env_var(var_name: &str) -> Option<String> {
        let value = std::env::var(var_name).ok();
        if value.is_none() {
            tracing::warn!("Failed to get environment variable {}", var_name);
        }
        value
    }

    fn try_get_file_env_var(var_name: &str) -> Option<String> {
        let file_name = Self::try_get_env_var(var_name)?;
        let value = std::fs::read_to_string(file_name).ok();
        if value.is_none() {
            tracing::warn!("Failed to read file described by environment variable {}", var_name);
        }
        value
    }

    pub fn try_new_from_env_variables(
        host_env_var: &str,
        dbname_env_var: &str,
        username_env_var: &str,
        password_file_env_var: &str,
    ) -> Option<Self> {
        let host = Self::try_get_env_var(host_env_var)?;
        let dbname = Self::try_get_env_var(dbname_env_var)?;
        let username = Self::try_get_env_var(username_env_var)?;
        let password = Self::try_get_file_env_var(password_file_env_var)?;
        Some(PostgresLocksProvider::new(&host, &dbname, &username, &password))
    }

    pub fn new_from_env_variables(
        host_env_var: &str,
        dbname_env_var: &str,
        username_env_var: &str,
        password_file_env_var: &str,
    ) -> Result<Self, String> {
        Self::try_new_from_env_variables(
            host_env_var,
            dbname_env_var,
            username_env_var,
            password_file_env_var,
        )
        .map_or(
            Err(String::from("Failed to start PostgresLocksProvider")),
            Ok,
        )
    }

    async fn get_client(&self) -> Result<Object, LocksProviderError> {
        self.pool
            .get()
            .await
            .map_err(|e| LocksProviderError::ConnectionFailure(Box::new(e)))
    }

    async fn query_raw(
        client: &impl GenericClient,
        sql: String,
        params: Vec<Box<dyn ToSql + Sync + Send>>,
    ) -> Result<RowStream, LocksProviderError> {
        let statement = client
            .prepare(&sql)
            .await
            .map_err(|e| LocksProviderError::RequestPreparationFailure(Box::new(e)))?;
        let params: Vec<&(dyn ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref())
            .map(|item| item as &(dyn ToSql + Sync))
            .collect();
        let res = client
            .query_raw(&statement, params)
            .await
            .map_err(|e| LocksProviderError::RequestExecutionFailure(Box::new(e)))?;
        Ok(res)
    }

    async fn one_row(rows: RowStream) -> Result<Row, LocksProviderError> {
        pin_mut!(rows);
        rows.try_next()
            .await
            .map_err(|_e| LocksProviderError::LockNotFound)?
            .ok_or(LocksProviderError::LockNotFound)
    }

    async fn many_rows(rows: RowStream) -> Result<Vec<Row>, LocksProviderError> {
        rows.try_collect()
            .await
            .map_err(|e| LocksProviderError::RequestExecutionFailure(Box::new(e)))
    }

    async fn query(
        &self,
        sql: String,
        params: Vec<Box<dyn ToSql + Sync + Send>>,
    ) -> Result<Vec<Row>, LocksProviderError> {
        let client = self.get_client().await?;
        let rows = Self::query_raw(&client, sql, params).await?;
        Self::many_rows(rows).await
    }
}

#[async_trait]
impl LocksProvider for PostgresLocksProvider {
    async fn create_lock(
        &self,
        repo: &str,
        user_name: &str,
        path: &str,
        ref_name: Option<&str>,
    ) -> Result<(Lock, bool), LocksProviderError> {
        let mut client = self.get_client().await?;
        let transaction = client
            .transaction()
            .await
            .map_err(|e| LocksProviderError::ConnectionFailure(Box::new(e)))?;

        let stream_get_lock = Self::query_raw(
            &transaction,
            String::from("SELECT id, path, ref_name, owner, locked_at FROM locks WHERE path=$1 AND repo=$2 LIMIT 1"),
            vec![Box::new(path.to_string()), Box::new(repo.to_string())]
        ).await?;

        match Self::one_row(stream_get_lock).await {
            Err(LocksProviderError::LockNotFound) => Ok(()),
            Err(e) => Err(e),
            Ok(lock) => return Ok((Lock::from_row(&lock)?, false)),
        }?;

        let stream = Self::query_raw(
            &transaction,
            String::from("INSERT INTO locks (path, ref_name, repo, owner) VALUES ($1, $2, $3, $4) RETURNING id, path, ref_name, owner, locked_at"),
            vec![
                Box::new(path.to_string()),
                Box::new(ref_name.unwrap_or("NULL").to_string()),
                Box::new(repo.to_string()),
                Box::new(user_name.to_string()),
            ]).await?;
        let created_lock = Self::one_row(stream)
            .await
            .map(|row| Lock::from_row(&row))??;

        transaction
            .commit()
            .await
            .map_err(|e| LocksProviderError::RequestExecutionFailure(Box::new(e)))?;

        Ok((created_lock, true))
    }

    async fn list_locks(
        &self,
        repo: &str,
        path: Option<&str>,
        id: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u64>,
        ref_name: Option<&str>,
    ) -> Result<(Option<String>, Vec<Lock>), LocksProviderError> {
        let mut query = SqlQueryBuilder::new();
        query
            .append("SELECT id, path, ref_name, owner, locked_at FROM locks WHERE ")
            .add_param_str_string("repo = ", repo)
            .add_param_optional_str_string(" AND path = ", path)
            .add_param_optional_str_i32(" AND id = ", id)
            .map_err(|_| LocksProviderError::InvalidId)?
            .add_param_optional_str_string(" AND ref_name = ", ref_name)
            .add_param_optional_str_i32(" AND id >= ", cursor)
            .map_err(|_| LocksProviderError::InvalidCursor)?
            .append(" ORDER BY id ASC")
            .limit(limit, 100, 1, 5, 1000)
            .map_err(|_| LocksProviderError::InvalidLimit)?;

        let (sql, params) = query.build();
        let locks = self
            .query(sql, params)
            .await?
            .iter()
            .map(Lock::from_row)
            .collect::<Result<Vec<Lock>, LocksProviderError>>()?;

        let limit = limit.unwrap_or(100);
        let next_cursor = if locks.len() > limit.try_into().unwrap() {
            Some(locks.last().unwrap().id.clone())
        } else {
            None
        };

        Ok((
            next_cursor,
            locks.into_iter().take(limit.try_into().unwrap()).collect(),
        ))
    }

    async fn delete_lock(
        &self,
        repo: &str,
        user_name: &str,
        id: &str,
        ref_name: Option<&str>,
        force: Option<bool>,
    ) -> Result<Lock, LocksProviderError> {
        let force = force.is_some_and(|f| f);
        let mut client = self.get_client().await?;
        let transaction = client
            .transaction()
            .await
            .map_err(|e| LocksProviderError::ConnectionFailure(Box::new(e)))?;

        let mut query = SqlQueryBuilder::new();
        query
            .append("SELECT id, path, ref_name, owner, locked_at FROM locks WHERE ")
            .add_param_str_string("repo = ", repo)
            .add_param_str_i32(" AND id = ", id)
            .map_err(|_| LocksProviderError::InvalidId)?
            .add_param_optional_str_string(" AND ref_name = ", ref_name);
        let (sql, params) = query.build();
        let stream = Self::query_raw(&transaction, sql, params).await?;
        let lock = Self::one_row(stream).await
            .map(|row| Lock::from_row(&row))??;

        if lock.owner.name != user_name && !force {
            return Err(LocksProviderError::ForceDeleteRequired)
        }

        let mut query = SqlQueryBuilder::new();
        query
            .append("DELETE FROM locks WHERE ")
            .add_param_str_string("repo = ", repo)
            .add_param_str_i32(" AND id = ", id)
            .map_err(|_| LocksProviderError::InvalidId)?
            .add_param_optional_str_string(" AND ref_name = ", ref_name);
        let (sql, params) = query.build();
        Self::query_raw(&transaction, sql, params).await?;
        transaction.commit().await
            .map_err(|e| LocksProviderError::RequestExecutionFailure(Box::new(e)))?;

        Ok(lock)
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    // use crate::traits::locks;
    use rand::{self, Rng};

    use super::*;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    fn random_db_name() -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
        const PASSWORD_LEN: usize = 30;
        let mut rng = rand::thread_rng();

        let password: String = (0..PASSWORD_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        password
    }

    async fn init_test_database() -> (String, PostgresLocksProvider) {
        // take a random string of size 16, containing only letters (no numbers)
        let dbname: String = random_db_name();

        // 1) create database
        let (client, connection) =
            tokio_postgres::connect("host=localhost user=postgres password=1", NoTls)
                .await
                .unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let stmt = client
            .prepare(format!("CREATE DATABASE {};", dbname).as_str())
            .await
            .unwrap();
        client.execute(&stmt, &[]).await.unwrap();

        // 2) create table
        let (client, connection) = tokio_postgres::connect(
            format!("host=localhost user=postgres password=1 dbname={}", dbname).as_str(),
            NoTls,
        )
        .await
        .unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let stmt = client
            .prepare(
                "CREATE TABLE locks (
                    id SERIAL PRIMARY KEY,
                    path TEXT NOT NULL,
                    ref_name TEXT NOT NULL,
                    repo TEXT NOT NULL,
                    owner TEXT NOT NULL,
                    locked_at TIMESTAMP NOT NULL DEFAULT NOW()
                )",
            )
            .await
            .unwrap();
        client.execute(&stmt, &[]).await.unwrap();

        let locks_provider =
            PostgresLocksProvider::new("localhost", &dbname, "postgres", "1");

        (dbname, locks_provider)
    }

    async fn cleanup(dbname: String) {
        let (client, connection) =
            tokio_postgres::connect("host=localhost user=postgres password=1", NoTls)
                .await
                .unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let stmt = client
            .prepare(format!("DROP DATABASE {};", dbname).as_str())
            .await
            .unwrap();
        client.execute(&stmt, &[]).await.unwrap();
    }

    #[test]
    fn test_create_lock() {
        // 1) init db
        let (dbname, locks_provider) = aw!(init_test_database());

        // 2) create lock
        aw!(locks_provider.create_lock("repo", "user", "path", Some("ref_name"))).unwrap();
        let (_, locks) =
            aw!(locks_provider.list_locks("repo", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].path, "path");
        assert_eq!(locks[0].ref_name, "ref_name");
        assert_eq!(locks[0].owner.name, "user");
        let locked_since = SystemTime::now()
            .duration_since(locks[0].locked_at)
            .unwrap();
        assert!(locked_since.as_secs() < 2);

        // 3) cleanup
        aw!(cleanup(dbname));
    }

    #[test]
    fn test_create_duplicate_lock() {
        // 1) init db
        let (dbname, locks_provider) = aw!(init_test_database());

        // 2) create duplicate lock
        let (l1, new1) =
            aw!(locks_provider.create_lock("repo", "user", "path", Some("ref_name"))).unwrap();
        let (l2, new2) =
            aw!(locks_provider.create_lock("repo", "user2", "path", Some("ref_name2"))).unwrap();

        // 3) Only the first one has been created, but the first one is returned in both cases
        assert_eq!(l1.id, l2.id);
        assert_eq!(l1.owner.name, "user");
        assert_eq!(l2.owner.name, "user");
        assert!(new1);
        assert!(!new2);

        // 4) Only the first one has been created in the database
        let (_, locks) =
            aw!(locks_provider.list_locks("repo", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].path, "path");
        assert_eq!(locks[0].ref_name, "ref_name");
        assert_eq!(locks[0].owner.name, "user");

        // 5) cleanup
        aw!(cleanup(dbname));
    }

    #[test]
    fn test_filtering_locks() {
        // 1) init db
        let (dbname, locks_provider) = aw!(init_test_database());

        // 2) create a few locks
        let (l1, _) =
            aw!(locks_provider.create_lock("repo1", "user1", "path1", Some("ref_name1"))).unwrap();
        let (l2, _) =
            aw!(locks_provider.create_lock("repo1", "user1", "path2", Some("ref_name2"))).unwrap();
        let (l3, _) =
            aw!(locks_provider.create_lock("repo1", "user1", "path3", Some("ref_name3"))).unwrap();
        let (l4, _) =
            aw!(locks_provider.create_lock("repo1", "user1", "path4", Some("ref_name4"))).unwrap();
        let (l5, _) =
            aw!(locks_provider.create_lock("repo1", "user1", "path5", Some("ref_name5"))).unwrap();
        let (l6, _) =
            aw!(locks_provider.create_lock("repo1", "user1", "path6", Some("ref_name6"))).unwrap();
        let (l7, _) =
            aw!(locks_provider.create_lock("repo1", "user1", "path7", Some("ref_name7"))).unwrap();
        let (l8, _) =
            aw!(locks_provider.create_lock("repo8", "user8", "path8", Some("ref_name8"))).unwrap();

        // 3) list locks of repo 8
        let (next_cursor, locks) =
            aw!(locks_provider.list_locks("repo8", None, None, None, None, None)).unwrap();
        assert!(next_cursor.is_none());
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].id, l8.id);

        // 4) list locks of path 1
        let (next_cursor, locks) =
            aw!(locks_provider.list_locks("repo1", Some("path1"), None, None, None, None)).unwrap();
        assert!(next_cursor.is_none());
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].id, l1.id);

        // 5) list locks of path 1 and ref_name 1
        let (next_cursor, locks) = aw!(locks_provider.list_locks(
            "repo1",
            Some("path1"),
            None,
            None,
            None,
            Some("ref_name1")
        ))
        .unwrap();
        assert!(next_cursor.is_none());
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].id, l1.id);

        // 6) list locks of repo 1, make use of limit
        let (next_cursor, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, Some(3), None)).unwrap();
        assert_eq!(next_cursor.unwrap(), l4.id);
        assert_eq!(locks.len(), 3);
        assert_eq!(locks[0].id, l1.id);
        assert_eq!(locks[1].id, l2.id);
        assert_eq!(locks[2].id, l3.id);
        let (next_cursor, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, Some(&l4.id), Some(3), None))
                .unwrap();
        assert_eq!(next_cursor.unwrap(), l7.id);
        assert_eq!(locks.len(), 3);
        assert_eq!(locks[0].id, l4.id);
        assert_eq!(locks[1].id, l5.id);
        assert_eq!(locks[2].id, l6.id);
        let (next_cursor, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, Some(&l7.id), Some(3), None))
                .unwrap();
        assert!(next_cursor.is_none());
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].id, l7.id);

        // 7) cleanup
        aw!(cleanup(dbname));
    }

    #[test]
    fn test_delete_lock() {
        // 1) init db
        let (dbname, locks_provider) = aw!(init_test_database());

        // 2) create a few locks
        let (l1, _) =
            aw!(locks_provider.create_lock("repo1", "user1", "path1", Some("ref_name1"))).unwrap();
        let (l2, _) =
            aw!(locks_provider.create_lock("repo1", "user2", "path2", Some("ref_name2"))).unwrap();
        let (l3, _) =
            aw!(locks_provider.create_lock("repo1", "user1", "path3", Some("ref_name3"))).unwrap();

        // 3) delete own user lock
        let deleted =
            aw!(locks_provider.delete_lock("repo1", "user1", &l1.id, None, None)).unwrap();
        assert_eq!(deleted.id, l1.id);
        let (_, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 2);

        // 4) delete other user lock
        let deleted_lock = aw!(locks_provider.delete_lock("repo1", "user1", &l2.id, None, None));
        assert!(deleted_lock.is_err());
        let (_, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 2);

        // 5) delete other user lock with force
        aw!(locks_provider.delete_lock("repo1", "user1", &l2.id, None, Some(true))).unwrap();
        let (_, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 1);

        // 6) delete lock with wrong ref_name
        let deleted_lock =
            aw!(locks_provider.delete_lock("repo1", "user1", &l3.id, Some("ref_name2"), None));
        assert!(deleted_lock.is_err());
        let (_, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 1);

        // 7) delete lock with correct ref_name
        aw!(locks_provider.delete_lock("repo1", "user1", &l3.id, Some("ref_name3"), None)).unwrap();
        let (_, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 0);

        // 8) cleanup
        aw!(cleanup(dbname));
    }
}
