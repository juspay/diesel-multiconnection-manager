extern crate derive_more;
use derive_more::{Deref, DerefMut, Display};
use std::collections::HashMap;

use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    MysqlConnection, PgConnection, SqliteConnection,
};

#[derive(Debug, Display)]
pub enum DatabaseKind {
    Postgres,
    MySQL,
    SQLite,
}

type GenericPool<M> = Pool<ConnectionManager<M>>;
type GenericConnection<M> = PooledConnection<ConnectionManager<M>>;
type ResultConnection<M> = anyhow::Result<GenericConnection<M>>;

#[derive(Clone, Debug)]
pub enum MultiConnectionPool {
    Pg(GenericPool<PgConnection>),
    Mysql(GenericPool<MysqlConnection>),
    Sqlite(GenericPool<SqliteConnection>),
}

#[derive(Debug, Display)]
#[display(
    fmt = "connection config\nname : {}\ndatabase engine : {}\nURL: {}\nschema: {}\nconnection count: {}",
    connection_name,
    database,
    database_host_url,
    schema,
    connection_count
)]
pub struct ConnectionConfig {
    connection_name: String,
    database: DatabaseKind,
    database_name: String,
    database_host_url: String,
    schema: String,
    connection_count: u32,
    options: Option<String>,
}

impl ConnectionConfig {
    pub fn new(
        connection_name: String,
        database: DatabaseKind,
        database_name: String,
        database_host_url: String,
        schema: String,
        connection_count: u32,
        options: Option<String>,
    ) -> Self {
        ConnectionConfig {
            connection_name,
            database,
            database_name,
            database_host_url,
            schema,
            connection_count,
            options,
        }
    }

    pub fn conn_url(&self) -> String {
        match self.database {
            DatabaseKind::Postgres => self.pg_conn_url(),
            DatabaseKind::MySQL => self.mysql_conn_url(),
            DatabaseKind::SQLite => self.sqlite_conn_url(),
        }
    }

    fn pg_conn_url(&self) -> String {
        if let Some(configs) = &self.options {
            format!(
                "{}/{}?{}&options=-c%20search_path%3D{},$user,public",
                self.database_host_url, self.database_name, configs, self.schema
            )
        } else {
            format!(
                "{}/{}?options=-c%20search_path%3D{},$user,public",
                self.database_host_url, self.database_name, self.schema
            )
        }
    }

    fn mysql_conn_url(&self) -> String {
        // mysql considers schema and database name the same
        if let Some(configs) = &self.options {
            format!(
                "{}/{}?{}",
                self.database_host_url, self.database_name, configs
            )
        } else {
            format!("{}/{}", self.database_host_url, self.database_name)
        }
    }

    fn sqlite_conn_url(&self) -> String {
        // provide a file name, file URI or :memory:
        if self.database_host_url.eq(":memory:") {
            return self.database_host_url.clone();
        }
        format!("{}{}", self.database_host_url, self.database_name)
    }
}

#[derive(Deref, DerefMut, Clone)]
pub struct PgSchemaManager(HashMap<String, MultiConnectionPool>);

impl<const N: usize> From<[ConnectionConfig; N]> for PgSchemaManager {
    fn from(value: [ConnectionConfig; N]) -> Self {
        let mut schema_manager: PgSchemaManager = PgSchemaManager(HashMap::new());
        for config in value.into_iter() {
            let pool: MultiConnectionPool = match config.database {
                DatabaseKind::Postgres => {
                    let manager = ConnectionManager::<PgConnection>::new(config.conn_url());
                    MultiConnectionPool::Pg(
                        Pool::builder()
                            .max_size(config.connection_count)
                            .build(manager)
                            .expect(
                                format!("Invalid config provided, {}", config.connection_name)
                                    .as_str(),
                            ),
                    )
                }
                DatabaseKind::MySQL => {
                    let manager = ConnectionManager::<MysqlConnection>::new(config.conn_url());
                    MultiConnectionPool::Mysql(
                        Pool::builder()
                            .max_size(config.connection_count)
                            .build(manager)
                            .expect(
                                format!("Invalid config provided, {}", config.connection_name)
                                    .as_str(),
                            ),
                    )
                }
                DatabaseKind::SQLite => {
                    let manager = ConnectionManager::<SqliteConnection>::new(config.conn_url());
                    MultiConnectionPool::Sqlite(
                        Pool::builder()
                            .max_size(config.connection_count)
                            .build(manager)
                            .expect(
                                format!("Invalid config provided, {}", config.connection_name)
                                    .as_str(),
                            ),
                    )
                }
            };
            schema_manager.insert(config.connection_name.clone(), pool);
        }
        schema_manager
    }
}

impl PgSchemaManager {
    // hello darkness my old friend
    // This would have been easier in haskell

    pub fn get_pg_conn(&self, name: String) -> ResultConnection<PgConnection> {
        let conn = match self
            .get(&name)
            .expect(format!("Invalid connection name provided: {name}").as_str())
        {
            MultiConnectionPool::Pg(conn) => conn.get()?,
            _ => anyhow::bail!("Connection is not of type Postgres"),
        };
        Ok(conn)
    }

    pub fn get_mysql_conn(&self, name: String) -> ResultConnection<MysqlConnection> {
        let conn = match self
            .get(&name)
            .expect(format!("Invalid connection name provided: {name}").as_str())
        {
            MultiConnectionPool::Mysql(conn) => conn.get()?,
            _ => anyhow::bail!("Connection is not of type Mysql"),
        };
        Ok(conn)
    }

    pub fn get_sqlite_conn(&self, name: String) -> ResultConnection<SqliteConnection> {
        let conn = match self
            .get(&name)
            .expect(format!("Invalid connection name provided: {name}").as_str())
        {
            MultiConnectionPool::Sqlite(conn) => conn.get()?,
            _ => anyhow::bail!("Connection is not of type Sqlite"),
        };
        Ok(conn)
    }
}
