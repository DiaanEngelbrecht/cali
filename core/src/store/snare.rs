use sqlx::Executor;

pub trait DBConnection<'c>: Executor<'c, Database = sqlx::MySql> {}
impl<'c, T: Executor<'c, Database = sqlx::MySql>> DBConnection<'c> for T {}

pub struct Snare<T> {
    pub query: String,
    pub table_name: String,
    pub data: T,
}

pub trait Ensnared {
    fn insert<'a>(
        &'a mut self,
    ) -> sqlx::query::Query<
        '_,
        sqlx::MySql,
        <sqlx::MySql as sqlx::database::HasArguments<'_>>::Arguments,
    >;
}

pub trait Ensnarable {
    fn insert_parts(&self) -> (String, String);

    fn capture<'a>(
        &'a self,
        query: sqlx::query::Query<
            'a,
            sqlx::MySql,
            <sqlx::MySql as sqlx::database::HasArguments<'_>>::Arguments,
        >,
    ) -> sqlx::query::Query<
        'a,
        sqlx::MySql,
        <sqlx::MySql as sqlx::database::HasArguments<'_>>::Arguments,
    >;
} 

impl<T: Ensnarable> Ensnared for Snare<T> {
    fn insert<'a>(
        &'a mut self,
    ) -> sqlx::query::Query<
        '_,
        sqlx::MySql,
        <sqlx::MySql as sqlx::database::HasArguments<'_>>::Arguments,
    > {
        let (values, bindings) = self.data.insert_parts();
        self.query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table_name, values, bindings
        );

        self.data.capture(sqlx::query(&self.query))
    }
}
