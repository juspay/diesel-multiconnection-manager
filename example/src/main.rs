pub mod schema;

use std::env;

use crate::schema::*;

use diesel::{
    insert_into,
    r2d2::{ConnectionManager, PooledConnection},
    ExpressionMethods, Insertable, MysqlConnection, PgConnection, QueryDsl, Queryable,
    QueryableByName, RunQueryDsl, Selectable, SqliteConnection,
};
use tenancy::{ConnectionConfig, MultiConnectionManager};

#[derive(QueryableByName, Queryable, Selectable, Insertable, Clone, Debug)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = users)]
pub struct User {
    pub username: String,
    pub email: String,
}

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let pg_database_url =
        env::var("POSTGRES_DATABASE_URL").expect("POSTGRES_DATABASE_URL must be set");
    let mysql_database_url =
        env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL must be set");
    let sqlite_database_url = String::new();
    let configs = [
        ConnectionConfig::new(
            "test_1_pg".into(),
            tenancy::DatabaseKind::Postgres,
            "test".into(),
            pg_database_url.clone(),
            "test1".into(),
            10,
            None,
        ),
        ConnectionConfig::new(
            "test_2_pg".into(),
            tenancy::DatabaseKind::Postgres,
            "test".into(),
            pg_database_url.clone(),
            "test2".into(),
            10,
            None,
        ),
        ConnectionConfig::new(
            "test_1_mysql".into(),
            tenancy::DatabaseKind::MySQL,
            "test1".into(),
            mysql_database_url.clone(),
            "test1".into(), //schema does not even matter
            5,
            None,
        ),
        ConnectionConfig::new(
            "test_2_mysql".into(),
            tenancy::DatabaseKind::MySQL,
            "test2".into(),
            mysql_database_url.clone(),
            "test2".into(),
            10,
            None,
        ),
        ConnectionConfig::new(
            "test_1_sqlite".into(),
            tenancy::DatabaseKind::SQLite,
            "test1.db".into(),
            sqlite_database_url.clone(),
            "test1".into(), //schema does not even matter
            5,
            None,
        ),
        ConnectionConfig::new(
            "test_2_sqlite".into(),
            tenancy::DatabaseKind::SQLite,
            "test2.db".into(),
            sqlite_database_url.clone(),
            "test2".into(),
            10,
            None,
        ),
    ];
    let schema_manager = MultiConnectionManager::from(configs);

    run_example_postgres(
        schema_manager.get_pg_conn("test_1_pg".into()).unwrap(),
        schema_manager.get_pg_conn("test_2_pg".into()).unwrap(),
    );

    run_example_mysql(
        schema_manager
            .get_mysql_conn("test_1_mysql".into())
            .unwrap(),
        schema_manager
            .get_mysql_conn("test_2_mysql".into())
            .unwrap(),
    );

    run_example_sqlite(
        schema_manager
            .get_sqlite_conn("test_1_sqlite".into())
            .unwrap(),
        schema_manager
            .get_sqlite_conn("test_2_sqlite".into())
            .unwrap(),
    );
    Ok(())
}

fn run_example_postgres(
    mut test1_conn: PooledConnection<ConnectionManager<PgConnection>>,
    mut test2_conn: PooledConnection<ConnectionManager<PgConnection>>,
) {
    use crate::users::dsl::*;
    let test_number = rand::random::<i32>();
    let record = (
        username.eq(format!("test{}", test_number)),
        email.eq(format!("test{}@test.com", test_number)),
    );
    let _ = insert_into(users)
        .values(record.clone())
        .execute(&mut test1_conn);
    let _ = insert_into(users).values(record).execute(&mut test2_conn);
    let test_one_records = users
        .select(username)
        .load::<String>(&mut test1_conn)
        .expect("error");
    let test_two_records = users
        .select(username)
        .load::<String>(&mut test2_conn)
        .expect("error");
    println!(
        "postgres test1: {:?}, test2: {:?}",
        test_one_records, test_two_records
    );
}

fn run_example_mysql(
    mut test1_conn: PooledConnection<ConnectionManager<MysqlConnection>>,
    mut test2_conn: PooledConnection<ConnectionManager<MysqlConnection>>,
) {
    let test_number = rand::random::<i32>();
    let record = (
        username.eq(format!("test{}", test_number)),
        email.eq(format!("test{}@test.com", test_number)),
    );
    use crate::users::dsl::*;
    let _ = insert_into(users)
        .values(record.clone())
        .execute(&mut test1_conn);
    let _ = insert_into(users).values(record).execute(&mut test2_conn);
    let test_one_records = users
        .select(username)
        .load::<String>(&mut test1_conn)
        .expect("error");
    let test_two_records = users
        .select(username)
        .load::<String>(&mut test2_conn)
        .expect("error");
    println!(
        "mysql test1: {:?}, test2: {:?}",
        test_one_records, test_two_records
    );
}

fn run_example_sqlite(
    mut test1_conn: PooledConnection<ConnectionManager<SqliteConnection>>,
    mut test2_conn: PooledConnection<ConnectionManager<SqliteConnection>>,
) {
    let test_number = rand::random::<i32>();
    let record = (
        username.eq(format!("test{}", test_number)),
        email.eq(format!("test{}@test.com", test_number)),
    );
    use crate::users::dsl::*;
    let _ = insert_into(users)
        .values(record.clone())
        .execute(&mut test1_conn);
    let _ = insert_into(users).values(record).execute(&mut test2_conn);
    let test_one_records = users
        .select(username)
        .load::<String>(&mut test1_conn)
        .expect("error");
    let test_two_records = users
        .select(username)
        .load::<String>(&mut test2_conn)
        .expect("error");
    println!(
        "sqlite test1: {:?}, test2: {:?}",
        test_one_records, test_two_records
    );
}
