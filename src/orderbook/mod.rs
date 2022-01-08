use std::{collections::HashMap, fmt::{Error, Display, Formatter}};

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
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Response::Acknowledge{user_id, order_id} => {
                write!(f, "A, {}, {}", user_id, order_id)
            },
            Response::Best { side, price, qty } => {
                write!(f, "B, {}, {}, {}", side, if *price == 0 { String::from("-") } else {price.to_string()}, if *qty == 0 {String::from("-")} else {qty.to_string()})
            },
            Response::Reject { user_id, order_id } => {
                write!(f, "R, {}, {}", user_id, order_id)
            },
            Response::Trade { buyer_id, buyer_order_id, seller_id, seller_order_id, price, qty } => {
                write!(f, "T, {}, {}, {}, {}, {}, {}", buyer_id, buyer_order_id, seller_id, seller_order_id, price, qty)
            }
        }
    }
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
    trade_active: bool,
}

impl OrderBook {
    pub fn new(ticker: &str, trade_active: bool) -> Self {
        OrderBook {
            max_bid: 0,
            min_ask: 0,
            ticker: String::from(ticker),
            asks: HashMap::new(),
            bids: HashMap::new(),
            trades: vec![],
            trade_active: trade_active,
        }
    }

    pub fn ticker(&self) -> &str {
        self.ticker.as_str()
    }

    pub fn bids(&self) -> usize {
        self.bids.len()
    }

    pub fn asks(&self) -> usize {
        self.asks.len()
    }

