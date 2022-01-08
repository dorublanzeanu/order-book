mod orderbook;
use orderbook::{Order, OrderBook, Response, UserAction};

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};

use regex::Regex;
use tokio::runtime::Runtime;

async fn produce_input(filename: &str, prod: &mut Arc<Mutex<Vec<UserAction>>>) {
    let file = File::open(filename).unwrap();
    let lines = BufReader::new(file).lines();

    println!("Produce");

    let new_order_re =
        Regex::new(r"^N, ([0-9]+), ([[:alpha:]]+), ([0-9]+), ([0-9]+), ([BS]), ([0-9]+)").unwrap();
    let cancel_order_re = Regex::new(r"^C, ([0-9]+), ([0-9]+)").unwrap();

    for line in lines {
        match line {
            Ok(line) => {
                println!("{}", line);
                if line.starts_with("N") {
                    let captures = new_order_re.captures(&line).unwrap();

                    prod.lock().ok().unwrap().push(UserAction::NewOrder {
                        user_id: captures.get(1).unwrap().as_str().parse::<u32>().unwrap(),
                        symbol: String::from(captures.get(2).unwrap().as_str()),
                        price: captures.get(3).unwrap().as_str().parse::<u32>().unwrap(),
                        qty: captures.get(4).unwrap().as_str().parse::<u32>().unwrap(),
                        side: String::from(captures.get(5).unwrap().as_str()),
                        order_id: captures.get(6).unwrap().as_str().parse::<u32>().unwrap(),
                    });
                } else if line.starts_with("C") {
                    let captures = cancel_order_re.captures(&line).unwrap();
                    prod.lock().ok().unwrap().push(UserAction::CancelOrder {
                        user_id: captures.get(1).unwrap().as_str().parse::<u32>().unwrap(),
                        order_id: captures.get(2).unwrap().as_str().parse::<u32>().unwrap(),
                    })
                } else if line.starts_with("F") {
                    prod.lock().ok().unwrap().push(UserAction::Flush);
                }
            }
            Err(_) => {}
        }
    }
}

async fn process(
    prod: &mut Arc<Mutex<Vec<UserAction>>>,
    cons: &mut Arc<Mutex<HashMap<String, OrderBook>>>,
    resp: &mut Arc<Mutex<Vec<(Option<Response>, Option<Response>)>>>,
) {
    println!("Process");
    while let Ok(v) = prod.lock().as_mut() {
        if v.len() > 0 {
            println!("Process actions: {}", v.len());
            let action = v.remove(0);

            if let Ok(val) = cons.lock().as_mut() {
                let entry = val
                    .entry(String::from("IBM"))
                    .or_insert(OrderBook::new("IBM", false));
                let response = entry.new_user_action(action);
                if let Ok(rsp) = resp.lock().as_mut() {
                    rsp.push(response);
                }
            }
        }
    }
}

async fn show_results(resp: &mut Arc<Mutex<Vec<(Option<Response>, Option<Response>)>>>) {
    println!("Results");
    while let Ok(v) = resp.lock().as_mut() {
        if v.len() > 0 {
            println!("{:?}", v.remove(0));
        }
    }
}

fn main() {
    let rt = Runtime::new().unwrap();

    let mut prod = Arc::new(Mutex::new(vec![]));
    let mut cons = Arc::new(Mutex::new(HashMap::new()));
    let mut resp: Arc<Mutex<Vec<(Option<Response>, Option<Response>)>>> =
        Arc::new(Mutex::new(vec![]));

    rt.block_on(async move {
        let mut prod_ref = Arc::clone(&prod);
        let mut resp_ref = Arc::clone(&resp);

        println!("hello from the async block");

        //bonus, you could spawn tasks too
        let produce_handle = tokio::spawn(async move {
            produce_input("./input/input.csv", &mut prod).await
        });
        let process_handle =
            tokio::spawn(async move { process(&mut prod_ref, &mut cons, &mut resp).await });
        let res_handle = tokio::spawn(async move { show_results(&mut resp_ref).await });

        let (r1, r2, r3) = tokio::join!(produce_handle, process_handle, res_handle);
    });
}
