use rand::Rng;
use std::{thread, time::Duration};

#[derive(Clone)]
struct Item {
    name: String,
    qty: u128,
}

#[derive(Clone)]
struct Inventory {
    items: Vec<Item>,
}

#[derive(Clone)]
struct Order {
    id: u8,
    items: Vec<Item>,
}

fn main() {
    let mut store_inventory: Inventory = Inventory {
        items: vec![
            Item {
                name: "item1".to_string(),
                qty: 4,
            },
            Item {
                name: "item2".to_string(),
                qty: 3,
            },
        ],
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
        proces_order(&mut store_inventory, order);
        log_inventory(&store_inventory);
    }
}

fn log_inventory(inventory: &Inventory) {
    println!("\n| Current Inventory");
    for item in &inventory.items {
        println!("– {} has a qty of {}", item.name, item.qty);
    }
    println!("\n");
}

fn proces_order(inventory: &mut Inventory, order: Order) {
    println!("processing order #{}", order.id);
    let taken_inventory = hold_inventory(inventory, &order);
    if taken_inventory.items.len() > 0 {
        if collect_payment() {
            println!("Sucessfully collected payment for order #{}", order.id);
        } else {
            println!("Error while collecting payment for order #{}", order.id);
            supply_inventory(inventory, &mut inventory_from_order(&order));
        }
    }
}

fn hold_inventory(inventory: &mut Inventory, order: &Order) -> Inventory {
    let did_take = take_inventory(inventory, order);
    if did_take {
        inventory_from_order(order)
    } else {
        Inventory { items: vec![] }
    }
}

fn take_inventory(inventory: &mut Inventory, order: &Order) -> bool {
    let mut taken_inventory = Inventory { items: vec![] };

    for inventory_item in inventory.items.iter_mut() {
        for order_item in &order.items {
            if inventory_item.name == order_item.name {
                if inventory_item.qty >= order_item.qty {
                    inventory_item.qty -= order_item.qty;
                    taken_inventory.items.push(order_item.clone());
                } else {
                    println!(
                        "| Returning inventory for order #{}. \n| {} has qty of {} but order is requesting qty of {}",
                        order.id, inventory_item.name, inventory_item.qty, order_item.qty
                    );
                    supply_inventory(inventory, &mut taken_inventory);
                    return false;
                }
            }
        }
    }

    true
}

fn inventory_from_order(order: &Order) -> Inventory {
    let mut new_inventory = Inventory { items: vec![] };

    for order_item in &order.items {
        new_inventory.items.push(order_item.clone());
    }

    new_inventory
}

fn supply_inventory(inventory: &mut Inventory, new_inventory: &mut Inventory) {
    for inventory_item in inventory.items.iter_mut() {
        for new_item in new_inventory.items.iter_mut() {
            if inventory_item.name == new_item.name {
                inventory_item.qty += new_item.qty;
            }
        }
    }
}

fn collect_payment() -> bool {
    thread::sleep(Duration::from_secs(rand::thread_rng().gen_range(1..=3)));
    rand::thread_rng().gen_bool(0.7)
}