    fn new_order(&mut self, side: Side, order: Order) -> (Option<Response>, Option<Response>) {
        let mut res = (None, None);
        let price = order.price();

        match side {
            Side::Buy => {
                let entry = self.bids.entry(order.price()).or_insert(vec![]);

                if price >= self.min_ask && self.asks.len() > 0 {
                    if self.trade_active {
                        // Check if corresponding order in asks and can trade
                        if let Some(val) = self.asks.get_mut(&order.price()) {
                            match val.iter().enumerate().find_map(|(k, o)| {
                                if o.qty() == order.qty() {
                                    Some(k)
                                } else {
                                    None
                                }
                            }) {
                                Some(k) => {
                                    let ack = order.ack();
                                    let trade = Trade::new(order, val.remove(k));
                                    let trade_resp = trade.get_trade_response();

                                    self.trades.push(trade);
                                    if val.len() == 0 {
                                        self.asks.remove_entry(&price);
                                    }
                                    res = (Some(ack), Some(trade_resp));
                                }
                                None => {
                                    res = (Some(order.reject()), None);
                                }
                            }
                        }
                    } else {
                        res = (Some(order.reject()), None);
                    }
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

                if price <= self.max_bid && self.bids.len() > 0 {
                    // Check if corresponding order in asks and can trade
                    if self.trade_active {
                        if let Some(val) = self.bids.get_mut(&order.price()) {
                            match val.iter().enumerate().find_map(|(k, o)| {
                                if o.qty() == order.qty() {
                                    Some(k)
                                } else {
                                    None
                                }
                            }) {
                                Some(k) => {
                                    let ack = order.ack();
                                    let trade = Trade::new(val.remove(k), order);
                                    let trade_resp = trade.get_trade_response();

                                    self.trades.push(trade);
                                    if val.len() == 0 {
                                        self.bids.remove_entry(&price);
                                    }
                                    res = (Some(ack), Some(trade_resp));
                                }
                                None => {
                                    res = (Some(order.reject()), None);
                                }
                            }
                        }
                    } else {
                        res = (Some(order.reject()), None);
                    }
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
        let mut res = (Some(Response::Acknowledge { user_id, order_id }), None);
        let mut found = false;
        let (mut price, mut order_idx) = (0u32, 0usize);

        for (k, v) in self.asks.iter_mut() {
            let x_in_v = v.iter().enumerate().find_map(|(k, val)| {
                if val.user_id == user_id && val.order_id == order_id {
                    Some(k)
                } else {
                    None
                }
            });
            if let Some(idx) = x_in_v {
                price = *k;
                order_idx = idx;
                found = true;
                break;
            }
        }

        if found {
            let v = self.asks.get_mut(&price).unwrap();

            if v.len() == 1 {
                self.asks.remove_entry(&price);
                if price == self.min_ask {
                    match self.asks.keys().min() {
                        Some(k) => {
                            self.min_ask = *k;
                            res = (
                                Some(Response::Acknowledge { user_id, order_id }),
                                Some(self.asks.get_key_value(&k).unwrap().1[0].best(Side::Sell)),
                            );
                        }
                        None => {
                            self.min_ask = 0;
                            res = (
                                Some(Response::Acknowledge { user_id, order_id }),
                                Some(Response::Best{side: String::from("S"), price: 0, qty: 0}),
                            );
                        }
                    };
                }
            } else {
                // if one of best asks is canceled -> show best
                v.remove(order_idx);
                res = (
                    Some(Response::Acknowledge { user_id, order_id }),
                    Some(self.asks.get_key_value(&price).unwrap().1[0].best(Side::Sell)),
                );
            }
            res
        } else {
            for (k, v) in self.bids.iter_mut() {
                let x_in_v = v.iter().enumerate().find_map(|(k, val)| {
                    if val.user_id == user_id && val.order_id == order_id {
                        Some(k)
                    } else {
                        None
                    }
                });
                if let Some(idx) = x_in_v {
                    price = *k;
                    order_idx = idx;
                    found = true;
                    break;
                }
            }

            if found {
                let v = self.bids.get_mut(&price).unwrap();

                if v.len() == 1 {
                    self.bids.remove_entry(&price);
                    if price == self.max_bid {
                        match self.bids.keys().max() {
                            Some(k) => {
                                self.max_bid = *k;
                                res = (
                                    Some(Response::Acknowledge { user_id, order_id }),
                                    Some(self.bids.get_key_value(&k).unwrap().1[0].best(Side::Buy)),
                                );
                            }
                            None => {
                                self.max_bid = 0;
                                res = (
                                    Some(Response::Acknowledge { user_id, order_id }),
                                    Some(Response::Best{side: String::from("B"), price: 0, qty: 0}),
                                );
                            }
                        }
                    }
                } else {
                    // if one of best bids is canceled -> show best
                    v.remove(order_idx);
                    res = (
                        Some(Response::Acknowledge { user_id, order_id }),
                        Some(self.bids.get_key_value(&price).unwrap().1[0].best(Side::Buy)),
                    );
                }
            } else {
                res = (Some(Response::Reject { user_id, order_id }), None)
            }
            res
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

    macro_rules! add_new_order {
        ($ob:expr, $user_id:expr, $symbol:expr, $price:expr, $qty:expr, $side:expr, $order_id:expr) => {
            $ob.new_user_action(UserAction::NewOrder {
                user_id: $user_id,
                symbol: String::from($symbol),
                price: $price,
                qty: $qty,
                side: String::from($side),
                order_id: $order_id,
            })
        };
        ($ob:expr, $user_id:expr, $order_id:expr) => {
            $ob.new_user_action(UserAction::CancelOrder {
                user_id: $user_id,
                order_id: $order_id,
            })
        };
        ($ob:expr) => {
            $ob.new_user_action(UserAction::Flush)
        };
    }

    #[test]
    #[ignore]
    fn test_empty_orderbook() {
        let ob = OrderBook::new("TSLA", false);
        assert_eq!("TSLA", ob.ticker());
    }

    #[test]
    #[ignore]
    fn test_add_1_bid() {
        let mut ob = OrderBook::new("TSLA", false);

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
        let mut ob = OrderBook::new("TSLA", false);

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
        let mut ob = OrderBook::new("TSLA", false);

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
    fn test_scenario_1() {
        // #name: scenario 1
        // #descr:balanced book
        let mut ob = OrderBook::new("IBM", false);

        // # build book, TOB = 10/11
        // N, 1, IBM, 10, 100, B, 1
        // A, 1, 1
        // B, B, 10, 100
        let res1 = add_new_order!(ob, 1, "IBM", 10, 100, "B", 1);
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

        // N, 1, IBM, 12, 100, S, 2
        // A, 1, 2
        // B, S, 12, 100
        let res2 = add_new_order!(ob, 1, "IBM", 12, 100, "S", 2);
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

        // N, 2, IBM, 9, 100, B, 101
        // A, 2, 101
        let res3 = add_new_order!(ob, 2, "IBM", 9, 100, "B", 101);
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res3.1, None);

        // N, 2, IBM, 11, 100, S, 102
        // A, 2, 102
        // B, S, 11, 100
        let res4 = add_new_order!(ob, 2, "IBM", 11, 100, "S", 102);
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

        // # hit book on each side, generate reject
        // N, 1, IBM, 11, 100, B, 3
        // R, 1, 3
        let res5 = add_new_order!(ob, 1, "IBM", 11, 100, "B", 3);
        assert_eq!(
            res5.0,
            Some(Response::Reject {
                user_id: 1,
                order_id: 3
            })
        );
        assert_eq!(res5.1, None);

        // N, 2, IBM, 10, 100, S, 103
        // R, 2, 103
        let res6 = add_new_order!(ob, 2, "IBM", 10, 100, "S", 103);
        assert_eq!(
            res6.0,
            Some(Response::Reject {
                user_id: 2,
                order_id: 103
            })
        );
        assert_eq!(res6.1, None);

        // # replenish book on each side, TOB = 10/11
        // N, 1, IBM, 10, 100, B, 4
        // A, 1, 4
        // B, B, 10, 200
        let res7 = add_new_order!(ob, 1, "IBM", 10, 100, "B", 4);
        assert_eq!(
            res7.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 4
            })
        );
        assert_eq!(
            res7.1,
            Some(Response::Best {
                side: String::from("B"),
                price: 10,
                qty: 200
            })
        );

        // N, 2, IBM, 11, 100, S, 104
        // A, 2, 104
        // B, S, 11, 200
        let res8 = add_new_order!(ob, 2, "IBM", 11, 100, "S", 104);
        assert_eq!(
            res8.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 104
            })
        );
        assert_eq!(
            res8.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 11,
                qty: 200
            })
        );

        // F
        let res9 = add_new_order!(ob);
        assert_eq!(res9, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }

    #[test]
    fn test_scenario_2() {
        // #name: scenario 2
        // #descr: shallow bid
        let mut ob = OrderBook::new("AAPL", false);

        // # build book, shallow bid, TOB = 10/11
        // N, 1, AAPL, 10, 100, B, 1
        // A, 1, 1
        // B, B, 10, 100
        let res1 = add_new_order!(ob, 1, "AAPL", 10, 100, "B", 1);
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

        // N, 1, AAPL, 12, 100, S, 2
        // A, 1, 2
        // B, S, 12, 100
        let res2 = add_new_order!(ob, 1, "AAPL", 12, 100, "S", 2);
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

        // N, 2, AAPL, 11, 100, S, 102
        // A, 2, 102
        // B, S, 11, 100
        let res3 = add_new_order!(ob, 2, "AAPL", 11, 100, "S", 102);
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 102
            })
        );
        assert_eq!(
            res3.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 11,
                qty: 100
            })
        );

        // # hit bid, generate reject
        // N, 2, AAPL, 10, 100, S, 103
        // R, 2, 103
        let res4 = add_new_order!(ob, 2, "AAPL", 10, 100, "S", 103);
        assert_eq!(
            res4.0,
            Some(Response::Reject {
                user_id: 2,
                order_id: 103
            })
        );
        assert_eq!(res4.1, None);

        // #  increase volume to Bid TOB 10, 200
        // N, 1, AAPL, 10, 100, B, 3
        // A, 1, 3
        // B, B, 10, 200
        let res5 = add_new_order!(ob, 1, "AAPL", 10, 100, "B", 3);
        assert_eq!(
            res5.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 3
            })
        );
        assert_eq!(
            res5.1,
            Some(Response::Best {
                side: String::from("B"),
                price: 10,
                qty: 200
            })
        );

        // F
        let res6 = add_new_order!(ob);
        assert_eq!(res6, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }

    #[test]
    fn test_scenario_3() {
        // #name: scenario 3
        // #descr: shallow ask
        let mut ob = OrderBook::new("VAL", false);

        // # build book, shallow ask, TOB = 10/11
        // N, 1, VAL, 10, 100, B, 1
        // A, 1, 1
        // B, B, 10, 100
        let res1 = add_new_order!(ob, 1, "VAL", 10, 100, "B", 1);
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

        // N, 2, VAL, 9, 100, B, 101
        // A, 2, 101
        let res2 = add_new_order!(ob, 2, "VAL", 9, 100, "B", 101);
        assert_eq!(
            res2.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res2.1, None);

        // N, 2, VAL, 11, 100, S, 102
        // A, 2, 102
        // B, S, 11, 100
        let res3 = add_new_order!(ob, 2, "VAL", 11, 100, "S", 102);
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 102
            })
        );
        assert_eq!(
            res3.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 11,
                qty: 100
            })
        );

        // # hit ask, generate reject
        // N, 1, VAL, 11, 100, B, 2
        // R, 1, 2
        let res4 = add_new_order!(ob, 1, "VAL", 11, 100, "B", 2);
        assert_eq!(
            res4.0,
            Some(Response::Reject {
                user_id: 1,
                order_id: 2
            })
        );
        assert_eq!(res4.1, None);

        // # increase volume to Ask TOB 10, 200
        // N, 2, VAL, 11, 100, S, 103
        // A, 2, 103
        // B, S, 11, 200
        let res5 = add_new_order!(ob, 2, "VAL", 11, 100, "S", 103);
        assert_eq!(
            res5.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 103
            })
        );
        assert_eq!(
            res5.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 11,
                qty: 200
            })
        );

        // F
        let res6 = add_new_order!(ob);
        assert_eq!(res6, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }

    #[test]
    fn test_scenario_4() {
        // #name: scenario 4
        // #descr: balanced book, limit below best bid
        let mut ob = OrderBook::new("IBM", false);

        // # build book, TOB = 10/11
        // N, 1, IBM, 10, 100, B, 1
        // A, 1, 1
        // B, B, 10, 100
        let res1 = add_new_order!(ob, 1, "IBM", 10, 100, "B", 1);
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

        // N, 1, IBM, 12, 100, S, 2
        // A, 1, 2
        // B, S, 12, 100
        let res2 = add_new_order!(ob, 1, "IBM", 12, 100, "S", 2);
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

        // N, 2, IBM, 9, 100, B, 101
        // A, 2, 101
        let res3 = add_new_order!(ob, 2, "IBM", 9, 100, "B", 101);
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res3.1, None);

        // N, 2, IBM, 11, 100, S, 102
        // A, 2, 102
        // B, S, 11, 100
        let res4 = add_new_order!(ob, 2, "IBM", 11, 100, "S", 102);
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

        // # limit below best bid, generate reject
        // N, 2, IBM, 9, 100, S, 103
        // R, 2, 103
        let res5 = add_new_order!(ob, 2, "IBM", 9, 100, "S", 103);
        assert_eq!(
            res5.0,
            Some(Response::Reject {
                user_id: 2,
                order_id: 103
            })
        );
        assert_eq!(res5.1, None);

        // F
        let res6 = add_new_order!(ob);
        assert_eq!(res6, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }

    #[test]
    fn test_scenario_5() {
        // #name: scenario 5
        // #descr: balanced book, limit above best ask
        let mut ob = OrderBook::new("IBM", false);

        // N, 1, IBM, 10, 100, B, 1
        // A, 1, 1
        // B, B, 10, 100
        let res1 = add_new_order!(ob, 1, "IBM", 10, 100, "B", 1);
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

        // N, 1, IBM, 12, 100, S, 2
        // A, 1, 2
        // B, S, 12, 100
        let res2 = add_new_order!(ob, 1, "IBM", 12, 100, "S", 2);
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

        // N, 2, IBM, 9, 100, B, 101
        // A, 2, 101
        let res3 = add_new_order!(ob, 2, "IBM", 9, 100, "B", 101);
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res3.1, None);

        // N, 2, IBM, 11, 100, S, 102
        // A, 2, 102
        // B, S, 11, 100
        let res4 = add_new_order!(ob, 2, "IBM", 11, 100, "S", 102);
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

        // # limit above best ask, generate reject
        // N, 1, IBM, 12, 100, B, 103
        // R, 1, 103
        let res5 = add_new_order!(ob, 1, "IBM", 12, 100, "B", 103);
        assert_eq!(
            res5.0,
            Some(Response::Reject {
                user_id: 1,
                order_id: 103
            })
        );
        assert_eq!(res5.1, None);

        // F
        let res6 = add_new_order!(ob);
        assert_eq!(res6, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }

    #[test]
    fn test_scenario_6() {
        // #name: scenario 6
        // #descr: tighten spread through new limit orders
        let mut ob = OrderBook::new("IBM", false);

        // # build book, TOB = 10/11
        // N, 1, IBM, 10, 100, B, 1
        // A, 1, 1
        // B, B, 10, 100
        let res1 = add_new_order!(ob, 1, "IBM", 10, 100, "B", 1);
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

        // N, 1, IBM, 16, 100, S, 2
        // A, 1, 2
        // B, S, 16, 100
        let res2 = add_new_order!(ob, 1, "IBM", 16, 100, "S", 2);
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
                price: 16,
                qty: 100
            })
        );

        // N, 2, IBM, 9, 100, B, 101
        // A, 2, 101
        let res3 = add_new_order!(ob, 2, "IBM", 9, 100, "B", 101);
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res3.1, None);

        // N, 2, IBM, 15, 100, S, 102
        // A, 2, 102
        // B, S, 15, 100
        let res4 = add_new_order!(ob, 2, "IBM", 15, 100, "S", 102);
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
                price: 15,
                qty: 100
            })
        );

        // # new bid, ask TOB = 11/14
        // N, 2, IBM, 11, 100, B, 103
        // A, 2, 103
        // B, B, 11, 100
        let res5 = add_new_order!(ob, 2, "IBM", 11, 100, "B", 103);
        assert_eq!(
            res5.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 103
            })
        );
        assert_eq!(
            res5.1,
            Some(Response::Best {
                side: String::from("B"),
                price: 11,
                qty: 100
            })
        );

        // N, 1, IBM, 14, 100, S, 3
        // A, 1, 3
        // B, S, 14, 100
        let res6 = add_new_order!(ob, 1, "IBM", 14, 100, "S", 3);
        assert_eq!(
            res6.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 3
            })
        );
        assert_eq!(
            res6.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 14,
                qty: 100
            })
        );

        // F
        let res7 = add_new_order!(ob);
        assert_eq!(res7, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }

    #[test]
    fn test_scenario_7() {
        // #name: scenario 7
        // #descr: balanced book, limit sell partial
        let mut ob = OrderBook::new("IBM", false);

        // # build book, TOB = 10/11
        // N, 1, IBM, 10, 100, B, 1
        // A, 1, 1
        // B, B, 10, 100
        let res1 = add_new_order!(ob, 1, "IBM", 10, 100, "B", 1);
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

        // N, 1, IBM, 12, 100, S, 2
        // A, 1, 2
        // B, S, 12, 100
        let res2 = add_new_order!(ob, 1, "IBM", 12, 100, "S", 2);
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

        // N, 2, IBM, 9, 100, B, 101
        // A, 2, 101
        let res3 = add_new_order!(ob, 2, "IBM", 9, 100, "B", 101);
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res3.1, None);

        // N, 2, IBM, 11, 100, S, 102
        // A, 2, 102
        // B, S, 11, 100
        let res4 = add_new_order!(ob, 2, "IBM", 11, 100, "S", 102);
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

        // # limit sell, generate reject
        // N, 2, IBM, 10, 20, S, 103
        // R, 2, 103
        let res5 = add_new_order!(ob, 2, "IBM", 10, 20, "S", 103);
        assert_eq!(
            res5.0,
            Some(Response::Reject {
                user_id: 2,
                order_id: 103
            })
        );
        assert_eq!(res5.1, None);

        // F
        let res6 = add_new_order!(ob);
        assert_eq!(res6, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }

    #[test]
    fn test_scenario_8() {
        // #name: scenario 8
        // #descr: balanced book, limit buy partial
        let mut ob = OrderBook::new("IBM", false);

        // # build book, TOB = 10/11
        // N, 1, IBM, 10, 100, B, 1
        // A, 1, 1
        // B, B, 10, 100
        let res1 = add_new_order!(ob, 1, "IBM", 10, 100, "B", 1);
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

        // N, 1, IBM, 12, 100, S, 2
        // A, 1, 2
        // B, S, 12, 100
        let res2 = add_new_order!(ob, 1, "IBM", 12, 100, "S", 2);
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

        // N, 2, IBM, 9, 100, B, 101
        // A, 2, 101
        let res3 = add_new_order!(ob, 2, "IBM", 9, 100, "B", 101);
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res3.1, None);

        // N, 2, IBM, 11, 100, S, 102
        // A, 2, 102
        // B, S, 11, 100
        let res4 = add_new_order!(ob, 2, "IBM", 11, 100, "S", 102);
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

        // # limit buy, generate reject
        // N, 1, IBM, 11, 20, B, 3
        // R, 1, 3
        let res5 = add_new_order!(ob, 1, "IBM", 11, 20, "B", 3);
        assert_eq!(
            res5.0,
            Some(Response::Reject {
                user_id: 1,
                order_id: 3
            })
        );
        assert_eq!(res5.1, None);

        // F
        let res6 = add_new_order!(ob);
        assert_eq!(res6, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }

    #[test]
    fn test_scenario_9() {
        // #name: scenario 9
        // #descr: balanced book, cancel best bid and offer
        let mut ob = OrderBook::new("IBM", false);

        // # build book, TOB = 10/11
        // N, 1, IBM, 10, 100, B, 1
        // A, 1, 1
        // B, B, 10, 100
        let res1 = add_new_order!(ob, 1, "IBM", 10, 100, "B", 1);
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

        // N, 1, IBM, 12, 100, S, 2
        // A, 1, 2
        // B, S, 12, 100
        let res2 = add_new_order!(ob, 1, "IBM", 12, 100, "S", 2);
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

        // N, 2, IBM, 9, 100, B, 101
        // A, 2, 101
        let res3 = add_new_order!(ob, 2, "IBM", 9, 100, "B", 101);
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res3.1, None);

        // N, 2, IBM, 11, 100, S, 102
        // A, 2, 102
        // B, S, 11, 100
        let res4 = add_new_order!(ob, 2, "IBM", 11, 100, "S", 102);
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

        // # cancel best bid and offer
        // C, 1, 1
        // A, 1, 1
        // B, B, 9, 100
        let res5 = add_new_order!(ob, 1, 1);
        assert_eq!(
            res5.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 1
            })
        );
        assert_eq!(
            res5.1,
            Some(Response::Best {
                side: String::from("B"),
                price: 9,
                qty: 100
            })
        );

        // C, 2, 102
        // A, 2, 102
        // B, S, 12, 100
        let res6 = add_new_order!(ob, 2, 102);
        assert_eq!(
            res6.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 102
            })
        );
        assert_eq!(
            res6.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 12,
                qty: 100
            })
        );

        // F
        let res7 = add_new_order!(ob);
        assert_eq!(res7, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }

    #[test]
    fn test_scenario_13() {
        // #name: scenario 5
        // #descr: balanced book, limit above best ask
        let mut ob = OrderBook::new("IBM", true);

        // N, 1, IBM, 10, 100, B, 1
        // A, 1, 1
        // B, B, 10, 100
        let res1 = add_new_order!(ob, 1, "IBM", 10, 100, "B", 1);
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

        // N, 1, IBM, 12, 100, S, 2
        // A, 1, 2
        // B, S, 12, 100
        let res2 = add_new_order!(ob, 1, "IBM", 12, 100, "S", 2);
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

        // N, 2, IBM, 9, 100, B, 101
        // A, 2, 101
        let res3 = add_new_order!(ob, 2, "IBM", 9, 100, "B", 101);
        assert_eq!(
            res3.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res3.1, None);

        // N, 2, IBM, 11, 100, S, 102
        // A, 2, 102
        // B, S, 11, 100
        let res4 = add_new_order!(ob, 2, "IBM", 11, 100, "S", 102);
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
        // # limit above best ask, generate reject
        // N, 1, IBM, 12, 100, B, 103
        // T, 1, 103, 2, 102, 11, 100
        // B, S, 12, 100
        let res5 = add_new_order!(ob, 1, "IBM", 12, 100, "B", 103);
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

        // F
        let res6 = add_new_order!(ob);
        assert_eq!(res6, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }
}
