pub mod schema;

use std::env;

use crate::schema::*;

use chrono::NaiveDateTime;
use diesel::{
    insert_into, ExpressionMethods, Insertable, QueryDsl, Queryable, QueryableByName, RunQueryDsl,
    Selectable,
};
use pgschema::{ConnectionConfig, PgSchemaManager};

#[derive(QueryableByName, Queryable, Selectable, Insertable, Clone, Debug)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = users)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}

fn main() {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let configs = [
        ConnectionConfig::new("test_1".into(), database_url.clone(), "test1".into(), 10),
        ConnectionConfig::new("test_2".into(), database_url.clone(), "test2".into(), 10),
    ];
    let schema_manager = PgSchemaManager::from(configs);
    use crate::users::dsl::*;
    let mut test1_conn = schema_manager.get_conn("test_1".into()).unwrap();
    let mut test2_conn = schema_manager.get_conn("test_2".into()).unwrap();
    let _ = insert_into(users)
        .values((username.eq("test1"), email.eq("test1@test.com")))
        .execute(&mut test1_conn);
    let _ = insert_into(users)
        .values((username.eq("test2"), email.eq("test2@test.com")))
        .execute(&mut test2_conn);
    let test_one_records = users
        .select(username)
        .load::<String>(&mut test1_conn)
        .expect("error");
    let test_two_records = users
        .select(username)
        .load::<String>(&mut test2_conn)
        .expect("error");
    println!(
        "test1: {:?}, test2: {:?}",
        test_one_records, test_two_records
    );
}
