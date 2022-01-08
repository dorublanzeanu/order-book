use std::{collections::HashMap, fmt::Error};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn new(s: String) -> Self {
        if s.starts_with("B") {
            Self::Buy
        } else {
            Self::Sell
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Response {
    Acknowledge {
        user_id: u32,
        order_id: u32,
    },
    Best {
        side: String,
        price: u32,
        qty: u32,
    },
    Reject {
        user_id: u32,
        order_id: u32,
    },
    Trade {
        buyer_id: u32,
        buyer_order_id: u32,
        seller_id: u32,
        seller_order_id: u32,
        price: u32,
        qty: u32,
    },
    Nil,
}

pub enum UserAction {
    NewOrder {
        user_id: u32,
        symbol: String,
        price: u32,
        qty: u32,
        side: String,
        order_id: u32,
    },
    CancelOrder {
        user_id: u32,
        order_id: u32,
    },
    Flush,
}

#[derive(Debug)]
pub struct Order {
    user_id: u32,
    price: u32,
    qty: u32,
    order_id: u32,
}

impl Order {
    pub fn new(user_id: u32, price: u32, qty: u32, order_id: u32) -> Self {
        Order {
            user_id,
            price,
            qty,
            order_id,
        }
    }

    pub fn price(&self) -> u32 {
        self.price
    }

    pub fn qty(&self) -> u32 {
        self.qty
    }

    pub fn best(&self, side: Side) -> Response {
        Response::Best {
            side: match side {
                Side::Buy => String::from("B"),
                Side::Sell => String::from("S"),
            },
            price: self.price,
            qty: self.qty,
        }
    }

    pub fn ack(&self) -> Response {
        Response::Acknowledge {
            user_id: self.user_id,
            order_id: self.order_id,
        }
    }

    pub fn reject(&self) -> Response {
        Response::Reject {
            user_id: self.user_id,
            order_id: self.order_id,
        }
    }
}

#[derive(Debug)]
/// Struct to keep records of trades
/// This struct is used only when a trade was made
pub struct Trade {
    buyer_id: u32,
    seller_id: u32,
    buyer_order_id: u32,
    seller_order_id: u32,
    price: u32,
    qty: u32,
}

impl Trade {
    /// Creates a Trade from two `Order`s
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

    pub fn get_trade_response(&self) -> Response {
        Response::Trade {
            buyer_id: self.buyer_id,
            buyer_order_id: self.buyer_order_id,
            seller_id: self.seller_id,
            seller_order_id: self.seller_order_id,
            price: self.price,
            qty: self.qty,
        }
    }
}

#[derive(Debug)]
pub struct OrderBook {
    max_bid: u32,
    min_ask: u32,
    ticker: String,
    asks: HashMap<u32, Vec<Order>>,
    bids: HashMap<u32, Vec<Order>>,
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

    fn new_order(&mut self, side: Side, order: Order) -> (Option<Response>, Option<Response>) {
        let mut res = (None, None);
        let price = order.price();

        match side {
            Side::Buy => {
                let entry = self.bids.entry(order.price()).or_insert(vec![]);

                if price > self.min_ask && self.asks.len() > 0 {
                    if self.trade_active {
                        // Check if corresponding order in asks and can trade
                        if let Some(val) = self.asks.get_mut(&order.price()) {
                            let ack = order.ack();
                            let trade = Trade::new(order, val.remove(0));
                            let trade_resp = trade.get_trade_response();

                            self.trades.push(trade);
                            if val.len() == 0 {
                                self.asks.remove_entry(&price);
                            }
                            res = (Some(ack), Some(trade_resp));
                        }
                    } else {
                        res = (Some(order.reject()), None);
                    }
                } else if price == self.min_ask {
                    // if prices is the same as best ask, reject
                    res = (Some(order.reject()), None);
                } else if price >= self.max_bid || self.max_bid == 0 {
                    // if none of the above check if best bid and
                    let ack = order.ack();
                    self.max_bid = price;
                    entry.push(order);
                    res = (
                        Some(ack),
                        Some(entry.iter().fold(
                            Response::Best {
                                side: String::from("B"),
                                price: 0,
                                qty: 0,
                            },
                            |acc, o| match acc {
                                Response::Best { side, price, qty } => Response::Best {
                                    side,
                                    price: o.price(),
                                    qty: qty + o.qty(),
                                },
                                _ => acc,
                            },
                        )),
                    );
                } else {
                    // if none of the above matches, ack order
                    res = (Some(order.ack()), None);
                    entry.push(order);
                }
            }
            Side::Sell => {
                let entry = self.asks.entry(order.price()).or_insert(vec![]);

                if price < self.max_bid && self.bids.len() > 0 {
                    // Check if corresponding order in asks and can trade
                    if let Some(val) = self.bids.get_mut(&order.price()) {
                        let ack = order.ack();
                        let trade = Trade::new(val.remove(0), order);
                        let trade_resp = trade.get_trade_response();

                        self.trades.push(trade);
                        if val.len() == 0 {
                            self.bids.remove_entry(&price);
                        }
                        res = (Some(ack), Some(trade_resp));
                    }
                } else if price == self.max_bid {
                    // if prices is the same as best ask, reject
                    res = (Some(order.reject()), None);
                } else if price <= self.min_ask || self.min_ask == 0 {
                    let ack = order.ack();
                    // if none of the above check if best bid
                    self.min_ask = price;
                    entry.push(order);
                    res = (
                        Some(ack),
                        Some(entry.iter().fold(
                            Response::Best {
                                side: String::from("S"),
                                price: 0,
                                qty: 0,
                            },
                            |acc, o| match acc {
                                Response::Best { side, price, qty } => Response::Best {
                                    side,
                                    price: o.price(),
                                    qty: qty + o.qty(),
                                },
                                _ => acc,
                            },
                        )),
                    );
                } else {
                    // if none of the above matches, ack order
                    res = (Some(order.ack()), None);
                    entry.push(order);
                }
            }
        }

        res
    }

    fn cancel_order(
        &mut self,
        user_id: u32,
        order_id: u32,
    ) -> (Option<Response>, Option<Response>) {
        let mut found = false;
        for (k, v) in self.asks.iter_mut() {
            let x_in_v = v.iter().enumerate().find_map(|(k, val)| {
                if val.user_id == user_id && val.order_id == order_id {
                    Some(k)
                } else {
                    None
                }
            });
            if let Some(k) = x_in_v {
                v.remove(k);
                found = true;
                break;
            }
        }

        if found {
            (Some(Response::Acknowledge { user_id, order_id }), None)
        } else {
            (Some(Response::Reject { user_id, order_id }), None)
        }
    }

    fn flush(&mut self) {
        self.max_bid = 0;
        self.min_ask = 0;
        self.ticker = String::from("");
        self.asks.clear();
        self.bids.clear();
    }

    pub fn new_user_action(&mut self, action: UserAction) -> (Option<Response>, Option<Response>) {
        match action {
            UserAction::NewOrder {
                user_id,
                symbol,
                price,
                qty,
                side,
                order_id,
            } => self.new_order(Side::new(side), Order::new(user_id, price, qty, order_id)),
            UserAction::CancelOrder { user_id, order_id } => self.cancel_order(user_id, order_id),
            UserAction::Flush => {
                self.flush();
                (None, None)
            }
        }
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

        ob.new_user_action(UserAction::NewOrder {
            user_id: 1,
            symbol: String::from("TSLA"),
            price: 10,
            qty: 100,
            side: String::from("B"),
            order_id: 1,
        });

        assert_eq!("TSLA", ob.ticker());
    }

    #[test]
    #[ignore]
    fn test_add_1_bid_1_ask() {
        let mut ob = OrderBook::new("TSLA");

        let res1 = ob.new_user_action(UserAction::NewOrder {
            user_id: 1,
            symbol: String::from("TSLA"),
            price: 10,
            qty: 100,
            side: String::from("B"),
            order_id: 1,
        });

        let res2 = ob.new_user_action(UserAction::NewOrder {
            user_id: 2,
            symbol: String::from("TSLA"),
            price: 10,
            qty: 100,
            side: String::from("S"),
            order_id: 1,
        });

        assert_eq!("TSLA", ob.ticker());
        assert_eq!(
            res1.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 1
            })
        );
        assert_eq!(
            res1.1,
            Some(Response::Best {
                side: String::from("B"),
                price: 10,
                qty: 100
            })
        );
        assert_eq!(
            res2.0,
            Some(Response::Reject {
                user_id: 2,
                order_id: 1
            })
        );
        assert_eq!(res2.0, None);
    }

    #[test]
    #[ignore]
    fn test_add_3_bids_verfy_sorted() {
        let mut ob = OrderBook::new("TSLA");

        let res1 = ob.new_user_action(UserAction::NewOrder {
            user_id: 1,
            symbol: String::from("TSLA"),
            price: 10,
            qty: 100,
            side: String::from("B"),
            order_id: 1,
        });

        let res2 = ob.new_user_action(UserAction::NewOrder {
            user_id: 2,
            symbol: String::from("TSLA"),
            price: 12,
            qty: 100,
            side: String::from("S"),
            order_id: 1,
        });

        let res3 = ob.new_user_action(UserAction::NewOrder {
            user_id: 3,
            symbol: String::from("TSLA"),
            price: 11,
            qty: 100,
            side: String::from("S"),
            order_id: 1,
        });

        println!("{:?}", ob);
        assert_eq!("TSLA", ob.ticker());
        assert_eq!(
            res1.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 1
            })
        );
        assert_eq!(
            res1.1,
            Some(Response::Best {
                side: String::from("B"),
                price: 10,
                qty: 100
            })
        );
        assert_eq!(
            res2.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 1
            })
        );
        assert_eq!(
            res2.1,
            Some(Response::Best {
                side: String::from("B"),
                price: 12,
                qty: 100
            })
        );
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 3,
                order_id: 1
            })
        );
        assert_eq!(res3.1, None);
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

        let res1 = ob.new_user_action(UserAction::NewOrder {
            user_id: 1,
            symbol: String::from("IBM"),
            price: 10,
            qty: 100,
            side: String::from("B"),
            order_id: 1,
        });
        assert_eq!(
            res1.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 1
            })
        );
        assert_eq!(
            res1.1,
            Some(Response::Best {
                side: String::from("B"),
                price: 10,
                qty: 100
            })
        );

        let res2 = ob.new_user_action(UserAction::NewOrder {
            user_id: 1,
            symbol: String::from("IBM"),
            price: 12,
            qty: 100,
            side: String::from("S"),
            order_id: 2,
        });
        assert_eq!(
            res2.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 2
            })
        );
        assert_eq!(
            res2.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 12,
                qty: 100
            })
        );

        let res3 = ob.new_user_action(UserAction::NewOrder {
            user_id: 2,
            symbol: String::from("IBM"),
            price: 9,
            qty: 100,
            side: String::from("B"),
            order_id: 101,
        });
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res3.1, None);

        let res4 = ob.new_user_action(UserAction::NewOrder {
            user_id: 2,
            symbol: String::from("IBM"),
            price: 11,
            qty: 100,
            side: String::from("S"),
            order_id: 102,
        });
        assert_eq!(
            res4.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 102
            })
        );
        assert_eq!(
            res4.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 11,
                qty: 100
            })
        );

        // If trading is ON
        let res5 = ob.new_user_action(UserAction::NewOrder {
            user_id: 1,
            symbol: String::from("IBM"),
            price: 12,
            qty: 100,
            side: String::from("B"),
            order_id: 103,
        });
        assert_eq!(
            res5.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 103
            })
        );
        assert_eq!(
            res5.1,
            Some(Response::Trade {
                buyer_id: 1,
                buyer_order_id: 103,
                seller_id: 1,
                seller_order_id: 2,
                price: 12,
                qty: 100
            })
        );

        println!("{:?}", ob);
    }
}
