use std::cmp::Ordering;
use std::time::SystemTime;

//Order
// TODO:: Find a way to attach these enums to the Order struct only and not a global enums
// TODO:: Fix the string types in this project, currently all of them are owned strings

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug, Copy)]
pub enum OrderType {
    Mkt,
    Limit,
}

#[derive(Clone, Debug, Copy)]
pub enum EventType {
    New,
    Rpl,
    Cxl,
}

#[derive(Clone, Debug)]
pub struct Order {
    pub id_: String,
    pub symbol_: String,
    pub qty_: i32,
    pub price_: f32,
    pub entry_time_: SystemTime,
    pub side_: OrderSide,
    pub type_: OrderType,
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        self.entry_time_
            .partial_cmp(&other.entry_time_)
            .unwrap_or(Ordering::Equal)
    }
}

impl Eq for Order {}
