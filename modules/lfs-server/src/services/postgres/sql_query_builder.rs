use tokio_postgres::types::ToSql;

pub struct SqlQueryBuilder {
    query: String,
    params: Vec<Box<dyn ToSql + Sync + Send>>,
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
        self.query.push_str("$");
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
        self.pre_add_param(sql);
        self.params.push(Box::new(param.parse::<i32>()?));
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
