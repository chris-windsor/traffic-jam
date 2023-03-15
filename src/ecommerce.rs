use serde::{Deserialize, Serialize};

use crate::authorize_net::{Address, CreditCard};

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
