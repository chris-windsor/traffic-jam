pub mod authorize_net;
pub mod db;
pub mod inventory;
pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy::dotenv;
use models::Product;
use std::env;

use crate::models::NewProduct;

pub fn create_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Unable to retrieve database URL from .env");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not create connection pool")
}

pub fn create_product(conn: &mut PgConnection, title: &str, stock: &i32) -> Product {
    use crate::schema::products;

    let new_product = NewProduct { title, stock };

    diesel::insert_into(products::table)
        .values(&new_product)
        .get_result(conn)
        .expect("Error encountered while saving product")
}
