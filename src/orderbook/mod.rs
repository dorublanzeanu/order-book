use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

/// This mod implements an orders book inner functionaity.
///
/// Provides an abstraction over two [HashMap]s that hold the orders
/// per each price.
///

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
/// This enum is an internal mod enum that describes the direction
/// of an order.
///
/// This enum is used internally to select between logic applied to
/// sell and buy orders.
pub(super) enum Side {
    Buy,
    Sell,
}

impl Side {
    /// Provides a way to get a [Side] from a [String]
    pub(super) fn new(s: String) -> Self {
        if s.starts_with('B') {
            Self::Buy
        } else {
            Self::Sell
        }
    }

    /// Returns one letter string used at output
    pub(super) fn get_one_letter_string(&self) -> String {
        match self {
            Self::Buy => "B".to_string(),
            Self::Sell => "S".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
/// This enum is a public enum that describes result of a [UserAction]
/// on the [OrderBook]
///
/// This enum's most important role is to abstract the output's needed
/// data format.
pub enum Response {
    /// This variant of [Response] enum is used to acknowledge a calid [UserAction]
    Acknowledge { user_id: u32, order_id: u32 },
    /// This variant of [Response] enum is used show the Top of Book has modified and
    /// there is a new Best
    Best { side: String, price: u32, qty: u32 },
    /// This variant of [Response] enum is used to reject a bad [UserAction]
    Reject { user_id: u32, order_id: u32 },
    /// This variant of [Response] enum signals there is a match of prices that produced
    /// a trade
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
    /// Implement the Display trait to easily print desired output format
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Response::Acknowledge { user_id, order_id } => {
                write!(f, "A, {}, {}", user_id, order_id)
            }
            Response::Best { side, price, qty } => {
                write!(
                    f,
                    "B, {}, {}, {}",
                    side,
                    if *price == 0 {
                        String::from("-")
                    } else {
                        price.to_string()
                    },
                    if *qty == 0 {
                        String::from("-")
                    } else {
                        qty.to_string()
                    }
                )
            }
            Response::Reject { user_id, order_id } => {
                write!(f, "R, {}, {}", user_id, order_id)
            }
            Response::Trade {
                buyer_id,
                buyer_order_id,
                seller_id,
                seller_order_id,
                price,
                qty,
            } => {
                write!(
                    f,
                    "T, {}, {}, {}, {}, {}, {}",
                    buyer_id, buyer_order_id, seller_id, seller_order_id, price, qty
                )
            }
        }
    }
}

/// This enum is a public enum that describes the possible [UserAction]s
/// on the [OrderBook]
///
/// This enum's most important role is to represent the input in a format
/// that the [OrderBook] can understand.
pub enum UserAction {
    /// This enum variant describes a new order that comes from an user
    NewOrder {
        user_id: u32,
        symbol: String,
        price: u32,
        qty: u32,
        side: String,
        order_id: u32,
    },
    /// This enum variant describes a cancel order from an user
    CancelOrder { user_id: u32, order_id: u32 },
    /// This enum variant describes a flush command that instructs the [OrderBook]
    /// to reset.
    Flush,
}

#[derive(Debug)]
/// This struct is a private struct used to represent an open [Order]
/// in the [OrderBook] asks/bids.
///
/// This struct holds a part of the order info because the reset of them,
/// such as [Side] is derived from the implementation
pub(super) struct Order {
    user_id: u32,
    price: u32,
    qty: u32,
    order_id: u32,
}

impl Order {
    /// Function to create an [Order] from raw data
    pub(super) fn new(user_id: u32, price: u32, qty: u32, order_id: u32) -> Self {
        Order {
            user_id,
            price,
            qty,
            order_id,
        }
    }

    /// Price getter
    pub(super) fn price(&self) -> u32 {
        self.price
    }

    /// Quantity geter
    pub(super) fn qty(&self) -> u32 {
        self.qty
    }

