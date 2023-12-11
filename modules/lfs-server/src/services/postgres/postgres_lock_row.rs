use crate::api::locks::response::LockOwner;
use std::fmt;
use std::time::SystemTime;
use tokio_postgres::{row::RowIndex, types::FromSql, Row};

use crate::traits::locks::{Lock, LocksProviderError};

impl Lock {
    fn try_get_from_row<'a, I, T>(row: &'a Row, index: I) -> Result<T, LocksProviderError>
    where
        I: RowIndex + fmt::Display,
        T: FromSql<'a>,
    {
        row.try_get::<'a, I, T>(index)
            .map_err(|e| LocksProviderError::ParsingResponseDataFailure(Box::new(e)))
    }

    pub fn from_row(row: &Row) -> Result<Lock, LocksProviderError> {
        let id: i32 = Self::try_get_from_row(&row, 0)?;
        let path: String = Self::try_get_from_row(&row, 1)?;
        let ref_name: String = Self::try_get_from_row(&row, 2)?;
        let owner: String = Self::try_get_from_row(&row, 3)?;
        let locked_at: SystemTime = Self::try_get_from_row(&row, 4)?;
        Ok(Lock {
            id: id.to_string(),
            path,
            ref_name,
            owner: LockOwner { name: owner },
            locked_at,
        })
    }
}
