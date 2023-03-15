use traffic_jam::{
    authorize_net::{Address, ChargeCreditCardRequest, CreditCard},
    ecommerce::Customer,
    inventory::Order,
};

#[tokio::main]
async fn main() {
    let order = Order {
        id: 123,
        items: vec![],
    };

    let customer = Customer {
        first_name: String::from(""),
        last_name: String::from(""),
        email: String::from(""),
        phone_number: String::from(""),
        ip_address: String::from(""),
        billing_address: Address {
            first_name: String::from(""),
            last_name: String::from(""),
            company: String::from(""),
            address: String::from(""),
            city: String::from(""),
            state: String::from(""),
            zip: String::from(""),
            country: String::from(""),
        },
        shipping_address: Address {
            first_name: String::from(""),
            last_name: String::from(""),
            company: String::from(""),
            address: String::from(""),
            city: String::from(""),
            state: String::from(""),
            zip: String::from(""),
            country: String::from(""),
        },
        credit_card: CreditCard {
            card_code: String::from(""),
            card_number: String::from(""),
            expiration_date: String::from(""),
        },
    };

    ChargeCreditCardRequest::create(&order, customer)
        .await
        .expect("Unable to make payment capture");
}
