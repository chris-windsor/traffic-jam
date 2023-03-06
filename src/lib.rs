pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use models::Product;
use std::env;

use crate::models::NewProduct;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("Unable to retrieve database URL from .env");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_product(conn: &mut PgConnection, title: &str, stock: &i32) -> Product {
    use crate::schema::products;

    let new_product = NewProduct { title, stock };

    diesel::insert_into(products::table)
        .values(&new_product)
        .get_result(conn)
        .expect("Error encountered while saving product")
}
