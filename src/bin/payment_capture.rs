use traffic_jam::authorize_net::ChargeCreditCardRequest;

#[tokio::main]
async fn main() {
    ChargeCreditCardRequest::create()
        .await
        .expect("Unable to make payment capture");
}
