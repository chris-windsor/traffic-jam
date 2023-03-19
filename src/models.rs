use bigdecimal::BigDecimal;
use diesel::prelude::*;
use serde::Serialize;

use crate::schema::products;

#[derive(Queryable, Serialize)]
pub struct Product {
    pub id: i32,
    pub title: String,
    pub stock: i32,
    pub price: BigDecimal,
}

#[derive(Insertable)]
#[diesel(table_name = products)]
pub struct NewProduct<'a> {
    pub title: &'a str,
    pub stock: &'a i32,
}
