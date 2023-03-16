use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::authorize_net::{Address, AuthorizeNetFee, CreditCard};

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

#[derive(Deserialize, Serialize, Clone)]
pub struct Invoice {
    pub subtotal: BigDecimal,
    pub shipping: BigDecimal,
    pub taxes: BigDecimal,
    pub total: BigDecimal,
}

impl Invoice {
    pub fn create(subtotal: BigDecimal, shipping: BigDecimal, taxes: BigDecimal) -> Self {
        Invoice {
            subtotal: subtotal.clone(),
            shipping: shipping.clone(),
            taxes: taxes.clone(),
            total: subtotal + shipping + taxes,
        }
    }

    pub fn get_shipping(&self) -> AuthorizeNetFee {
        AuthorizeNetFee {
            name: String::from("Shipping"),
            description: String::from("Flat rate shipping fee"),
            amount: self.shipping.to_string(),
        }
    }

    pub fn get_taxes(&self) -> AuthorizeNetFee {
        AuthorizeNetFee {
            name: String::from("Taxes"),
            description: String::from(""),
            amount: self.taxes.to_string(),
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