    /// Gets a [Response::Best] from the order which
    /// is returned to the user to signal that this order
    /// is Best of the Book
    pub(super) fn best(&self, side: Side) -> Response {
        Response::Best {
            side: match side {
                Side::Buy => String::from("B"),
                Side::Sell => String::from("S"),
            },
            price: self.price,
            qty: self.qty,
        }
    }

    /// Gets a [Response::Acknowledge] from the order which
    /// is returned to the user to signal that this order
    /// was received
    pub(super) fn ack(&self) -> Response {
        Response::Acknowledge {
            user_id: self.user_id,
            order_id: self.order_id,
        }
    }

    /// Gets a [Response::Acknowledge] from the order which
    /// is returned to the user to signal that this order
    /// was rejected
    pub(super) fn reject(&self) -> Response {
        Response::Reject {
            user_id: self.user_id,
            order_id: self.order_id,
        }
    }
}

#[derive(Debug)]
/// Struct to keep records of trades
/// This struct is used only when a trade is made
pub(super) struct Trade {
    buyer_id: u32,
    seller_id: u32,
    buyer_order_id: u32,
    seller_order_id: u32,
    price: u32,
    qty: u32,
}

impl Trade {
    /// Creates a Trade from two [Order]s by consuming the [Order]s
    pub(super) fn new(o1: Order, o2: Order) -> Self {
        Trade {
            buyer_id: o1.user_id,
            seller_id: o2.user_id,
            buyer_order_id: o1.order_id,
            seller_order_id: o2.order_id,
            price: o1.price,
            qty: o1.qty,
        }
    }

