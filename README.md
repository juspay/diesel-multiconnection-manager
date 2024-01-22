# Diesel MultiConnection Manager

diesel multiconnection manager lets you create and manage connections to multiple database hosts, schemas and database systems in your rust codebase.

## Use cases

- multi-tenancy
- CQRS
- DB operations to location specific partitions 
- Wherever you need multiple database systems, database hosts or schemas

## Dependencies

- Diesel

## installation

Add the crates.io installation guide here

## How to use

### Setup the connection configurations

Create a connection config

```
ConnectionConfig::new(
    connection_name,
    database_system,
    database_name,
    database_host_url,
    schema_name,
    max_connections_allowed,
    database_options,
)
```
**Connection name must be unique** and database options is well, optional. This is what it would look like in code:

```
let configs = [
        ConnectionConfig::new(
            "test_1_pg".into(),
            DatabaseKind::Postgres,
            "test".into(),
            pg_database_url.clone(),
            "test1".into(),
            10,
            None,
        ),
        ....
        ConnectionConfig::new(
            "test_1_mysql".into(),
            DatabaseKind::MySQL,
            "test1".into(),
            mysql_database_url.clone(),
            None,
            5,
            None,
        ),
        ....
        ConnectionConfig::new(
            "test_1_sqlite".into(),
            DatabaseKind::SQLite,
            "test1.db".into(),
            sqlite_database_url.clone(),
            None,
            5,
            None,
        ),
        ....
    ];
```

### Initialize the MultiConnectionManager

The connection manager lets you get connections to a particular database. Just give it a vector of connection configs to start

```
let manager = MultiConnectionManager::from(configs);
```
### Get a connection whenever you need it

Based on the database system, call these functions to get a connection:
```
get_pg_conn(connection_name)
get_mysql_conn(connection_name)
get_sqlite_conn(connection_name)
```
All these functions return `PooledConnection<ConnectionManager<M>>` where `M` is any of `PgConnection | MysqlConnection | SqliteConnection` 

That's pretty much it, enjoy!