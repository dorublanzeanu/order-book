#[allow(dead_code)]

pub enum Side {
    Buy,
    Sell,
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