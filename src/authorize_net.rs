use std::env;

use dotenvy::dotenv;
use rand::Rng;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ChargeCreditCardRequest {
    createTransactionRequest: CreateTransactionRequest,
}

#[derive(Deserialize, Serialize)]
struct CreateTransactionRequest {
    merchantAuthentication: MerchantAuthentication,
    refId: String,
    transactionRequest: TransactionRequest,
}

#[derive(Deserialize, Serialize)]
struct MerchantAuthentication {
    name: String,
    transactionKey: String,
}

#[derive(Deserialize, Serialize)]
struct TransactionRequest {
    transactionType: String,
    amount: String,
    payment: Payment,
    lineItems: Vec<()>,
    tax: Fee,
    duty: Fee,
    shipping: Fee,
    poNumber: String,
    customer: Customer,
    billTo: Address,
    shipTo: Address,
    customerIP: String,
    transactionSettings: TransactionSettings,
    userFields: UserFields,
    processingOptions: ProcessingOptions,
    subsequentAuthInformation: SubsequentAuthInformation,
    authorizationIndicatorType: AuthorizationIndicatorType,
}

#[derive(Deserialize, Serialize)]
struct AuthorizationIndicatorType {
    authorizationIndicator: String,
}

#[derive(Deserialize, Serialize)]
struct Address {
    firstName: String,
    lastName: String,
    company: String,
    address: String,
    city: String,
    state: String,
    zip: String,
    country: String,
}

#[derive(Deserialize, Serialize)]
struct Customer {
    id: String,
}

#[derive(Deserialize, Serialize)]
struct Fee {
    amount: String,
    name: String,
    description: String,
}

#[derive(Deserialize, Serialize)]
struct LineItems {
    lineItem: LineItem,
}

#[derive(Deserialize, Serialize)]
struct LineItem {
    itemId: String,
    name: String,
    description: String,
    quantity: String,
    unitPrice: String,
}

#[derive(Deserialize, Serialize)]
struct Payment {
    creditCard: CreditCard,
}

#[derive(Deserialize, Serialize)]
struct CreditCard {
    cardNumber: String,
    expirationDate: String,
    cardCode: String,
}

#[derive(Deserialize, Serialize)]
struct ProcessingOptions {
    isSubsequentAuth: String,
}

#[derive(Deserialize, Serialize)]
struct SubsequentAuthInformation {
    originalNetworkTransId: String,
    originalAuthAmount: String,
    reason: String,
}

#[derive(Deserialize, Serialize)]
struct TransactionSettings {
    setting: TransactionSetting,
}

#[derive(Deserialize, Serialize)]
struct TransactionSetting {
    settingName: String,
    settingValue: String,
}

#[derive(Deserialize, Serialize)]
struct UserFields {
    userField: Vec<UserField>,
}

#[derive(Deserialize, Serialize)]
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
            createTransactionRequest: CreateTransactionRequest {
                merchantAuthentication: MerchantAuthentication {
                    name: merchant_id,
                    transactionKey: transaction_key,
                },
                refId: ref_id,
                transactionRequest: TransactionRequest {
                    transactionType: transaction_type,
                    amount: transaction_total,
                    payment: Payment {
                        creditCard: CreditCard {
                            cardCode: credit_card_cvv,
                            cardNumber: credit_card_number,
                            expirationDate: credit_card_exp,
                        },
                    },
                    lineItems: vec![],
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
                    poNumber: po_number,
                    customer: Customer { id: customer_id },
                    billTo: Address {
                        address: "123 Main St".to_string(),
                        city: "Lehi".to_string(),
                        company: "".to_string(),
                        country: "US".to_string(),
                        firstName: "Jamie".to_string(),
                        lastName: "Son".to_string(),
                        state: "UT".to_string(),
                        zip: "84043".to_string(),
                    },
                    shipTo: Address {
                        address: "123 Main St".to_string(),
                        city: "Lehi".to_string(),
                        company: "".to_string(),
                        country: "US".to_string(),
                        firstName: "Jamie".to_string(),
                        lastName: "Son".to_string(),
                        state: "UT".to_string(),
                        zip: "84043".to_string(),
                    },
                    customerIP: customer_ip,
                    transactionSettings: TransactionSettings {
                        setting: TransactionSetting {
                            settingName: "testRequest".to_string(),
                            settingValue: "false".to_string(),
                        },
                    },
                    userFields: UserFields { userField: vec![] },
                    processingOptions: ProcessingOptions {
                        isSubsequentAuth: "true".to_string(),
                    },
                    subsequentAuthInformation: SubsequentAuthInformation {
                        originalAuthAmount: "45".to_string(),
                        originalNetworkTransId: "123456789NNNH".to_string(),
                        reason: "resubmission".to_string(),
                    },
                    authorizationIndicatorType: AuthorizationIndicatorType {
                        authorizationIndicator: "final".to_string(),
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
