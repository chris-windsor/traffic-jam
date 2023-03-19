use bigdecimal::BigDecimal;
use std::{io::stdin, str::FromStr};
use traffic_jam::*;

fn main() {
    let pool = create_pool();
    let conn = &mut pool.get().unwrap();

    let mut title = String::new();
    let mut stock = String::new();
    let mut price = String::new();

    println!("What is the name of this new product?");
    stdin()
        .read_line(&mut title)
        .expect("Unable to read title input");
    let title = title.trim();

    println!("How many of '{}' should we start with?", title);
    stdin()
        .read_line(&mut stock)
        .expect("Unable to read stock input");
    let stock: i32 = stock.trim().parse().expect("Unable to parse stock value");

    println!("What should the price of '{}' be?", title);
    stdin()
        .read_line(&mut price)
        .expect("Unable to read price input");
    let price: BigDecimal = BigDecimal::from_str(price.trim()).expect("Unable to parse price");

    let product = create_product(conn, title, &stock, &price);
    println!(
        "\nSaved '{}'(#{}), and set its stock level to {}",
        title, product.id, stock
    );
}
