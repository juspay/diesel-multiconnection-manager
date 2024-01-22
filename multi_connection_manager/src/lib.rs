//! diesel multiconnection manager lets you create and manage connections to multiple database hosts, schemas and database systems in your rust codebase.
//!
//! The way to use it is -
//!
//! ### Create a connection config

//! ```
//! ConnectionConfig::new(
//!     connection_name,
//!     database_system,
//!     database_name,
//!     database_host_url,
//!     schema_name,
//!     max_connections_allowed,
//!     database_options,
//! )
//! ```
//! **Connection name must be unique** and database options is well, optional. This is what it would look like in code:
//!
//! ```
//! let configs = [
//!         ConnectionConfig::new(
//!             "test_1_pg".into(),
//!             DatabaseKind::Postgres,
//!             "test".into(),
//!             pg_database_url.clone(),
//!             "test1".into(),
//!             10,
//!             None,
//!         ),
//!         .... //other postgres configs
//!         ConnectionConfig::new(
//!             "test_1_mysql".into(),
//!             DatabaseKind::MySQL,
//!             "test1".into(),
//!             mysql_database_url.clone(),
//!             None, //schema does not even matter
//!             5,
//!             None,
//!         ),
//!         .... // other mysql configs
//!         ConnectionConfig::new(
//!             "test_1_sqlite".into(),
//!             DatabaseKind::SQLite,
//!             "test1.db".into(),
//!             sqlite_database_url.clone(),
//!             None, //schema does not even matter
//!             5,
//!             None,
//!         ),
//!         .... // other sqlite configs
//!     ];
//! ```

//! ### Initialize the MultiConnectionManager
//!
//! The connection manager lets you get connections to a particular database. Just give it a vector of connection configs to start
//!
//! ```
//! let manager = MultiConnectionManager::from(configs);
//! ```
//! ### Get a connection whenever you need it
//!
//! Based on the database system, call these functions to get a connection:
//! ```
//! schema_manager.get_pg_conn(connection_name)
//! schema_manager.get_mysql_conn(connection_name)
//! schema_manager.get_sqlite_conn(connection_name)
//! ```
//! All these functions return `PooledConnection<ConnectionManager<M>>` where `M` is any of `PgConnection | MysqlConnection | SqliteConnection`
//!
//! That's pretty much it, enjoy!
//!
//! Read more about dependencies and contributions here: [github link]

use derive_more::{Deref, DerefMut, Display};
use std::collections::HashMap;
use thiserror::Error;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

#[cfg(feature = "postgres")]
use diesel::PgConnection;

#[cfg(feature = "mysql")]
use diesel::MysqlConnection;

#[cfg(feature = "sqlite")]
use diesel::SqliteConnection;

#[derive(Debug, Display)]
pub enum DatabaseKind {
    #[cfg(feature = "postgres")]
    Postgres,
    #[cfg(feature = "mysql")]
    MySQL,
    #[cfg(feature = "sqlite")]
    SQLite,
}

#[derive(Error, Debug)]
pub enum McmError {
    #[error("Connection to {db} could not be made due to {error}")]
    ConnectionError { db: DatabaseKind, error: String },
    #[error("Connection to {db} could not be made due to {conn_name}")]
    InvalidConnectionNameError { db: DatabaseKind, conn_name: String },
    #[error("Connection is present but not made to a {db} database")]
    InvalidConnectionTypeError { db: DatabaseKind },
    #[error("{error} for connection {conn_name}, database {db}")]
    R2D2Error {
        db: DatabaseKind,
        conn_name: String,
        error: String,
    },
}

type GenericPool<M> = Pool<ConnectionManager<M>>;
type GenericConnection<M> = PooledConnection<ConnectionManager<M>>;
type ResultConnection<M> = Result<GenericConnection<M>, McmError>;
type McmResult<T> = Result<T, McmError>;

#[derive(Clone, Debug)]
pub enum MultiConnectionPool {
    #[cfg(feature = "postgres")]
    Pg(GenericPool<PgConnection>),
    #[cfg(feature = "mysql")]
    Mysql(GenericPool<MysqlConnection>),
    #[cfg(feature = "sqlite")]
    Sqlite(GenericPool<SqliteConnection>),
}

#[derive(Debug, Display)]
#[display(
    fmt = "connection config\nname : {}\ndatabase engine : {}\nURL: {}\nschema: {:?}\nconnection count: {}",
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
    schema: Option<String>,
    connection_count: u32,
    options: Option<String>,
}