    /// Creates a [Response::Trade] to send to the user to signal the
    /// trade
    pub(super) fn get_trade_response(&self) -> Response {
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
/// This struct provides the needed functionality to create,
/// interact with an [OrderBook]
///
/// It hold information suchh as ask/bid [Order]s, trades, ticker.
///
/// # Examples
///
/// ```
/// use orderbook::{OrderBook, UserAction};
/// // Creates OrderBook - with Trading disabled
/// let mut ob = OrderBook::new("IBM", false);
///
/// // Add new Order
/// let response = ob.new_user_action(UserAction::NewOrder{
///     user_id: 1,
///     symbol: "IBM",
///     price: 10,
///     qty: 100,
///     side: "B",
///     order_id: 1,
/// });
/// assert_eq!(
///     response.0,
///     Some(Response::Acknowledge{user_id: 1, order_id: 1}
/// ));
/// assert_eq!(
///     response.1,
///     Some(Response::Best{side: "B".to_string(), user_id: 1, order_id: 1}
/// ));
/// ```
///
pub struct OrderBook {
    /// Maximum bid
    max_bid: u32,
    /// Minimum ask
    min_ask: u32,
    /// OrderBook's ticker for which holds orders
    ticker: String,
    /// [HashMap] with ask orders
    asks: HashMap<u32, Vec<Order>>,
    /// [HashMap] with bid orders
    bids: HashMap<u32, Vec<Order>>,
    /// [Vec] of [Trades] - this is empty if `trade_active` is `false`
    trades: Vec<Trade>,
    /// Enables trading functionality
    trade_active: bool,
}

impl OrderBook {
    /// Creates a new empty [OrderBook]
    pub fn new(ticker: &str, trade_active: bool) -> Self {
        OrderBook {
            max_bid: 0,
            min_ask: 0,
            ticker: String::from(ticker),
            asks: HashMap::new(),
            bids: HashMap::new(),
            trades: vec![],
            trade_active,
        }
    }

    #[allow(dead_code)]
    /// Ticker getter
    pub fn ticker(&self) -> &str {
        self.ticker.as_str()
    }

    #[allow(dead_code)]
    /// Returns number of current bids
    pub fn bids(&self) -> usize {
        self.bids.len()
    }

    #[allow(dead_code)]
    /// Returns number of current asks
    pub fn asks(&self) -> usize {
        self.asks.len()
    }

    /// Private method that tries to insert a new order for given collection
    /// TODO: In case an order which matches offer, implement a way to print
    /// Ack, Trade, Best
    /// As of now it only prins: Ack, Trade
    fn new_order_logic(
        // Collection in which to insert
        col_insert: &mut HashMap<u32, Vec<Order>>,
        // Collection in which to search equivalent offer
        col_search: &mut HashMap<u32, Vec<Order>>,
        // Vec of trades in case of need
        trades: &mut Vec<Trade>,
        // Best price of same time - competitors
        best: &mut u32,
        // Best opposite price - the offer
        best_opposite: &mut u32,
        order: Order,
        side: Side,
        trade_active: bool,
        // function to check whether price crosses book
        f: impl Fn(u32, u32) -> bool,
    ) -> (Option<Response>, Option<Response>) {
        let mut res = (None, None);
        let price = order.price();

        // Get Vec of orders corresponding with price in target insert collection
        let entry = col_insert.entry(order.price()).or_insert(vec![]);

        // if price crosses book and there are opposing offers
        if f(price, *best_opposite) && !col_search.is_empty() {
            // Check if corresponding order can trade
            if trade_active {
                // Get corresponding offer price order list
                if let Some(val) = col_search.get_mut(&order.price()) {
                    match val.iter().enumerate().find_map(|(k, o)| {
                        if o.qty() == order.qty() {
                            Some(k)
                        } else {
                            None
                        }
                    }) {
                        // If corresponding price exists in offers list
                        Some(k) => {
                            // Get ack response before consuming
                            let ack = order.ack();

                            // Get trade by consuming the 2 orders
                            let trade = match side {
                                Side::Buy => Trade::new(order, val.remove(k)),
                                Side::Sell => Trade::new(val.remove(k), order),
                            };

                            // Get trade response to return from this method
                            let trade_resp = trade.get_trade_response();

                            trades.push(trade);

                            // If removed equivalent last offer of this price
                            if val.is_empty() {
                                col_search.remove_entry(&price);

                                // Recalculate best opposite offer
                                *best_opposite = match side {
                                    Side::Buy => *col_search.keys().min().unwrap_or(&0),
                                    Side::Sell => *col_search.keys().max().unwrap_or(&0),
                                };

                            }
                            res = (Some(ack), Some(trade_resp));
                        }
                        // If corresponding price does not exist in offers list -> reject book crossing
                        None => {
                            res = (Some(order.reject()), None);
                        }
                    }
                }
            }
            // Reject Order - Trading not allowed
            else {
                res = (Some(order.reject()), None);
            }
        }
        // Check if new order exceeds current best
        else if f(price, *best) || *best == 0 {
            // Get ack response before consuming order
            let ack = order.ack();

            // Change best price
            *best = price;

            // Push order to OrderBook
            entry.push(order);

            // Get Response with Best being Sum of quantities
            res = (
                Some(ack),
                Some(entry.iter().fold(
                    Response::Best {
                        side: side.get_one_letter_string(),
                        price: 0,
                        qty: 0,
                    },
                    |acc, o| match acc {
                        Response::Best {
                            side,
                            price: _,
                            qty,
                        } => Response::Best {
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

        res
    }

    /// Private method that tries to insert a new order
    fn new_order(&mut self, side: Side, order: Order) -> (Option<Response>, Option<Response>) {
        match side {
            // Side::Buy => self.new_buy_order(order),
            Side::Buy => Self::new_order_logic(
                &mut self.bids,
                &mut self.asks,
                &mut self.trades,
                &mut self.max_bid,
                &mut self.min_ask,
                order,
                Side::Buy,
                self.trade_active,
                |a, b| a >= b,
            ),
            Side::Sell => Self::new_order_logic(
                &mut self.asks,
                &mut self.bids,
                &mut self.trades,
                &mut self.min_ask,
                &mut self.max_bid,
                order,
                Side::Sell,
                self.trade_active,
                |a, b| a <= b,
            ),
        }
    }

    /// Search for Order in HashMap
    fn get_order_index(
        col: &HashMap<u32, Vec<Order>>,
        user_id: u32,
        order_id: u32,
    ) -> Option<(u32, usize)> {
        let mut res = None;

        // Search for the order in bids HashMap
        for (k, v) in col.iter() {
            // Call find for each price list of Orders to get the index in Vec
            let x_in_v = v.iter().enumerate().find_map(|(k, val)| {
                if val.user_id == user_id && val.order_id == order_id {
                    Some(k)
                } else {
                    None
                }
            });

            // If there is a valid index -> break
            if let Some(idx) = x_in_v {
                res = Some((*k, idx));
                break;
            }
        }

        res
    }

    /// This private method runs the logic to cancel orders
    fn cancel_order_logic(
        col: &mut HashMap<u32, Vec<Order>>,
        side: Side,
        user_id: u32,
        order_id: u32,
        best: &mut u32,
    ) -> (Option<Response>, Option<Response>) {
        let res;
        let found;
        let (price, order_idx) = match Self::get_order_index(col, user_id, order_id) {
            Some((x, y)) => {
                found = true;
                (x, y)
            }
            None => {
                found = false;
                (0, 0)
            }
        };

        // If the order was found in the asks eliminate it
        if found {
            // Get mutable reference to containing vec
            let v = col.get_mut(&price).unwrap();

            // Check if it is the only order -> eliminate the whole entry
            if v.len() == 1 {
                // Remove price entry from HashMap since it will be empty afterwards
                col.remove_entry(&price);

                // it means afterwards we will have a new min_ask
                if price == *best {
                    // find new best
                    let new_best = match side {
                        Side::Buy => col.keys().max(),
                        Side::Sell => col.keys().min(),
                    };

                    match new_best {
                        // if new best -> return (Ack, Best(Side, best, qty))
                        Some(k) => {
                            *best = *k;
                            res = (
                                Some(Response::Acknowledge { user_id, order_id }),
                                Some(col.get_key_value(k).unwrap().1[0].best(side)),
                            );
                        }
                        // if new best -> return (Ack, Best(Side, 0, 0))
                        None => {
                            *best = 0;
                            res = (
                                Some(Response::Acknowledge { user_id, order_id }),
                                Some(Response::Best {
                                    side: side.get_one_letter_string(),
                                    price: 0,
                                    qty: 0,
                                }),
                            );
                        }
                    };
                }
                // In case the order is beneath best
                else {
                    res = (Some(Response::Acknowledge { user_id, order_id }), None);
                }
            } else {
                // if one of best orders is canceled -> show best
                v.remove(order_idx);
                if price == *best {
                    res = (
                        Some(Response::Acknowledge { user_id, order_id }),
                        Some(col.get_key_value(&price).unwrap().1[0].best(side)),
                    );
                } else {
                    res = (Some(Response::Acknowledge { user_id, order_id }), None);
                }
            }
            res
        } else {
            (Some(Response::Reject { user_id, order_id }), None)
        }
    }

    /// Private method that tries to cancel an order
    fn cancel_order(
        &mut self,
        user_id: u32,
        order_id: u32,
    ) -> (Option<Response>, Option<Response>) {
        // Try cancelling from asks
        let res = Self::cancel_order_logic(
            &mut self.asks,
            Side::Sell,
            user_id,
            order_id,
            &mut self.min_ask,
        );
        match res {
            // if we get two responses (Ack, Best) -> means it is canceled and new best
            (Some(a), Some(b)) => (Some(a), Some(b)),
            // if we get one response (Ack, None) -> order cancelled
            (Some(Response::Acknowledge { user_id, order_id }), None) => {
                (Some(Response::Acknowledge { user_id, order_id }), None)
            }
            // for any other response it means the order is not found -> search in bids
            (_, _) => Self::cancel_order_logic(
                &mut self.bids,
                Side::Buy,
                user_id,
                order_id,
                &mut self.max_bid,
            ),
        }
    }

    /// Private method that flushes the [OrderBook]
    fn flush(&mut self) {
        self.max_bid = 0;
        self.min_ask = 0;
        self.ticker = String::from("");
        self.asks.clear();
        self.bids.clear();
    }

    /// Public method used to interract with the [OrderBook]
    ///
    /// This method translates the [UserAction] received as parameter
    /// to a suitable input dependng on the type of [UserAction]
    pub fn new_user_action(&mut self, action: UserAction) -> (Option<Response>, Option<Response>) {
        match action {
            UserAction::NewOrder {
                user_id,
                symbol: _,
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

#[cfg(test)]
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
    fn test_scenario_10() {
        // #name: scenario 10
        // #descr: balanced book, cancel behind best bid and offer
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
        // C, 1, 2
        // A, 1, 2
        let res5 = add_new_order!(ob, 1, 2);
        assert_eq!(
            res5.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 2
            })
        );
        assert_eq!(res5.1, None);

        // C, 2, 101
        // A, 2, 101
        let res6 = add_new_order!(ob, 2, 101);
        assert_eq!(
            res6.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(res6.1, None);

        // F
        let res7 = add_new_order!(ob);
        assert_eq!(res7, (None, None));
        assert_eq!(0, ob.asks());
        assert_eq!(0, ob.bids());
        assert_eq!("", ob.ticker());
    }

    #[test]
    fn test_scenario_11() {
        // #name: scenario 11
        // #descr: balanced book, cancel all bids
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

        // C, 2, 101
        // A, 2, 101
        // B, B, -, -
        let res6 = add_new_order!(ob, 2, 101);
        assert_eq!(
            res6.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 101
            })
        );
        assert_eq!(
            res6.1,
            Some(Response::Best {
                side: String::from("B"),
                price: 0,
                qty: 0
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
    fn test_scenario_12() {
        // #name: scenario 12
        // #descr: balanced book, TOB volume changes
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

        // # increase and decrease the TOB volume
        // N, 2, IBM, 11, 100, S, 103
        // A, 2, 103
        // B, S, 11, 200
        let res5 = add_new_order!(ob, 2, "IBM", 11, 100, "S", 103);
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

        // C, 2, 103
        // A, 2, 103
        // B, S, 11, 100
        let res6 = add_new_order!(ob, 2, 103);
        assert_eq!(
            res6.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 103
            })
        );
        assert_eq!(
            res6.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 11,
                qty: 100
            })
        );

        // # cancel all asks
        // C, 2, 102
        // A, 2, 102
        // B, S, 12, 100
        let res7 = add_new_order!(ob, 2, 102);
        assert_eq!(
            res7.0,
            Some(Response::Acknowledge {
                user_id: 2,
                order_id: 102
            })
        );
        assert_eq!(
            res7.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 12,
                qty: 100
            })
        );

        // C, 1, 2
        // A, 1, 2
        // B, S, -, -
        let res8 = add_new_order!(ob, 1, 2);
        assert_eq!(
            res8.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 2
            })
        );
        assert_eq!(
            res8.1,
            Some(Response::Best {
                side: String::from("S"),
                price: 0,
                qty: 0
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

    #[test]
    fn test_scenario_14() {
        // #name: scenario 3
        // #descr: shallow ask
        let mut ob = OrderBook::new("VAL", true);

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
        // T, 1, 2, 2, 102, 11, 100
        let res4 = add_new_order!(ob, 1, "VAL", 11, 100, "B", 2);
        assert_eq!(
            res4.0,
            Some(Response::Acknowledge {
                user_id: 1,
                order_id: 2
            })
        );
        assert_eq!(
            res4.1,
            Some(Response::Trade {
                buyer_id: 1,
                buyer_order_id: 2,
                seller_id: 2,
                seller_order_id: 102,
                price: 11,
                qty: 100
            })
        );

        // # increase volume to Ask TOB 10, 200
        // N, 2, VAL, 11, 100, S, 103
        // A, 2, 103
        // B, S, 11, 100
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
