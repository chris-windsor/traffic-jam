use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{sse::Event, IntoResponse, Sse},
    routing::{get, post},
    Json, Router,
};
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use diesel::prelude::*;
use futures::Stream;
use http::Method;
use lazy_static::lazy_static;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{HashMap, VecDeque},
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::broadcast::{self, Sender};
use traffic_jam::*;

use crate::authorize_net::ChargeCreditCardRequest;
use crate::db::POOL;
use crate::ecommerce::{Discount, Invoice};
use crate::inventory::*;
use crate::models::*;
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize)]
struct ResultProduct {
    id: i32,
    title: String,
    stock: i32,
    price: BigDecimal,
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

#[derive(Clone)]
struct AppState {
    tx: Sender<String>,
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
    let (tx, _) = broadcast::channel::<String>(100);
    let app_state = AppState { tx: tx.clone() };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/products", get(query_products))
        .route(
            "/product/:product_id",
            get(product_data).post(update_product),
        )
        .route("/process_order", post(process_order))
        .route("/event_stream", get(sse_handler))
        .route("/event_socket", get(ws_handler))
        .layer(cors)
        .with_state(app_state.clone());
    let addr = SocketAddr::from(([127, 0, 0, 1], 4567));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct Pagination {
    offset: i64,
    limit: i64,
}

async fn query_products(
    query: Query<Pagination>,
) -> (StatusCode, Json<DetailedResponse<Vec<Product>>>) {
    use self::schema::products::dsl::*;

    let conn = &mut POOL.get().unwrap();

    let query = query.0;
    let offset = query.offset;
    let limit = query.limit;

    let results = products
        .offset(offset)
        .limit(limit)
        .load::<Product>(conn)
        .unwrap();

    (
        StatusCode::OK,
        Json(DetailedResponse {
            data: Some(results),
            error: None,
        }),
    )
}

async fn product_data(
    Path(product_id): Path<i32>,
) -> (StatusCode, Json<DetailedResponse<ResultProduct>>) {
    use self::schema::products::dsl::*;

    let conn = &mut POOL.get().unwrap();

    let filter_id: i32 = product_id;
    let result_product: Option<Product> = products.find(filter_id).first(conn).optional().unwrap();

    match result_product {
        Some(item) => (
            StatusCode::OK,
            Json(DetailedResponse {
                data: Some(ResultProduct {
                    id: item.id,
                    title: item.title,
                    stock: item.stock,
                    price: item.price,
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

async fn update_product(
    Path(product_id): Path<i32>,
    State(state): State<AppState>,
    Json(new_product): Json<Product>,
) -> (StatusCode, Json<DetailedResponse<Product>>) {
    use self::schema::products::dsl::*;

    let conn = &mut POOL.get().unwrap();

    let updated_product = diesel::update(products.find(product_id))
        .set((
            title.eq(new_product.title),
            stock.eq(new_product.stock),
            price.eq(new_product.price),
        ))
        .get_result::<Product>(conn);

    match updated_product {
        Ok(product) => {
            let completion_msg = json!(product).to_string();
            let _ = state.tx.send(completion_msg.to_owned());
            UDPATE_QUEUE
                .lock()
                .unwrap()
                .push_back(completion_msg.to_owned());

            (
                StatusCode::ACCEPTED,
                Json(DetailedResponse {
                    data: Some(product),
                    error: None,
                }),
            )
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(DetailedResponse {
                data: None,
                error: Some(RequestError {
                    message: "Could not find product to update".to_string(),
                    detail: "Please specify a valid product id".to_string(),
                }),
            }),
        ),
    }
}

async fn process_order(
    State(state): State<AppState>,
    Json(req_body): Json<CreateOrderRequest>,
) -> (StatusCode, Json<DetailedResponse<Order>>) {
    use self::schema::products::dsl::*;

    let order_id: usize = rand::thread_rng().gen_range(1..10000);
    let processing_msg = format!("Processing order {}", order_id).to_string();
    let _ = state.tx.send(processing_msg.to_owned());
    UDPATE_QUEUE
        .lock()
        .unwrap()
        .push_back(processing_msg.to_owned());

    let new_order: Order = Order {
        id: order_id,
        items: req_body.items,
    };

    let discounts: Vec<Discount> = vec![];

    let invoice = Invoice::create(
        &new_order,
        discounts,
        BigDecimal::from_f32(5.0).unwrap(),
        BigDecimal::from_f32(0.0715).unwrap(),
    );

    let process_handle = tokio::spawn(async move {
        if HOLDING_INVENTORY.lock().unwrap().hold_items(&new_order) {
            match ChargeCreditCardRequest::create(&new_order, invoice, req_body.customer).await {
                Ok(_) => {
                    HOLDING_INVENTORY.lock().unwrap().release_order(&order_id);

                    let order_product_ids: Vec<i32> =
                        new_order.items.iter().map(|item| item.id).collect();

                    let conn = &mut POOL.get().unwrap();
                    let new_stock_values = products
                        .filter(id.eq_any(order_product_ids))
                        .load::<Product>(conn)
                        .expect("Unable to retrieve current stock values for order products");

                    let completion_msg = json!(new_stock_values).to_string();
                    let _ = state.tx.send(completion_msg.to_owned());
                    UDPATE_QUEUE
                        .lock()
                        .unwrap()
                        .push_back(completion_msg.to_owned());
                    return (
                        StatusCode::OK,
                        Json(DetailedResponse {
                            data: Some(new_order),
                            error: None,
                        }),
                    );
                }
                Err(_) => {
                    HOLDING_INVENTORY.lock().unwrap().undo_hold(&order_id);
                    let failure_msg =
                        format!("Error while collecting payment for order #{}", order_id);
                    let _ = state.tx.send(failure_msg.to_owned());
                    UDPATE_QUEUE
                        .lock()
                        .unwrap()
                        .push_back(failure_msg.to_owned());
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

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|ws| async { ws_loop(state, ws).await })
}

async fn ws_loop(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        ws.send(Message::Text(msg)).await.unwrap();
    }
}
