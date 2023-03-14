use self::models::*;
use crate::inventory::*;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{sse::Event, Html, IntoResponse, Sse},
    routing::{get, post},
    Json, Router,
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use futures::Stream;
use lazy_static::lazy_static;
use rand::Rng;
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::broadcast::{self, Sender};
use traffic_jam::{authorize_net::ChargeCreditCardRequest, *};

mod inventory;

#[derive(Serialize)]
struct ResultProduct {
    id: i32,
    title: String,
    stock: i32,
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

type PgPool = Pool<ConnectionManager<PgConnection>>;

lazy_static! {
    pub static ref POOL: PgPool = create_pool();
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

    let app = Router::new()
        .route("/product/:product_id", get(product_data))
        .route("/process_order", post(process_order))
        .route("/dashboard", get(dashboard))
        .route("/event_stream", get(sse_handler))
        .route("/event_socket", get(ws_handler))
        .with_state(app_state.clone());
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

    let conn = &mut POOL.get().unwrap();

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
    State(state): State<AppState>,
    Json(order): Json<CreateOrder>,
) -> (StatusCode, Json<DetailedResponse<Order>>) {
    let order_id: usize = rand::thread_rng().gen_range(1..10000);
    let processing_msg = format!("Processing order {}", order_id).to_string();
    let _ = state.tx.send(processing_msg.to_owned());
    UDPATE_QUEUE
        .lock()
        .unwrap()
        .push_back(processing_msg.to_owned());

    let new_order: Order = Order {
        id: order_id,
        items: order.items,
    };

    let process_handle = tokio::spawn(async move {
        if HOLDING_INVENTORY.lock().unwrap().hold_items(&new_order) {
            match ChargeCreditCardRequest::create().await {
                Ok(charge_details) => {
                    HOLDING_INVENTORY.lock().unwrap().release_order(&order_id);

                    let completion_msg = format!(
                        "Completed order {}. It has refId {}",
                        order_id, charge_details.ref_id
                    )
                    .to_string();
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

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|ws| async { ws_loop(state, ws).await })
}

async fn ws_loop(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        ws.send(Message::Text(Json(&msg).to_string()))
            .await
            .unwrap();
    }
}
