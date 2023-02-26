use rand::Rng;
use std::{collections::HashMap, time::Duration};
use tokio::{signal, time::sleep};

#[derive(Clone)]
struct Item {
    name: String,
    qty: u128,
}

#[derive(Clone)]
struct Inventory {
    items: HashMap<String, Item>,
}

#[derive(Clone)]
struct Order {
    id: u8,
    items: Vec<Item>,
}

#[tokio::main]
async fn main() {
    let mut store_inventory: Inventory = Inventory {
        items: HashMap::from([
            (
                "item1".to_string(),
                Item {
                    name: "item1".to_string(),
                    qty: 4,
                },
            ),
            (
                "item2".to_string(),
                Item {
                    name: "item2".to_string(),
                    qty: 3,
                },
            ),
        ]),
    };

    let orders: Vec<Order> = vec![
        Order {
            id: 1,
            items: vec![
                Item {
                    name: "item1".to_string(),
                    qty: 3,
                },
                Item {
                    name: "item2".to_string(),
                    qty: 2,
                },
            ],
        },
        Order {
            id: 2,
            items: vec![
                Item {
                    name: "item1".to_string(),
                    qty: 1,
                },
                Item {
                    name: "item2".to_string(),
                    qty: 2,
                },
            ],
        },
        Order {
            id: 3,
            items: vec![Item {
                name: "item1".to_string(),
                qty: 1,
            }],
        },
    ];

    log_inventory(&store_inventory);

    for order in orders {
        proces_order(&mut store_inventory, order).await;
        log_inventory(&store_inventory);
    }

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}

fn log_inventory(inventory: &Inventory) {
    println!("\n| Current Inventory");
    for (item_name, item) in &inventory.items {
        println!("– {} has a qty of {}", item_name, item.qty);
    }
    println!("\n");
}

async fn proces_order(inventory: &mut Inventory, order: Order) {
    println!("processing order #{}", order.id);
    let taken_inventory = hold_inventory(inventory, &order);
    if taken_inventory.items.len() > 0 {
        if collect_payment(&order).await {
            println!("Sucessfully collected payment for order #{}", order.id);
        } else {
            supply_inventory(inventory, &mut inventory_from_order(&order));
            println!("Error while collecting payment for order #{}", order.id);
        }
    }
}

fn hold_inventory(inventory: &mut Inventory, order: &Order) -> Inventory {
    let did_take = take_inventory(inventory, order);
    if did_take {
        inventory_from_order(order)
    } else {
        Inventory {
            items: HashMap::from([]),
        }
    }
}

fn take_inventory(inventory: &mut Inventory, order: &Order) -> bool {
    let mut taken_inventory = Inventory {
        items: HashMap::from([]),
    };

    for order_item in &order.items {
        match inventory.items.get_mut(&order_item.name) {
            Some(inventory_item) => {
                if inventory_item.qty >= order_item.qty {
                    inventory_item.qty -= order_item.qty;
                    taken_inventory
                        .items
                        .insert(order_item.name.clone(), order_item.clone());
                } else {
                    println!("| Returning inventory for order #{}. \n| {} has qty of {} but order is requesting qty of {}", 
                        order.id, inventory_item.name, inventory_item.qty, order_item.qty);
                    supply_inventory(inventory, &mut taken_inventory);
                    return false;
                }
            }
            None => {
                panic!("Could not find inventory item named {}", order_item.name);
            }
        }
    }

    true
}

fn inventory_from_order(order: &Order) -> Inventory {
    let mut new_inventory = Inventory {
        items: HashMap::from([]),
    };

    for order_item in &order.items {
        new_inventory
            .items
            .insert(order_item.name.clone(), order_item.clone());
    }

    new_inventory
}

fn supply_inventory(inventory: &mut Inventory, new_inventory: &mut Inventory) {
    for (new_item_name, new_item) in &new_inventory.items {
        match inventory.items.get_mut(new_item_name) {
            Some(inventory_item) => inventory_item.qty += new_item.qty,
            None => {
                panic!("Could not find inventory item named {}", new_item_name);
            }
        }
    }
}

async fn collect_payment(order: &Order) -> bool {
    println!("Collecting payment for order {}", order.id);
    let sleep_time = Duration::from_secs(rand::thread_rng().gen_range(1..=3));
    sleep(sleep_time).await;
    let payment_status = rand::thread_rng().gen_bool(0.7);
    println!(
        "Payment status for order {} is: {}",
        order.id, payment_status
    );
    payment_status
}
