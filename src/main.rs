mod orderbook;
use orderbook::{Order, OrderBook, Response, Side};

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};

use regex::Regex;
use tokio::runtime::Runtime;

async fn produce_input(filename: &str, prod: &mut Arc<Mutex<Vec<Order>>>) {
    let file = File::open(filename).unwrap();
    let lines = BufReader::new(file).lines();

    println!("Produce");

    let re =
        Regex::new(r"^N, ([0-9]+), ([[:alpha:]]+), ([0-9]+), ([0-9]+), ([BS]), ([0-9]+)").unwrap();

    for line in lines {
        match line {
            Ok(line) => {
                println!("{}", line);
                if line.starts_with("N") {
                    let captures = re.captures(&line).unwrap();

                    prod.lock().ok().unwrap().push(Order::new(
                        match captures.get(5).unwrap().as_str() {
                            "B" => Side::Buy,
                            "S" => Side::Sell,
                            _ => panic!("Incorrect line format"),
                        },
                        captures.get(3).unwrap().as_str().parse::<i64>().unwrap(),
                        captures.get(4).unwrap().as_str().parse::<i64>().unwrap(),
                        captures.get(1).unwrap().as_str().parse::<i64>().unwrap(),
                        captures.get(6).unwrap().as_str().parse::<i64>().unwrap(),
                    ))
                } else if line.starts_with("C") {
                } else if line.starts_with("F") {
                }
            }
            Err(_) => {}
        }
    }
}

async fn process(
    prod: &mut Arc<Mutex<Vec<Order>>>,
    cons: &mut Arc<Mutex<HashMap<String, OrderBook>>>,
    resp: &mut Arc<Mutex<Vec<Response>>>,
) {
    println!("Process");
    while let Ok(v) = prod.lock().as_mut() {
        if v.len() > 0 {
            println!("Process orders: {}", v.len());
            let order = v.remove(0);

            if let Ok(val) = cons.lock().as_mut() {
                let entry = val
                    .entry(String::from("IBM"))
                    .or_insert(OrderBook::new("IBM"));
                let response = entry.new_order(order);
                if let Ok(rsp) = resp.lock().as_mut() {
                    rsp.push(response);
                }
            }
        }
    }
}

async fn show_results(resp: &mut Arc<Mutex<Vec<Response>>>) {
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
    let mut resp: Arc<Mutex<Vec<Response>>> = Arc::new(Mutex::new(vec![]));

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
