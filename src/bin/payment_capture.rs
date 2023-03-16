use bigdecimal::{BigDecimal, FromPrimitive};

use traffic_jam::{
    authorize_net::{Address, ChargeCreditCardRequest, CreditCard},
    ecommerce::{Customer, Discount, Invoice},
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

    let discounts: Vec<Discount> = vec![];

    let invoice = Invoice::create(
        &order,
        discounts,
        BigDecimal::from_f32(5.0).unwrap(),
        BigDecimal::from_f32(0.0715).unwrap(),
    );

    ChargeCreditCardRequest::create(&order, invoice, customer)
        .await
        .expect("Unable to make payment capture");
}
