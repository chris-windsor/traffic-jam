use traffic_jam::{authorize_net::ChargeCreditCardRequest, inventory::Order};

#[tokio::main]
async fn main() {
    let order = Order {
        id: 123,
        items: vec![],
    };

    ChargeCreditCardRequest::create(&order)
        .await
        .expect("Unable to make payment capture");
}
