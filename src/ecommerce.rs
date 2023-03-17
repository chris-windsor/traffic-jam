use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    authorize_net::{Address, AuthorizeNetFee, CreditCard},
    db::POOL,
    inventory::Order,
    models::*,
};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: String,
    pub ip_address: String,
    pub billing_address: Address,
    pub shipping_address: Address,
    pub credit_card: CreditCard,
}

pub struct Discount {
    amount: usize,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Invoice {
    pub subtotal: BigDecimal,
    pub shipping: BigDecimal,
    pub taxes: BigDecimal,
    pub total: BigDecimal,
}

impl Invoice {
    pub fn create(
        order: &Order,
        discounts: Vec<Discount>,
        shipping_fee: BigDecimal,
        tax_rate: BigDecimal,
    ) -> Self {
        let subtotal = Self::calc_subtotal(&order, discounts);
        let taxes = Self::calc_taxes(&subtotal, &tax_rate);

        Invoice {
            subtotal: subtotal.clone(),
            shipping: shipping_fee.clone(),
            taxes: taxes.clone(),
            total: subtotal.clone() + shipping_fee + taxes.clone(),
        }
    }

    fn calc_subtotal(order: &Order, discounts: Vec<Discount>) -> BigDecimal {
        use crate::schema::products::dsl::*;
        let conn = &mut POOL.get().unwrap();

        let mut subtotal = BigDecimal::from_f32(0.0).unwrap();

        for item in &order.items {
            let db_item: Option<Product> = products.find(item.id).first(conn).optional().unwrap();
            subtotal = subtotal + BigDecimal::from_i32(item.qty).unwrap() * db_item.unwrap().price;
        }

        for discount in discounts {
            subtotal = subtotal - BigDecimal::from_usize(discount.amount).unwrap();
        }

        subtotal
    }

    fn calc_taxes(subtotal: &BigDecimal, tax_rate: &BigDecimal) -> BigDecimal {
        subtotal * tax_rate
    }

    pub fn get_shipping(&self) -> AuthorizeNetFee {
        AuthorizeNetFee {
            name: String::from("Shipping"),
            description: String::from("Flat rate shipping fee"),
            amount: format!("{:.02}", self.shipping),
        }
    }

    pub fn get_taxes(&self) -> AuthorizeNetFee {
        AuthorizeNetFee {
            name: String::from("Taxes"),
            description: String::from(""),
            amount: format!("{:.02}", self.taxes),
        }
    }

    pub fn get_duty(&self) -> AuthorizeNetFee {
        AuthorizeNetFee {
            name: String::from(""),
            description: String::from(""),
            amount: String::from("0"),
        }
    }
}
