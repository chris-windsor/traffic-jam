use self::models::*;
use crate::POOL;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use traffic_jam::*;

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
        use self::schema::products::dsl::*;

        let conn = &mut POOL.get().unwrap();

        for order_item in &order.items {
            let result_product: Option<Product> =
                products.find(order_item.id).first(conn).optional().unwrap();
            match result_product {
                Some(item) => {
                    if item.stock >= order_item.qty {
                        diesel::update(products.find(order_item.id))
                            .set(stock.eq(stock - order_item.qty))
                            .get_result::<Product>(conn)
                            .expect("Unable to take inventory");
                    } else {
                        println!(
                            "| Order #{} over requested\n -{} has qty of {} but order is requesting qty of {}",
                            order.id, item.title, item.stock, order_item.qty
                        );
                        return false;
                    }
                }
                None => {
                    panic!("Could not find item with id {}", order_item.id);
                }
            }
        }

        self.items.insert(order.id, order.items.clone());
        true
    }

    pub fn undo_hold(&mut self, order_id: &usize) {
        use self::schema::products::dsl::*;

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
pub struct CreateOrder {
    pub items: Vec<Item>,
}
