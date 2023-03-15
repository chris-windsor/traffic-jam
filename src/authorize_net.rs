use dotenvy::dotenv;
use rand::Rng;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use std::env;

use crate::{ecommerce::Customer, inventory::Order};

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
    tax: AuthorizeNetFee,
    duty: AuthorizeNetFee,
    shipping: AuthorizeNetFee,
    po_number: String,
    customer: AuthorizeNetCustomer,
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

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub first_name: String,
    pub last_name: String,
    pub company: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizeNetCustomer {
    id: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizeNetFee {
    amount: String,
    name: String,
    description: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Payment {
    credit_card: CreditCard,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreditCard {
    pub card_number: String,
    pub expiration_date: String,
    pub card_code: String,
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

impl UserFields {
    fn get_default() -> Self {
        UserFields { user_field: vec![] }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserField {
    name: String,
    value: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargeCreditCardResponse {
    transaction_response: TransactionResponse,
    pub ref_id: String,
    messages: TransactionResponseResultMessages,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionResponse {
    response_code: String,
    auth_code: String,
    avs_result_code: String,
    cvv_result_code: String,
    trans_id: String,
    #[serde(rename(deserialize = "refTransID"))]
    ref_trans_id: String,
    trans_hash: String,
    test_request: String,
    account_number: String,
    account_type: String,
    messages: Vec<TransactionResponseMessage>,
    #[serde(default = "UserFields::get_default")]
    user_fields: UserFields,
    trans_hash_sha2: String,
    #[serde(rename(deserialize = "SupplementalDataQualificationIndicator"))]
    supplemental_data_qualification_indicator: usize,
    #[serde(default)]
    network_trans_id: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionResponseResultMessages {
    result_code: String,
    message: Vec<TransactionResponseResultMessage>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionResponseResultMessage {
    code: String,
    text: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionResponseMessage {
    code: String,
    description: String,
}

impl ChargeCreditCardRequest {
    pub async fn create(
        order: &Order,
        customer: Customer,
    ) -> Result<ChargeCreditCardResponse, Box<dyn std::error::Error>> {
        dotenv().ok();
        let client = reqwest::Client::new();

        let merchant_id = env::var("MERCHANT_ID").expect("Could not get MERCHANT_ID from .env");
        let transaction_key =
            env::var("TRANSACTION_KEY").expect("Could not get TRANSACTION_KEY from .env");

        let ref_id = String::from(order.id.to_string());
        let transaction_type = String::from("authCaptureTransaction");
        let transaction_total = String::from("100.0");

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
                            card_code: customer.credit_card.card_code,
                            card_number: customer.credit_card.card_number,
                            expiration_date: customer.credit_card.expiration_date,
                        },
                    },
                    line_items: vec![],
                    tax: AuthorizeNetFee {
                        amount: "7.32".to_string(),
                        description: "test".to_string(),
                        name: "taxes".to_string(),
                    },
                    duty: AuthorizeNetFee {
                        amount: "8.55".to_string(),
                        description: "test".to_string(),
                        name: "duty".to_string(),
                    },
                    shipping: AuthorizeNetFee {
                        amount: "5.25".to_string(),
                        description: "test".to_string(),
                        name: "shipping".to_string(),
                    },
                    po_number,
                    customer: AuthorizeNetCustomer { id: customer_id },
                    bill_to: customer.billing_address,
                    ship_to: customer.shipping_address,
                    customer_ip: customer.ip_address,
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

        let response: ChargeCreditCardResponse = serde_json::from_str(&response)?;
        Ok(response)
    }
}
