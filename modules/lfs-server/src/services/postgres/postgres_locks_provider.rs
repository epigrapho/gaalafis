use std::vec;

use async_trait::async_trait;
use deadpool_postgres::{Config, ManagerConfig, Object, Pool, RecyclingMethod, Runtime};
use tokio_postgres::{types::ToSql, NoTls, Row};

use crate::traits::locks::{Lock, LocksProvider, LocksProviderError};

use super::sql_query_builder::SqlQueryBuilder;

pub struct PostgresLocksProvider {
    pool: Pool,
}

impl PostgresLocksProvider {
    pub async fn new(host: &str, dbname: &str, username: &str, password: &str) -> Self {
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

    pub async fn new_from_env_variables(
        host_env_var: &str,
        dbname_env_var: &str,
        username_env_var: &str,
        password_file_env_var: &str,
    ) -> Self {
        let host = std::env::var(host_env_var).unwrap();
        let dbname = std::env::var(dbname_env_var).unwrap();
        let username = std::env::var(username_env_var).unwrap();
        let password_file = std::env::var(password_file_env_var).unwrap();
        let password = std::fs::read_to_string(password_file).unwrap();
        let locks_provider = PostgresLocksProvider::new(&host, &dbname, &username, &password).await;
        locks_provider
    }

    pub async fn get_client(&self) -> Result<Object, LocksProviderError> {
        self.pool
            .get()
            .await
            .map_err(|e| LocksProviderError::ConnectionFailure(Box::new(e)))
    }

    async fn query_one(
        &self,
        sql: String,
        params: Vec<Box<dyn ToSql + Sync + Send>>,
    ) -> Result<Row, LocksProviderError> {
        let client = self.get_client().await?;
        let params: Vec<&(dyn ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref())
            .map(|item| item as &(dyn ToSql + Sync))
            .collect();
        let stmt = client
            .prepare(&sql)
            .await
            .map_err(|e| LocksProviderError::RequestPreparationFailure(Box::new(e)))?;
        let row = client
            .query_one(&stmt, &params)
            .await
            .map_err(|e| LocksProviderError::RequestExecutionFailure(Box::new(e)))?;
        Ok(row)
    }

    async fn query(
        &self,
        sql: String,
        params: Vec<Box<dyn ToSql + Sync + Send>>,
    ) -> Result<Vec<Row>, LocksProviderError> {
        let client = self.get_client().await?;
        let params: Vec<&(dyn ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref())
            .map(|item| item as &(dyn ToSql + Sync))
            .collect();
        let stmt = client
            .prepare(&sql)
            .await
            .map_err(|e| LocksProviderError::RequestPreparationFailure(Box::new(e)))?;
        let rows = client
            .query(&stmt, &params)
            .await
            .map_err(|e| LocksProviderError::RequestExecutionFailure(Box::new(e)))?;
        Ok(rows)
    }
}

#[async_trait]
impl LocksProvider for PostgresLocksProvider {
    async fn create_lock(
        &self,
        repo: &str,
        user_name: &str,
        path: &str,
        ref_name: &str,
    ) -> Result<String, LocksProviderError> {
        let id = self.query_one(
                String::from("INSERT INTO locks (path, ref_name, repo, owner) VALUES ($1, $2, $3, $4) RETURNING id"),
                vec![
                    Box::new(path.to_string()),
                    Box::new(ref_name.to_string()),
                    Box::new(repo.to_string()),
                    Box::new(user_name.to_string()),
                ],
            ).await?
            .try_get::<_, i32>(0)
            .map_err(|e| LocksProviderError::ParsingResponseDataFailure(Box::new(e)))?
            .to_string();

        Ok(id)
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
            .limit(limit, 100, 1)
            .map_err(|_| LocksProviderError::InvalidLimit)?;

        let (sql, params) = query.build();
        let locks = self
            .query(sql, params)
            .await?
            .iter()
            .map(|row| Lock::from_row(row))
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
        let mut query = SqlQueryBuilder::new();

        query
            .append("DELETE FROM locks WHERE ")
            .add_param_str_string("repo = ", repo)
            .add_param_str_i32(" AND id = ", id)
            .map_err(|_| LocksProviderError::InvalidId)?
            .add_param_optional_str_string(" AND ref_name = ", ref_name)
            .add_param_skipable_str_string(" AND owner = ", user_name, force.is_some_and(|f| f))
            .append(" RETURNING id, path, ref_name, owner, locked_at");

        let (sql, params) = query.build();
        let row = self
            .query_one(sql, params)
            .await
            .map(|row| Lock::from_row(&row))??;

        Ok(row)
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

    async fn init_test_database() -> String {
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
        return dbname;
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
        let dbname = aw!(init_test_database());
        let locks_provider = aw!(PostgresLocksProvider::new(
            "localhost",
            &dbname,
            "postgres",
            "1"
        ));

        // 2) create lock
        aw!(locks_provider.create_lock("repo", "user", "path", "ref_name")).unwrap();
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
    fn test_filtering_locks() {
        // 1) init db
        let dbname = aw!(init_test_database());
        let locks_provider = aw!(PostgresLocksProvider::new(
            "localhost",
            &dbname,
            "postgres",
            "1"
        ));

        // 2) create a few locks
        let id1 = aw!(locks_provider.create_lock("repo1", "user1", "path1", "ref_name1")).unwrap();
        let id2 = aw!(locks_provider.create_lock("repo1", "user1", "path1", "ref_name2")).unwrap();
        let id3 = aw!(locks_provider.create_lock("repo1", "user1", "path3", "ref_name3")).unwrap();
        let id4 = aw!(locks_provider.create_lock("repo1", "user1", "path4", "ref_name4")).unwrap();
        let id5 = aw!(locks_provider.create_lock("repo1", "user1", "path5", "ref_name5")).unwrap();
        let id6 = aw!(locks_provider.create_lock("repo1", "user1", "path6", "ref_name6")).unwrap();
        let id7 = aw!(locks_provider.create_lock("repo1", "user1", "path7", "ref_name7")).unwrap();
        let id8 = aw!(locks_provider.create_lock("repo8", "user8", "path8", "ref_name8")).unwrap();

        // 3) list locks of repo 8
        let (next_cursor, locks) =
            aw!(locks_provider.list_locks("repo8", None, None, None, None, None)).unwrap();
        assert!(next_cursor.is_none());
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].id, id8);

        // 4) list locks of path 1
        let (next_cursor, locks) =
            aw!(locks_provider.list_locks("repo1", Some("path1"), None, None, None, None)).unwrap();
        assert!(next_cursor.is_none());
        assert_eq!(locks.len(), 2);
        assert_eq!(locks[0].id, id1);
        assert_eq!(locks[1].id, id2);

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
        assert_eq!(locks[0].id, id1);

        // 6) list locks of repo 1, make use of limit
        let (next_cursor, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, Some(3), None)).unwrap();
        assert_eq!(next_cursor.unwrap(), id4);
        assert_eq!(locks.len(), 3);
        assert_eq!(locks[0].id, id1);
        assert_eq!(locks[1].id, id2);
        assert_eq!(locks[2].id, id3);
        let (next_cursor, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, Some(&id4), Some(3), None)).unwrap();
        assert_eq!(next_cursor.unwrap(), id7);
        assert_eq!(locks.len(), 3);
        assert_eq!(locks[0].id, id4);
        assert_eq!(locks[1].id, id5);
        assert_eq!(locks[2].id, id6);
        let (next_cursor, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, Some(&id7), Some(3), None)).unwrap();
        assert!(next_cursor.is_none());
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].id, id7);

        // 7) cleanup
        aw!(cleanup(dbname));
    }

    #[test]
    fn test_delete_lock() {
        // 1) init db
        let dbname: String = aw!(init_test_database());
        let locks_provider = aw!(PostgresLocksProvider::new(
            "localhost",
            &dbname,
            "postgres",
            "1"
        ));

        // 2) create a few locks
        let id1 = aw!(locks_provider.create_lock("repo1", "user1", "path1", "ref_name1")).unwrap();
        let id2 = aw!(locks_provider.create_lock("repo1", "user2", "path1", "ref_name2")).unwrap();
        let id3 = aw!(locks_provider.create_lock("repo1", "user1", "path3", "ref_name3")).unwrap();

        // 3) delete own user lock
        let deleted = aw!(locks_provider.delete_lock("repo1", "user1", &id1, None, None)).unwrap();
        assert_eq!(deleted.id, id1);
        let (_, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 2);

        // 4) delete other user lock
        let deleted_lock = aw!(locks_provider.delete_lock("repo1", "user1", &id2, None, None));
        assert!(deleted_lock.is_err());
        let (_, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 2);

        // 5) delete other user lock with force
        aw!(locks_provider.delete_lock("repo1", "user1", &id2, None, Some(true))).unwrap();
        let (_, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 1);

        // 6) delete lock with wrong ref_name
        let deleted_lock =
            aw!(locks_provider.delete_lock("repo1", "user1", &id3, Some("ref_name2"), None));
        assert!(deleted_lock.is_err());
        let (_, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 1);

        // 7) delete lock with correct ref_name
        aw!(locks_provider.delete_lock("repo1", "user1", &id3, Some("ref_name3"), None)).unwrap();
        let (_, locks) =
            aw!(locks_provider.list_locks("repo1", None, None, None, None, None)).unwrap();
        assert_eq!(locks.len(), 0);

        // 8) cleanup
        aw!(cleanup(dbname));
    }
}
