use rand::Rng;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{signal, time::sleep};

#[derive(Clone)]
struct Item {
    name: String,
    qty: u128,
}

#[derive(Clone)]
struct Inventory {
    items: HashMap<String, Item>,
    held: HashMap<u8, Vec<Item>>,
}

#[derive(Clone)]
struct Order {
    id: u8,
    items: Vec<Item>,
}

impl Inventory {
    fn hold_items(&mut self, order: &Order) -> bool {
        let mut clone_inventory = self.items.clone();
        for order_item in &order.items {
            match clone_inventory.get_mut(&order_item.name) {
                Some(item) => {
                    if item.qty >= order_item.qty {
                        item.qty -= order_item.qty;
                    } else {
                        println!(
                            "| Order id{} over requested\n -{} has qty of {} but order is requesting qty of {}",
                            order.id, item.name, item.qty, order_item.qty
                        );
                        return false;
                    }
                }
                None => {
                    panic!("Could not find item with name {}", order_item.name);
                }
            }
        }

        self.items = clone_inventory;
        self.held.insert(order.id, order.items.clone());
        true
    }

    fn undo_hold(&mut self, order_id: &u8) {
        for held_item in self.held.get(order_id).unwrap() {
            match self.items.get_mut(&held_item.name) {
                Some(item) => item.qty += held_item.qty,
                None => {
                    panic!("Could not find item with name {}", held_item.name);
                }
            }
        }
    }

    fn release_order(&mut self, order_id: &u8) {
        self.held.remove(order_id);
    }

    fn log_inventory(&self) {
        println!("\n| Current Inventory");
        for (item_name, item) in &self.items {
            println!("- {} has a qty of {}", item_name, item.qty);
        }
        println!("\n");
    }
}

#[tokio::main]
async fn main() {
    let store_inventory: Inventory = Inventory {
        held: HashMap::from([]),
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

    store_inventory.log_inventory();

    let working_inventory = Arc::new(Mutex::new(store_inventory));

    for order in orders {
        proces_order(working_inventory.clone(), order).await;
    }

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}

async fn proces_order(inventory: Arc<Mutex<Inventory>>, order: Order) {
    println!("processing order #{}", order.id);
    tokio::spawn(async move {
        if inventory.lock().unwrap().hold_items(&order) {
            if collect_payment(&order).await {
                inventory.lock().unwrap().release_order(&order.id);
                println!("Sucessfully collected payment for order #{}", order.id);
                inventory.lock().unwrap().log_inventory();
            } else {
                inventory.lock().unwrap().undo_hold(&order.id);
                println!("Error while collecting payment for order #{}", order.id);
            }
        }
    });
}

async fn collect_payment(order: &Order) -> bool {
    println!("Collecting payment for order {}", order.id);
    let sleep_time = Duration::from_secs(rand::thread_rng().gen_range(1..=3));
    sleep(sleep_time).await;
    let payment_status = rand::thread_rng().gen_bool(0.7);
    payment_status
}
