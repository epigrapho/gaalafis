use tokio_postgres::types::ToSql;

pub struct SqlQueryBuilder {
    query: String,
    params: Vec<Box<dyn ToSql + Sync + Send>>,
}

impl Default for SqlQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlQueryBuilder {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            params: Vec::new(),
        }
    }

    pub fn append(&mut self, query: &str) -> &mut Self {
        self.query.push_str(query);
        self
    }

    fn pre_add_param(&mut self, sql: &str) {
        self.query.push_str(sql);
        self.query.push('$');
        self.query.push_str(&(self.params.len() + 1).to_string());
    }

    pub fn add_param_str_string(&mut self, sql: &str, param: &str) -> &mut Self {
        self.pre_add_param(sql);
        self.params.push(Box::new(param.to_string()));
        self
    }

    pub fn add_param_str_i32(
        &mut self,
        sql: &str,
        param: &str,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let number = param.parse::<i32>()?;
        self.pre_add_param(sql);
        self.params.push(Box::new(number));
        Ok(self)
    }

    pub fn add_param_skipable_str_string(
        &mut self,
        sql: &str,
        param: &str,
        skip: bool,
    ) -> &mut Self {
        if !skip {
            self.add_param_str_string(sql, param);
        }
        self
    }

    pub fn add_param_optional_str_string(&mut self, sql: &str, param: Option<&str>) -> &mut Self {
        if let Some(param) = param {
            self.add_param_str_string(sql, param);
        }
        self
    }

    pub fn add_param_optional_str_i32(
        &mut self,
        sql: &str,
        param: Option<&str>,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        if let Some(param) = param {
            self.add_param_str_i32(sql, param)?;
        }
        Ok(self)
    }

    pub fn limit(
        &mut self,
        limit: Option<u64>,
        default: u64,
        overflow: i64,
        min: u64,
        max: u64,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let mut limit = limit.unwrap_or(default);
        if limit < min {
            limit = min
        }
        if limit > max {
            limit = max
        }
        self.pre_add_param(" LIMIT ");
        self.params.push(Box::new(limit as i64 + overflow));
        Ok(self)
    }

    pub fn build(self) -> (String, Vec<Box<dyn ToSql + Sync + Send>>) {
        (self.query, self.params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_query_and_params(
        builder: SqlQueryBuilder,
        expected_query: &str,
        expected_params: Vec<&str>,
    ) {
        let (query, params) = builder.build();
        assert_eq!(query, expected_query);
        assert_eq!(params.len(), expected_params.len());
        for (i, param) in params.iter().enumerate() {
            let param_debug = format!("{:?}", param);
            let expected_param = expected_params[i];
            assert_eq!(expected_param, param_debug);
        }
    }

    #[test]
    fn test_add_param_str_string() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.add_param_str_string("id = ", "1");

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE id = $1",
            vec!["\"1\""],
        );
    }

    #[test]
    fn test_add_param_optional_str_string_some() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.add_param_optional_str_string("id = ", Some("1"));

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE id = $1",
            vec!["\"1\""],
        );
    }

    #[test]
    fn test_add_param_optional_str_string_none() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.add_param_optional_str_string("id = ", None);

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE ",
            vec![],
        );
    }

    #[test]
    fn test_add_param_str_i32() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.add_param_str_i32("id = ", "1").unwrap();

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE id = $1",
            vec!["1"],
        );
    }

    #[test]
    fn test_add_param_str_i32_not_int() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        let result = builder.add_param_str_i32("id = ", "a");
        assert!(result.is_err());
        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE ",
            vec![],
        );
    }

    #[test]
    fn test_add_param_optional_str_i32_some() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.add_param_optional_str_i32("id = ", Some("1")).unwrap();

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE id = $1",
            vec!["1"],
        );
    }

    #[test]
    fn test_add_param_optional_str_i32_none() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.add_param_optional_str_i32("id = ", None).unwrap();

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE ",
            vec![],
        );
    }

    #[test]
    fn test_add_param_skipable_str_string_skip() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.add_param_skipable_str_string("id = ", "1", true);

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE ",
            vec![],
        );
    }

    #[test]
    fn test_add_param_skipable_str_string_not_skip() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.add_param_skipable_str_string("id = ", "1", false);

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE id = $1",
            vec!["\"1\""],
        );
    }

    #[test]
    fn test_limit() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.limit(Some(10), 10, 0, 1, 100).unwrap();

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE  LIMIT $1",
            vec!["10"],
        );
    }

    #[test]
    fn test_limit_overflow() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.limit(Some(10), 10, 1, 1, 100).unwrap();

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE  LIMIT $1",
            vec!["11"],
        );
    }

    #[test]
    fn test_limit_min() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.limit(Some(0), 10, 0, 1, 100).unwrap();

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE  LIMIT $1",
            vec!["1"],
        );
    }

    #[test]
    fn test_limit_max() {
        let mut builder = SqlQueryBuilder::new();
        builder.append("SELECT * FROM table WHERE ");
        builder.limit(Some(1000), 10, 0, 1, 100).unwrap();

        assert_query_and_params(
            builder,
            "SELECT * FROM table WHERE  LIMIT $1",
            vec!["100"],
        );
    }
}
