use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use lazy_static::lazy_static;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::sleep;

#[derive(Clone, Deserialize, Serialize)]
struct Item {
    name: String,
    qty: usize,
}

#[derive(Clone)]
struct Inventory {
    items: HashMap<String, Item>,
    held: HashMap<usize, Vec<Item>>,
}

#[derive(Clone, Serialize)]
struct Order {
    id: usize,
    items: Vec<Item>,
}

#[derive(Clone, Deserialize)]
struct CreateOrder {
    items: Vec<Item>,
}

#[derive(Serialize)]
struct RequestError {
    message: String,
    detail: String,
}

#[derive(Serialize)]
struct DetailedResponse<T> {
    data: Option<T>,
    error: Option<RequestError>,
}

impl Inventory {
    fn hold_items(&mut self, order: &Order) -> bool {
        let mut clone_inventory = self.items.clone();
        for order_item in &order.items {
            match clone_inventory.get_mut(&order_item.name) {
                Some(item) => {
                    if item.qty >= order_item.qty {
                        item.qty -= order_item.qty;
                    } else {
                        println!(
                            "| Order #{} over requested\n -{} has qty of {} but order is requesting qty of {}",
                            order.id, item.name, item.qty, order_item.qty
                        );
                        return false;
                    }
                }
                None => {
                    panic!("Could not find item with name {}", order_item.name);
                }
            }
        }

        self.items = clone_inventory;
        self.held.insert(order.id, order.items.clone());
        true
    }

    fn undo_hold(&mut self, order_id: &usize) {
        for held_item in self.held.get(order_id).unwrap() {
            match self.items.get_mut(&held_item.name) {
                Some(item) => item.qty += held_item.qty,
                None => {
                    panic!("Could not find item with name {}", held_item.name);
                }
            }
        }
    }

    fn release_order(&mut self, order_id: &usize) {
        self.held.remove(order_id);
    }

    fn log_inventory(&self) {
        println!("\n| Current Inventory");
        for (item_name, item) in &self.items {
            println!("- {} has a qty of {}", item_name, item.qty);
        }
        println!("\n");
    }
}

lazy_static! {
    static ref WORKING_INVENTORY: Arc<Mutex<Inventory>> = Arc::new(Mutex::new(Inventory {
        held: HashMap::from([]),
        items: HashMap::from([
            (
                "item1".to_string(),
                Item {
                    name: "item1".to_string(),
                    qty: 4,
                },
            ),
            (
                "item2".to_string(),
                Item {
                    name: "item2".to_string(),
                    qty: 3,
                },
            ),
        ]),
    }));
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/product/:id", get(product_data))
        .route("/process_order", post(process_order));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn product_data(Path(id): Path<String>) -> (StatusCode, Json<DetailedResponse<Item>>) {
    match WORKING_INVENTORY.lock().unwrap().items.get(&id) {
        Some(item) => (
            StatusCode::OK,
            Json(DetailedResponse {
                data: Some(item.to_owned()),
                error: None,
            }),
        ),
        None => (
            StatusCode::BAD_REQUEST,
            Json(DetailedResponse {
                data: None,
                error: Some(RequestError {
                    message: "Malformed Item Request".to_string(),
                    detail: format!("Item with id {} does not exist within the inventory", id)
                        .to_string(),
                }),
            }),
        ),
    }
}

async fn process_order(Json(order): Json<CreateOrder>) -> (StatusCode, Json<Order>) {
    let order_id: usize = rand::thread_rng().gen_range(1..10000);
    println!("processing order #{}", order_id);

    let new_order = Order {
        id: order_id,
        items: order.items,
    };

    tokio::spawn(async move {
        if WORKING_INVENTORY.lock().unwrap().hold_items(&new_order) {
            if collect_payment(order_id).await {
                WORKING_INVENTORY.lock().unwrap().release_order(&order_id);
                println!("Sucessfully collected payment for order #{}", order_id);
                WORKING_INVENTORY.lock().unwrap().log_inventory();
            } else {
                WORKING_INVENTORY.lock().unwrap().undo_hold(&order_id);
                println!("Error while collecting payment for order #{}", order_id);
            }
        }
    });

    let new_order = Order {
        id: order_id,
        items: vec![],
    };

    (StatusCode::OK, Json(new_order))
}

async fn collect_payment(order_id: usize) -> bool {
    println!("Collecting payment for order {}", order_id);
    let sleep_time = Duration::from_secs(rand::thread_rng().gen_range(1..=3));
    sleep(sleep_time).await;
    let payment_status = rand::thread_rng().gen_bool(0.7);
    payment_status
}
