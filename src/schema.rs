// @generated automatically by Diesel CLI.

diesel::table! {
    products (id) {
        id -> Int4,
        title -> Varchar,
        stock -> Int4,
        price -> Numeric,
    }
}