impl ConnectionConfig {
    pub fn new(
        connection_name: String,
        database: DatabaseKind,
        database_name: String,
        database_host_url: String,
        schema: Option<String>,
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
            #[cfg(feature = "postgres")]
            DatabaseKind::Postgres => self.pg_conn_url(),
            #[cfg(feature = "mysql")]
            DatabaseKind::MySQL => self.mysql_conn_url(),
            #[cfg(feature = "sqlite")]
            DatabaseKind::SQLite => self.sqlite_conn_url(),
        }
    }

    fn pg_conn_url(&self) -> String {
        match (&self.options, &self.schema) {
            (None, None) => format!(
                "{}/{}?options=-c%20search_path%3D$user,public",
                self.database_host_url, self.database_name
            ),
            (None, Some(sch)) => format!(
                "{}/{}?options=-c%20search_path%3D{},$user,public",
                self.database_host_url, self.database_name, sch
            ),
            (Some(configs), None) => format!(
                "{}/{}?{}&options=-c%20search_path%3D$user,public",
                self.database_host_url, self.database_name, configs
            ),
            (Some(configs), Some(sch)) => format!(
                "{}/{}?{}&options=-c%20search_path%3D{},$user,public",
                self.database_host_url, self.database_name, configs, sch
            ),
        }
    }

    fn mysql_conn_url(&self) -> String {
        // mysql considers schema and database name the same
        if let Some(configs) = &self.options {
            return format!(
                "{}/{}?{}",
                self.database_host_url, self.database_name, configs
            );
        }

        format!("{}/{}", self.database_host_url, self.database_name)
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
pub struct MultiConnectionManager(HashMap<String, MultiConnectionPool>);

impl MultiConnectionManager {
    pub fn new(value: Vec<ConnectionConfig>) -> McmResult<Self> {
        let mut schema_manager: MultiConnectionManager = MultiConnectionManager(HashMap::new());
        for config in value.into_iter() {
            let pool: MultiConnectionPool = match config.database {
                #[cfg(feature = "postgres")]
                DatabaseKind::Postgres => {
                    let manager = ConnectionManager::<PgConnection>::new(config.conn_url());
                    MultiConnectionPool::Pg(
                        Pool::builder()
                            .max_size(config.connection_count)
                            .build(manager)
                            .map_err(|err| McmError::ConnectionError {
                                db: DatabaseKind::Postgres,
                                error: err.to_string(),
                            })?,
                    )
                }
                #[cfg(feature = "mysql")]
                DatabaseKind::MySQL => {
                    let manager = ConnectionManager::<MysqlConnection>::new(config.conn_url());
                    MultiConnectionPool::Mysql(
                        Pool::builder()
                            .max_size(config.connection_count)
                            .build(manager)
                            .map_err(|err| McmError::ConnectionError {
                                db: DatabaseKind::MySQL,
                                error: err.to_string(),
                            })?,
                    )
                }
                #[cfg(feature = "sqlite")]
                DatabaseKind::SQLite => {
                    let manager = ConnectionManager::<SqliteConnection>::new(config.conn_url());
                    MultiConnectionPool::Sqlite(
                        Pool::builder()
                            .max_size(config.connection_count)
                            .build(manager)
                            .map_err(|err| McmError::ConnectionError {
                                db: DatabaseKind::SQLite,
                                error: err.to_string(),
                            })?,
                    )
                }
            };
            schema_manager.insert(config.connection_name.clone(), pool);
        }
        Ok(schema_manager)
    }

    // hello darkness my old friend
    // This would have been easier in haskell
    #[cfg(feature = "postgres")]
    pub fn get_pg_conn(&self, name: &'static str) -> ResultConnection<PgConnection> {
        let conn = match self.get(name).ok_or(McmError::InvalidConnectionNameError {
            db: DatabaseKind::Postgres,
            conn_name: name.into(),
        })? {
            MultiConnectionPool::Pg(conn) => conn.get().map_err(|err| McmError::R2D2Error {
                db: DatabaseKind::Postgres,
                conn_name: name.into(),
                error: err.to_string(),
            })?,
            _ => {
                return Err(McmError::InvalidConnectionTypeError {
                    db: DatabaseKind::Postgres,
                })
            }
        };
        Ok(conn)
    }

    #[cfg(feature = "mysql")]
    pub fn get_mysql_conn(&self, name: &'static str) -> ResultConnection<MysqlConnection> {
        let conn = match self.get(name).ok_or(McmError::InvalidConnectionNameError {
            db: DatabaseKind::MySQL,
            conn_name: name.into(),
        })? {
            MultiConnectionPool::Mysql(conn) => conn.get().map_err(|err| McmError::R2D2Error {
                db: DatabaseKind::MySQL,
                conn_name: name.into(),
                error: err.to_string(),
            })?,
            _ => {
                return Err(McmError::InvalidConnectionTypeError {
                    db: DatabaseKind::MySQL,
                })
            }
        };
        Ok(conn)
    }

    #[cfg(feature = "sqlite")]
    pub fn get_sqlite_conn(&self, name: &'static str) -> ResultConnection<SqliteConnection> {
        let conn = match self.get(name).ok_or(McmError::InvalidConnectionNameError {
            db: DatabaseKind::SQLite,
            conn_name: name.into(),
        })? {
            MultiConnectionPool::Sqlite(conn) => conn.get().map_err(|err| McmError::R2D2Error {
                db: DatabaseKind::SQLite,
                conn_name: name.into(),
                error: err.to_string(),
            })?,
            _ => {
                return Err(McmError::InvalidConnectionTypeError {
                    db: DatabaseKind::SQLite,
                })
            }
        };
        Ok(conn)
    }
}

/*
TODO: And yeah, update the documentation with prerequisites that need to be installed (PostgreSQL and MySQL packages), etc. Stuff for users who don’t use nix basically.
*/
