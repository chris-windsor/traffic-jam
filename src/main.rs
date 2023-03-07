use self::models::*;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{sse::Event, Html, Sse},
    routing::{get, post},
    Json, Router,
};
use diesel::prelude::*;
use futures::Stream;
use lazy_static::lazy_static;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::sleep;
use traffic_jam::*;

#[derive(Clone, Deserialize, Serialize)]
struct Item {
    id: i32,
    qty: i32,
}

#[derive(Serialize)]
struct ResultProduct {
    id: i32,
    title: String,
    stock: i32,
}

#[derive(Clone)]
struct LockedInventory {
    items: HashMap<usize, Vec<Item>>,
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

impl LockedInventory {
    fn hold_items(&mut self, order: &Order) -> bool {
        use self::schema::products::dsl::*;

        let conn = &mut establish_connection();

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

    fn undo_hold(&mut self, order_id: &usize) {
        use self::schema::products::dsl::*;

        let conn = &mut establish_connection();

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

    fn release_order(&mut self, order_id: &usize) {
        self.items.remove(order_id);
    }
}

lazy_static! {
    static ref HOLDING_INVENTORY: Arc<Mutex<LockedInventory>> =
        Arc::new(Mutex::new(LockedInventory {
            items: HashMap::from([])
        }));
    static ref UDPATE_QUEUE: Arc<Mutex<VecDeque<String>>> =
        Arc::new(Mutex::new(VecDeque::from([])));
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/product/:product_id", get(product_data))
        .route("/process_order", post(process_order))
        .route("/dashboard", get(dashboard))
        .route("/event_stream", get(sse_handler));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn product_data(
    Path(product_id): Path<String>,
) -> (StatusCode, Json<DetailedResponse<ResultProduct>>) {
    use self::schema::products::dsl::*;

    let conn = &mut establish_connection();
    let filter_id: i32 = product_id.parse().unwrap_or(0);
    let result_product: Option<Product> = products.find(filter_id).first(conn).optional().unwrap();

    match result_product {
        Some(item) => (
            StatusCode::OK,
            Json(DetailedResponse {
                data: Some(ResultProduct {
                    id: item.id,
                    title: item.title,
                    stock: item.stock,
                }),
                error: None,
            }),
        ),
        None => (
            StatusCode::BAD_REQUEST,
            Json(DetailedResponse {
                data: None,
                error: Some(RequestError {
                    message: "Malformed Item Request".to_string(),
                    detail: format!(
                        "Item with id {} does not exist within the inventory",
                        product_id
                    )
                    .to_string(),
                }),
            }),
        ),
    }
}

async fn process_order(
    Json(order): Json<CreateOrder>,
) -> (StatusCode, Json<DetailedResponse<Order>>) {
    let order_id: usize = rand::thread_rng().gen_range(1..10000);
    println!("processing order #{}", order_id);

    let new_order: Order = Order {
        id: order_id,
        items: order.items,
    };

    let process_handle = tokio::spawn(async move {
        if HOLDING_INVENTORY.lock().unwrap().hold_items(&new_order) {
            if collect_payment(order_id).await {
                HOLDING_INVENTORY.lock().unwrap().release_order(&order_id);
                println!("Sucessfully collected payment for order #{}", order_id);
                UDPATE_QUEUE
                    .lock()
                    .unwrap()
                    .push_back(format!("Completed order {}", order_id).to_string());
                return (
                    StatusCode::OK,
                    Json(DetailedResponse {
                        data: Some(new_order),
                        error: None,
                    }),
                );
            } else {
                HOLDING_INVENTORY.lock().unwrap().undo_hold(&order_id);
                println!("Error while collecting payment for order #{}", order_id);
                return (
                    StatusCode::OK,
                    Json(DetailedResponse {
                        data: None,
                        error: Some(RequestError {
                            message: "Unable to process payment method".to_string(),
                            detail: "Invalid payment details".to_string(),
                        }),
                    }),
                );
            }
        }

        return (
            StatusCode::OK,
            Json(DetailedResponse {
                data: None,
                error: Some(RequestError {
                    message: "Unable to hold inventory for order".to_string(),
                    detail: "Item in order is most likely out of stock".to_string(),
                }),
            }),
        );
    });

    process_handle.await.unwrap()
}

async fn collect_payment(order_id: usize) -> bool {
    println!("Collecting payment for order {}", order_id);
    let sleep_time = Duration::from_secs(rand::thread_rng().gen_range(1..=3));
    sleep(sleep_time).await;
    let payment_status = rand::thread_rng().gen_bool(0.7);
    payment_status
}

async fn dashboard() -> Html<&'static str> {
    Html(std::include_str!("../dashboard.html"))
}

async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            let latest_update = UDPATE_QUEUE.lock().unwrap().pop_front();
            match latest_update {
                Some(data) => {
                    yield Ok(Event::default().data(data));
                },
                None => {}
            }
        }
    };

    Sse::new(stream)
}
