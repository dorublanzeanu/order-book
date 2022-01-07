#[allow(dead_code)]

pub enum Side {
    Buy,
    Sell,
}

pub enum Response {
    Acknowledge,
    Best,
    Reject,
    Trade,
}

#[derive(Debug)]
struct Order {
    user_id: i64,
    order_id: i64,
    price: i64,
    qty: i64,
}

impl Order {
    pub fn new(price: i64, qty: i64, user_id: i64, order_id: i64) -> Self {
        Order {
            user_id,
            order_id,
            price,
            qty,
        }
    }

    pub fn price(&self) -> i64 {
        self.price
    }

}


#[derive(Debug)]
pub struct OrderBook {
    ticker: String,
    /// Ordered vector of asks
    asks: Vec<Order>,
    /// Ordered vector of bids
    bids: Vec<Order>,
}

impl OrderBook {
    pub fn new(ticker: &str) -> Self {
        OrderBook {
            ticker: String::from(ticker),
            asks: vec![],
            bids: vec![],
        }
    }

    pub fn ticker(&self) -> &str {
        self.ticker.as_str()
    }

    // maybe use binary search for performance here
    fn sorted_position(side: Side, v: &Vec<Order>, price: i64) -> usize {
        let mut pos = 0;
        for (k, o) in v.iter().enumerate() {
            match &side {
                Side::Buy => {
                    if price > o.price() {
                        pos = k;
                    }
                }
                Side::Sell => {
                    if price < o.price() {
                        pos = k;
                    }
                }
            }
        }

        pos
    }

    pub fn new_order(&mut self, side: Side, price: i64, qty: i64, user_id: i64, order_id: i64) -> Response {
        let mut res = Response::Reject;
        let order = Order::new(price, qty, user_id, order_id);

        match side {
            Side::Buy => {
                let pos = OrderBook::sorted_position(Side::Buy, &self.bids, price);

                // Check if this crosses the book
                if self.asks.len() > 0 {
                    if self.asks[0].price() <= price {
                        res = Response::Reject;
                    }
                }

                self.bids.insert(pos, order);
                res = Response::Acknowledge;
            }
            Side::Sell => {
                let pos = OrderBook::sorted_position(Side::Sell, &self.bids, price);
                self.asks.insert(pos, order)
            }
        }

        res
    }

}

mod tests {
    use super::*;

    #[test]
    fn test_empty_orderbook() {
        let ob = OrderBook::new("TSLA");
        assert_eq!("TSLA", ob.ticker());
    }

    #[test]
    fn test_add_1_bid() {
        let mut ob = OrderBook::new("TSLA");
        ob.new_order(Side::Buy, 10, 100, 1, 1);

        assert_eq!("TSLA", ob.ticker());
    }

    #[test]
    fn test_add_1_bid_1_ask() {
        let mut ob = OrderBook::new("TSLA");
        ob.new_order(Side::Buy, 10, 100, 1, 1);
        ob.new_order(Side::Sell, 10, 100, 1, 1);

        assert_eq!("TSLA", ob.ticker());
    }

    #[test]
    fn test_add_3_bids_verfy_sorted() {
        let mut ob = OrderBook::new("TSLA");
        ob.new_order(Side::Buy, 10, 100, 1, 1);
        ob.new_order(Side::Buy, 12, 100, 1, 1);
        ob.new_order(Side::Buy, 11, 100, 1, 1);
        println!("{:?}", ob);
        assert_eq!("TSLA", ob.ticker());
    }
}

