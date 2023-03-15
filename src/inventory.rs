use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{db::POOL, ecommerce::Customer, models::*};

#[derive(Clone, Deserialize, Serialize)]
pub struct Item {
    pub id: i32,
    pub qty: i32,
}

#[derive(Clone)]
pub struct LockedInventory {
    pub items: HashMap<usize, Vec<Item>>,
}

impl LockedInventory {
    pub fn hold_items(&mut self, order: &Order) -> bool {
        use crate::schema::products::dsl::*;

        let conn = &mut POOL.get().unwrap();

        let hold_transaction = conn
            .build_transaction()
            .read_write()
            .run::<(), diesel::result::Error, _>(|conn| {
                for order_item in &order.items {
                    let _ = diesel::update(products.find(order_item.id))
                        .set(stock.eq(stock - order_item.qty))
                        .execute(conn);

                    let result_product: Option<Product> =
                        products.find(order_item.id).first(conn).optional().unwrap();

                    match result_product {
                        Some(product) => {
                            if product.stock < 0 {
                                return Err(diesel::result::Error::RollbackTransaction);
                            }
                        }
                        None => {
                            panic!("Could not find item with id {}", order_item.id);
                        }
                    }
                }

                Ok(())
            });

        match hold_transaction {
            Ok(_) => {
                self.items.insert(order.id, order.items.clone());

                return true;
            }
            Err(_) => false,
        }
    }

    pub fn undo_hold(&mut self, order_id: &usize) {
        use crate::schema::products::dsl::*;

        let conn = &mut POOL.get().unwrap();

        for held_item in self.items.get(order_id).unwrap() {
            let result_product: Option<Product> =
                products.find(held_item.id).first(conn).optional().unwrap();
            match result_product {
                Some(_) => {
                    diesel::update(products.find(held_item.id))
                        .set(stock.eq(stock + held_item.qty))
                        .get_result::<Product>(conn)
                        .expect("Unable to return inventory");
                }
                None => {
                    panic!("Could not find item with id {}", held_item.id);
                }
            }
        }
    }

    pub fn release_order(&mut self, order_id: &usize) {
        self.items.remove(order_id);
    }
}

#[derive(Clone, Serialize)]
pub struct Order {
    pub id: usize,
    pub items: Vec<Item>,
}

#[derive(Clone, Deserialize)]
pub struct CreateOrderRequest {
    pub customer: Customer,
    pub items: Vec<Item>,
}
