use dotenvy::dotenv;
use rand::Rng;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargeCreditCardRequest {
    create_transaction_request: CreateTransactionRequest,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateTransactionRequest {
    merchant_authentication: MerchantAuthentication,
    ref_id: String,
    transaction_request: TransactionRequest,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MerchantAuthentication {
    name: String,
    transaction_key: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionRequest {
    transaction_type: String,
    amount: String,
    payment: Payment,
    line_items: Vec<()>,
    tax: Fee,
    duty: Fee,
    shipping: Fee,
    po_number: String,
    customer: Customer,
    bill_to: Address,
    ship_to: Address,
    #[serde(rename(serialize = "customerIP"))]
    customer_ip: String,
    transaction_settings: TransactionSettings,
    user_fields: UserFields,
    processing_options: ProcessingOptions,
    subsequent_auth_information: SubsequentAuthInformation,
    authorization_indicator_type: AuthorizationIndicatorType,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizationIndicatorType {
    authorization_indicator: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Address {
    first_name: String,
    last_name: String,
    company: String,
    address: String,
    city: String,
    state: String,
    zip: String,
    country: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Customer {
    id: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Fee {
    amount: String,
    name: String,
    description: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LineItems {
    line_item: LineItem,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LineItem {
    item_id: String,
    name: String,
    description: String,
    quantity: String,
    unit_price: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Payment {
    credit_card: CreditCard,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreditCard {
    card_number: String,
    expiration_date: String,
    card_code: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessingOptions {
    is_subsequent_auth: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SubsequentAuthInformation {
    original_network_trans_id: String,
    original_auth_amount: String,
    reason: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionSettings {
    setting: TransactionSetting,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionSetting {
    setting_name: String,
    setting_value: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserFields {
    user_field: Vec<UserField>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserField {
    name: String,
    value: String,
}

impl ChargeCreditCardRequest {
    pub async fn create() -> Result<(), Box<dyn std::error::Error>> {
        dotenv().ok();
        let client = reqwest::Client::new();

        let merchant_id = env::var("MERCHANT_ID").expect("Could not get MERCHANT_ID from .env");
        let transaction_key =
            env::var("TRANSACTION_KEY").expect("Could not get TRANSACTION_KEY from .env");

        let ref_id = String::from("123456");
        let transaction_type = String::from("authCaptureTransaction");
        let transaction_total = String::from("100.0");

        let customer_ip = String::from("192.168.1.1");
        let credit_card_number = String::from("5424000000000015");
        let credit_card_exp = String::from("2025-12");
        let credit_card_cvv = String::from("999");

        let po_number = rand::thread_rng().gen_range(0..100000).to_string();
        let customer_id = String::from("99999456654");

        let charge_request = ChargeCreditCardRequest {
            create_transaction_request: CreateTransactionRequest {
                merchant_authentication: MerchantAuthentication {
                    name: merchant_id,
                    transaction_key,
                },
                ref_id,
                transaction_request: TransactionRequest {
                    transaction_type,
                    amount: transaction_total,
                    payment: Payment {
                        credit_card: CreditCard {
                            card_code: credit_card_cvv,
                            card_number: credit_card_number,
                            expiration_date: credit_card_exp,
                        },
                    },
                    line_items: vec![],
                    tax: Fee {
                        amount: "7.32".to_string(),
                        description: "test".to_string(),
                        name: "taxes".to_string(),
                    },
                    duty: Fee {
                        amount: "8.55".to_string(),
                        description: "test".to_string(),
                        name: "duty".to_string(),
                    },
                    shipping: Fee {
                        amount: "5.25".to_string(),
                        description: "test".to_string(),
                        name: "shipping".to_string(),
                    },
                    po_number,
                    customer: Customer { id: customer_id },
                    bill_to: Address {
                        address: "123 Main St".to_string(),
                        city: "Lehi".to_string(),
                        company: "".to_string(),
                        country: "US".to_string(),
                        first_name: "Jamie".to_string(),
                        last_name: "Son".to_string(),
                        state: "UT".to_string(),
                        zip: "84043".to_string(),
                    },
                    ship_to: Address {
                        address: "123 Main St".to_string(),
                        city: "Lehi".to_string(),
                        company: "".to_string(),
                        country: "US".to_string(),
                        first_name: "Jamie".to_string(),
                        last_name: "Son".to_string(),
                        state: "UT".to_string(),
                        zip: "84043".to_string(),
                    },
                    customer_ip: customer_ip,
                    transaction_settings: TransactionSettings {
                        setting: TransactionSetting {
                            setting_name: "testRequest".to_string(),
                            setting_value: "false".to_string(),
                        },
                    },
                    user_fields: UserFields { user_field: vec![] },
                    processing_options: ProcessingOptions {
                        is_subsequent_auth: "true".to_string(),
                    },
                    subsequent_auth_information: SubsequentAuthInformation {
                        original_auth_amount: "45".to_string(),
                        original_network_trans_id: "123456789NNNH".to_string(),
                        reason: "resubmission".to_string(),
                    },
                    authorization_indicator_type: AuthorizationIndicatorType {
                        authorization_indicator: "final".to_string(),
                    },
                },
            },
        };

        let response = client
            .post("https://apitest.authorize.net/xml/v1/request.api")
            .header(CONTENT_TYPE, "application/json")
            .json(&charge_request)
            .send()
            .await?
            .text()
            .await?;

        // Authorize.NET returns a ZWSP at the start of the JSON response
        let response = str::replace(&response, "\u{feff}", "");

        let response: serde_json::Value = serde_json::from_str(&response)?;

        println!("Res: {}", response["transactionResponse"]["responseCode"]);

        Ok(())
    }
}
