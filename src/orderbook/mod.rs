use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, PartialEq)]
pub enum Response {
    Acknowledge,
    Best,
    Reject,
    Trade,
}

#[derive(Debug)]
pub struct Order {
    side: Side,
    user_id: i64,
    order_id: i64,
    price: i64,
    qty: i64,
}

impl Order {
    pub fn new(side: Side, price: i64, qty: i64, user_id: i64, order_id: i64) -> Self {
        Order {
            side,
            user_id,
            order_id,
            price,
            qty,
        }
    }

    pub fn price(&self) -> i64 {
        self.price
    }

    pub fn side(&self) -> Side {
        self.side
    }
}

// pub trait OrderTrait {
//     fn bigger(&self, other: impl OrderTrait) -> bool;
//     fn equal(&self, other: impl OrderTrait) -> bool;
//     fn smaller(&self, other: impl OrderTrait) -> bool;
// }

// pub struct Ask {
//     price: i64,
// }

// pub struct Bid {
//     price: i64,
// }

// impl OrderTrait for Ask {
//     fn bigger(&self, other: impl OrderTrait) -> bool {
//         true
//     }
//     fn equal(&self, other: impl OrderTrait) -> bool {
//         true
//     }
//     fn smaller(&self, other: impl OrderTrait) -> bool {
//         true
//     }
// }

// impl OrderTrait for Bid {
//     fn bigger(&self, other: impl OrderTrait) -> bool {
//         true
//     }
//     fn equal(&self, other: impl OrderTrait) -> bool {
//         true
//     }
//     fn smaller(&self, other: impl OrderTrait) -> bool {
//         true
//     }
// }

#[derive(Debug)]
pub struct Trade {
    buyer_id: i64,
    seller_id: i64,
    buyer_order_id: i64,
    seller_order_id: i64,
    price: i64,
    qty: i64,
}

impl Trade {
    pub fn new(o1: Order, o2: Order) -> Self {
        Trade {
            buyer_id: o1.user_id,
            seller_id: o2.user_id,
            buyer_order_id: o1.order_id,
            seller_order_id: o2.order_id,
            price: o1.price,
            qty: o1.qty,
        }
    }
}

#[derive(Debug)]
pub struct OrderBook {
    max_bid: i64,
    min_ask: i64,
    ticker: String,
    /// Ordered vector of asks
    asks: HashMap<i64, Vec<Order>>,
    /// Ordered vector of bids
    bids: HashMap<i64, Vec<Order>>,
    trades: Vec<Trade>,
}

impl OrderBook {
    pub fn new(ticker: &str) -> Self {
        OrderBook {
            max_bid: 0,
            min_ask: 0,
            ticker: String::from(ticker),
            asks: HashMap::new(),
            bids: HashMap::new(),
            trades: vec![],
        }
    }

    pub fn ticker(&self) -> &str {
        self.ticker.as_str()
    }

    pub fn new_order(&mut self, order: Order) -> Response {
        let mut res = Response::Reject;
        let price = order.price();

        match order.side() {
            Side::Buy => {
                let entry = self.bids.entry(order.price()).or_insert(vec![]);

                if price > self.min_ask && self.asks.len() > 0 {
                    // Check if corresponding order in asks and can trade
                    if let Some(val) = self.asks.get_mut(&order.price()) {
                        self.trades.push(Trade::new(order, val.remove(0)));
                        if val.len() == 0 {
                            self.asks.remove_entry(&price);
                        }
                        res = Response::Trade;
                    }
                } else if price == self.min_ask {
                    // if prices is the same as best ask, reject
                    res = Response::Reject;
                } else if price > self.max_bid || self.max_bid == 0 {
                    // if none of the above check if best bid
                    self.max_bid = price;
                    res = Response::Best;
                    entry.push(order);
                } else {
                    // if none of the above matches, ack order
                    res = Response::Acknowledge;
                    entry.push(order);
                }
            }
            Side::Sell => {
                let entry = self.asks.entry(order.price()).or_insert(vec![]);

                if price < self.max_bid && self.bids.len() > 0 {
                    // Check if corresponding order in asks and can trade
                    if let Some(val) = self.bids.get_mut(&order.price()) {
                        self.trades.push(Trade::new(order, val.remove(0)));
                        if val.len() == 0 {
                            self.bids.remove_entry(&price);
                        }
                        res = Response::Trade;
                    }
                } else if price == self.max_bid {
                    // if prices is the same as best ask, reject
                    res = Response::Reject;
                } else if price < self.min_ask || self.min_ask == 0 {
                    // if none of the above check if best bid
                    self.min_ask = price;
                    res = Response::Best;
                    entry.push(order);
                } else {
                    // if none of the above matches, ack order
                    res = Response::Acknowledge;
                    entry.push(order);
                }
            }
        }

        res
    }
}

mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_empty_orderbook() {
        let ob = OrderBook::new("TSLA");
        assert_eq!("TSLA", ob.ticker());
    }

    #[test]
    #[ignore]
    fn test_add_1_bid() {
        let mut ob = OrderBook::new("TSLA");

        ob.new_order(Order {
            side: Side::Buy,
            price: 10,
            qty: 100,
            user_id: 1,
            order_id: 1,
        });

        assert_eq!("TSLA", ob.ticker());
    }

    #[test]
    #[ignore]
    fn test_add_1_bid_1_ask() {
        let mut ob = OrderBook::new("TSLA");

        ob.new_order(Order {
            side: Side::Buy,
            price: 10,
            qty: 100,
            user_id: 1,
            order_id: 1,
        });
        ob.new_order(Order {
            side: Side::Sell,
            price: 10,
            qty: 100,
            user_id: 1,
            order_id: 1,
        });

        assert_eq!("TSLA", ob.ticker());
    }

    #[test]
    #[ignore]
    fn test_add_3_bids_verfy_sorted() {
        let mut ob = OrderBook::new("TSLA");
        ob.new_order(Order {
            side: Side::Buy,
            price: 10,
            qty: 100,
            user_id: 1,
            order_id: 1,
        });
        ob.new_order(Order {
            side: Side::Buy,
            price: 12,
            qty: 100,
            user_id: 1,
            order_id: 1,
        });
        ob.new_order(Order {
            side: Side::Buy,
            price: 11,
            qty: 100,
            user_id: 1,
            order_id: 1,
        });
        println!("{:?}", ob);
        assert_eq!("TSLA", ob.ticker());
    }

    #[test]
    fn test_scenario_5() {
        // N, 1, IBM, 10, 100, B, 1
        // N, 1, IBM, 12, 100, S, 2
        // N, 2, IBM, 9, 100, B, 101
        // N, 2, IBM, 11, 100, S, 102
        //
        // # limit above best ask, generate reject
        // N, 1, IBM, 12, 100, B, 103

        // A, 1, 1
        // B, B, 10, 100
        // A, 1, 2
        // B, S, 12, 100
        // A, 2, 101
        // A, 2, 102
        // B, S, 11, 100
        // R, 1, 103
        let mut ob = OrderBook::new("IBM");
        assert_eq!(
            Response::Best,
            ob.new_order(Order {
                side: Side::Buy,
                price: 10,
                qty: 100,
                user_id: 1,
                order_id: 1,
            })
        );
        assert_eq!(
            Response::Best,
            ob.new_order(Order {
                side: Side::Sell,
                price: 12,
                qty: 100,
                user_id: 1,
                order_id: 2,
            })
        );
        assert_eq!(
            Response::Acknowledge,
            ob.new_order(Order {
                side: Side::Buy,
                price: 9,
                qty: 100,
                user_id: 2,
                order_id: 101,
            })
        );
        assert_eq!(
            Response::Best,
            ob.new_order(Order {
                side: Side::Sell,
                price: 11,
                qty: 100,
                user_id: 2,
                order_id: 102,
            })
        );
        assert_eq!(
            Response::Trade,
            ob.new_order(Order {
                side: Side::Buy,
                price: 12,
                qty: 100,
                user_id: 1,
                order_id: 103,
            })
        );

        println!("{:?}", ob);
    }
}
