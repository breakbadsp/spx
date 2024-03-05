use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;

use msg::*;

#[derive(Debug, Clone)]
pub struct MatchingResult {
    matched_order_ids_: Vec<String>,
    executed_qty_: i32,
    executed_price_: f32,
}

impl MatchingResult {
    fn new() -> Self {
        MatchingResult {
            matched_order_ids_: Vec::new(),
            executed_qty_: 0,
            executed_price_: 0.0,
        }
    }
}

impl PartialEq for MatchingResult {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for MatchingResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MatchingResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.executed_qty_
            .partial_cmp(&other.executed_qty_)
            .unwrap_or(Ordering::Equal)
    }
}

impl Eq for MatchingResult {}


#[derive(Clone, Debug)]
struct Level {
    orders_: BTreeSet<Order>,
    price_: f32,
    side_: OrderSide,
}

impl PartialOrd for Level {
    fn partial_cmp(&self, p_other: &Self) -> Option<Ordering> {
        Some(self.cmp(p_other))
    }
}

impl PartialEq for Level {
    fn eq(&self, p_other: &Self) -> bool {
        self.cmp(p_other) == Ordering::Equal
    }
}

impl Ord for Level {
    fn cmp(&self, p_other: &Self) -> Ordering {
        self.compare(p_other)
    }
}

impl Eq for Level {}

impl Level {
    fn compare(&self, p_other: &Self) -> Ordering {
        //assert!(self.side_ == p_other.side_);
        match self.side_ {
            OrderSide::Buy => p_other
                .price_
                .partial_cmp(&self.price_)
                .unwrap_or(Ordering::Equal),
            OrderSide::Sell => self
                .price_
                .partial_cmp(&p_other.price_)
                .unwrap_or(Ordering::Equal),
        }
    }

    fn from_order(p_order: &Order) -> Self {
        let new_level = Level {
            price_: p_order.price_,
            orders_: BTreeSet::new(),
            side_: p_order.side_,
        };
        new_level
    }

    fn from_first_order(p_order: &Order) -> Self {
        let mut new_level = Level::from_order(p_order);
        new_level.add_order(p_order);
        new_level
    }

    fn add_order(&mut self, p_order: &Order) {
        self.orders_.insert(p_order.to_owned());
        println!("Order id {:?} added into {:?}", p_order.id_, self);
    }

    fn remove_order(&mut self, p_remove_order: &Order) -> bool {
        self.orders_.remove(p_remove_order)
    }

    fn match_order(&mut self, p_order: &mut Order) -> Result<Option<MatchingResult>, String> {
        //match the qty
        //step 1: get copy of first order
        //step 2: if p_order qty is == first order then remove that order and return total qty of current order as Ok
        //step 2: if p_order qty is < first order then replace that order with qty = that order qty - p_order qty
        //step 3: of p_order qty is > first order then remove first order, p_order.qty -= first_order.qty_  and repeat from step 1

        let mut executed_qty = 0;
        let mut remaining_qty = p_order.qty_;
        let mut avg_matched_price = 0.0;

        println!("Executing {remaining_qty}");
        let mut result = MatchingResult::new();
        while remaining_qty > 0 && !self.orders_.is_empty() {
            let first_order_if_any = self.orders_.first();
            match first_order_if_any {
                None => {
                    return Ok(None);
                }

                Some(first_order) => {
                    result.matched_order_ids_.push(first_order.id_.to_owned());
                    let mut copy_of_first_order = (*first_order).clone();
                    println!("match found order:\n\t {:?}", copy_of_first_order);

                    if remaining_qty == copy_of_first_order.qty_ {
                        //remove order and return exec qty
                        copy_of_first_order.qty_ = 0;
                        executed_qty = remaining_qty;
                        remaining_qty = 0;
                        avg_matched_price += copy_of_first_order.price_ * executed_qty as f32;
                        self.orders_.pop_first();
                    } else if remaining_qty < copy_of_first_order.qty_ {
                        executed_qty += remaining_qty;
                        copy_of_first_order.qty_ -= remaining_qty;
                        avg_matched_price += copy_of_first_order.price_ * remaining_qty as f32;
                        remaining_qty = 0;
                        println!("{executed_qty}  is executed and {remaining_qty} remaining, inplace order\n\t {:?}", copy_of_first_order);
                        self.orders_.replace(copy_of_first_order);
                        println!("Orders in level after this match:\n\t {:?}", self.orders_);
                    } else if remaining_qty > copy_of_first_order.qty_ {
                        let being_executed = copy_of_first_order.qty_;
                        copy_of_first_order.qty_ -= 0;
                        executed_qty += being_executed;
                        remaining_qty -= being_executed;
                        println!(
                            "{being_executed}  is being executed and {remaining_qty} remaining."
                        );
                        avg_matched_price += copy_of_first_order.price_ * being_executed as f32;
                        self.orders_.pop_first();
                    }
                }
            }
        }
        result.executed_qty_ = executed_qty;
        if executed_qty > 0 {
            result.executed_price_ = avg_matched_price / executed_qty as f32;
        }
        return Ok(Some(result));
    }
}

#[derive(Debug)]
struct OrderBook {
    bids_: BTreeSet<Level>,
    asks_: BTreeSet<Level>,
}

impl OrderBook {
    fn add_first_order(&mut self, p_order: &mut Order) -> Result<Option<MatchingResult>, String> {
        match p_order.side_ {
            OrderSide::Buy => {
                self.bids_.insert(Level::from_first_order(p_order));
                Ok(None)
            }

            OrderSide::Sell => {
                self.asks_.insert(Level::from_first_order(p_order));
                Ok(None)
            }
        }
    }

    fn get_level_match(&self, p_input_order: &Order) -> Option<&Level> {
        match p_input_order.side_ {
            OrderSide::Buy => match p_input_order.type_ {
                OrderType::Mkt => {
                    self.asks_.first()
                }
                OrderType::Limit => {
                    self.asks_.get(&Level::from_order(p_input_order))
                }
            },
            OrderSide::Sell => match p_input_order.type_ {
                OrderType::Mkt => {
                    self.bids_.first()
                }
                OrderType::Limit => {
                    self.bids_.get(&Level::from_order(p_input_order))
                }
            },
        }
    }

    fn get_level_match_from_id(&self, p_order: &Order) -> Option<(&Level, &Order)> {
        match p_order.side_ {
            OrderSide::Buy => {
                for level in &self.bids_ {
                    for order in &level.orders_ {
                        if order.id_ == p_order.id_ {
                            return Some((level, order));
                        }
                    }
                }
            }

            OrderSide::Sell => {
                for level in &self.asks_ {
                    for order in &level.orders_ {
                        if order.id_ == p_order.id_ {
                            return Some((level, order));
                        }
                    }
                }
            }
        }
        None
    }

    fn match_order(&mut self, p_order: &mut Order) -> Result<Option<MatchingResult>, String> {
        let found_level = self.get_level_match(p_order);
        match found_level {
            None => {
                Ok(None)
            }
            Some(matched_level) => {
                println!("Matched to {:?}", matched_level);
                let mut copy_of_matched_level = (*matched_level).clone();
                let match_result = copy_of_matched_level.match_order(p_order)?;

                if match_result.is_none() {
                    return Ok(None);
                }

                match p_order.side_ {
                    OrderSide::Buy => {
                        if copy_of_matched_level.orders_.is_empty() {
                            self.asks_.remove(&copy_of_matched_level);
                        } else {
                            self.asks_.replace(copy_of_matched_level);
                        }
                    }
                    OrderSide::Sell => {
                        if copy_of_matched_level.orders_.is_empty() {
                            self.bids_.remove(&copy_of_matched_level);
                        } else {
                            self.bids_.replace(copy_of_matched_level);
                        }
                    }
                }
                println!("After match {:?}", self);
                Ok(match_result)
            }
        }
    }

    fn add_order(&mut self, p_order: &mut Order) {
        let mut temp_level = Level::from_order(&p_order);
        match p_order.side_ {
            OrderSide::Buy => {
                let found_level = self.bids_.get(&temp_level);

                match found_level {
                    None => {
                        temp_level.add_order(p_order);
                        self.bids_.insert(temp_level);
                    }

                    Some(current_level) => {
                        let mut copy_of_found_level = (*current_level).clone();
                        copy_of_found_level.add_order(p_order);
                        self.bids_.replace(copy_of_found_level);
                    }
                }
            }

            OrderSide::Sell => {
                let found_level = self.asks_.get(&temp_level);

                match found_level {
                    None => {
                        temp_level.add_order(p_order);
                        self.asks_.insert(temp_level);
                    }

                    Some(current_level) => {
                        let mut copy_of_found_level = (*current_level).clone();
                        copy_of_found_level.add_order(p_order);
                        self.asks_.replace(copy_of_found_level);
                    }
                }
            }
        }
        println!("After add_order {:#?}", self);
    }

    fn remove_order_by_id(&mut self, p_order: &Order) -> bool {
        let level_order_match_or_none = self.get_level_match_from_id(p_order);
        match level_order_match_or_none {
            None => {
                return false;
            }
            Some((matched_level, matched_order)) => {
                let mut copy_of_found_level = (*matched_level).clone();
                let copy_of_found_order = (*matched_order).clone();
                if copy_of_found_level.remove_order(&copy_of_found_order) {
                    match p_order.side_ {
                        OrderSide::Buy => {
                            if copy_of_found_level.orders_.is_empty() {
                                return self.bids_.remove(&copy_of_found_level);
                            } else {
                                self.bids_.replace(copy_of_found_level);
                                //TODO:: verify replaced element
                            }
                            return true;
                        }
                        OrderSide::Sell => {
                            if copy_of_found_level.orders_.is_empty() {
                                return self.asks_.remove(&copy_of_found_level);
                            } else {
                                self.asks_.replace(copy_of_found_level);
                                //TODO:: verify replaced element
                            }
                            return true;
                        }
                    }
                }
            }
        }
        return false;
    }
}

#[derive(Debug)]
pub struct MatchingEngine {
    order_book_by_symbol_: HashMap<String, OrderBook>,
}

impl MatchingEngine {
    pub fn process_new_order(
        &mut self,
        p_order: &mut Order,
    ) -> Result<Option<MatchingResult>, String> {
        let order_book_or_error = self.get_book_by_symbol(&p_order.symbol_);
        match order_book_or_error {
            None => {
                if let Some(new_order_book) = self.add_order_book(&p_order.symbol_) {
                    return new_order_book.add_first_order(p_order);
                }
                Err(String::from(
                    "Failed to add first order in a order book of symbol {p_order.symbol_}",
                ))
            }

            Some(order_book) => {
                let matching_result_or_none = order_book.match_order(p_order)?;
                match matching_result_or_none {
                    None => {
                        order_book.add_order(p_order);
                        println!("After add {:?}", self);
                        Ok(None)
                    }
                    Some(match_result) => {
                        p_order.qty_ -= match_result.executed_qty_;
                        if p_order.qty_ > 0 {
                            order_book.add_order(p_order);
                        }
                        println!(
                            "Match result: {:?}, order qty: {} ",
                            match_result, p_order.qty_
                        );
                        Ok(Some(match_result))
                    }
                }
            }
        }
    }

    pub fn process_rpl_order(
        &mut self,
        p_order: &mut Order,
    ) -> Result<Option<MatchingResult>, String> {
        let order_book_or_error = self.get_book_by_symbol(&p_order.symbol_);
        match order_book_or_error {
            None => {
                Err(String::from(
                    "Failed find the order book of symbol {p_order.symbol_}, replace on order failed",
                ))
            }

            Some(order_book) => {
                let order_removed = order_book.remove_order_by_id(p_order);
                if !order_removed {
                    return Err(String::from(
                        "Failed to remove original order, replace failed",
                    ));
                }

                let matching_result_or_none = order_book.match_order(p_order)?;
                match matching_result_or_none {
                    None => {
                        order_book.add_order(p_order);
                        println!("After add {:?}", self);
                        Ok(None)
                    }
                    Some(match_result) => {
                        p_order.qty_ -= match_result.executed_qty_;
                        if p_order.qty_ > 0 {
                            order_book.add_order(p_order);
                        }
                        println!(
                            "Match result: {:?}, order qty: {} ",
                            match_result, p_order.qty_
                        );
                        Ok(Some(match_result))
                    }
                }
            }
        }
    }

    pub fn process_cxl_order(
        &mut self,
        p_order: &mut Order,
    ) -> Result<Option<MatchingResult>, String> {
        let order_book_or_error = self.get_book_by_symbol(&p_order.symbol_);
        match order_book_or_error {
            None => {
                Err(String::from(
           "Failed find the order book of symbol {p_order.symbol_}, replace on order failed",
          ))
            }

            Some(order_book) => {
                let order_removed = order_book.remove_order_by_id(p_order);
                if !order_removed {
                    return Err(String::from(
                        "Failed to remove original order, cancel failed",
                    ));
                }
                //TODO:: retrigger matching of top BIDS and ASKS if top is cancelled
                Ok(None)
            }
        }
    }

    pub fn contains(&self, p_symbol: &String) -> bool {
        self.order_book_by_symbol_.contains_key(p_symbol)
    }

    fn get_book_by_symbol(&mut self, p_symbol: &String) -> Option<&mut OrderBook> {
        if let Some(mutable_order) = self.order_book_by_symbol_.get_mut(p_symbol) {
            return Some(mutable_order);
        }
        None
    }

    fn add_order_book(&mut self, p_symbol: &String) -> Option<&mut OrderBook> {
        let new_order_book = OrderBook {
            bids_: BTreeSet::new(),
            asks_: BTreeSet::new(),
        };

        self.order_book_by_symbol_
            .insert(p_symbol.to_owned(), new_order_book);
        self.order_book_by_symbol_.get_mut(p_symbol)
    }
}

pub fn process_event(
    p_event_type: EventType,
    p_order: &mut Order,
    p_order_book_collection: &mut MatchingEngine,
) -> Result<Option<MatchingResult>, String> {
    match p_event_type {
        EventType::New => {
            println!("\nNew Order, received:\n\t {:?}", p_order);
            p_order_book_collection.process_new_order(p_order)
        }

        EventType::Rpl => {
            println!("\nReplace Order, received:\n\t {:?}", p_order);
            p_order_book_collection.process_rpl_order(p_order)
        }

        EventType::Cxl => {
            println!("\nCancel Order, received:\n\t {:?}", p_order);
            p_order_book_collection.process_cxl_order(p_order)
        }
    }
}

#[cfg(test)]
mod test {

    /*
     * Properties to verify :
     *   - exec Qty, exec Price, matched order id
     * Matching scenarios:
     *   - Simple match with Mkt order
     *   - Simple match with limit order (price match)
     *   - Best Price priority Then time priority (TODO:: More test cases)
     *   - Best Price for bids vs best price for sells
     *   - First order is mkt (no match found) => TODO::
     *   - Match with Multiple orders : TODO:: Testing
     *   -
     *   -
     */

    use super::*;

    fn validate_result(
        p_result: &Result<Option<MatchingResult>, String>,
        p_exp_exec_qty: i32,
        p_exp_exec_price: f32,
        p_matched_order_ids: Option<&Vec<String>>,
    ) {
        match p_result {
            Ok(match_result_or_none) => match match_result_or_none {
                None => {
                    assert!(p_exp_exec_qty == 0);
                }
                Some(match_result) => {
                    assert_eq!(match_result.executed_qty_, p_exp_exec_qty);
                    match p_matched_order_ids {
                        None => {
                            assert!(match_result.matched_order_ids_.is_empty());
                        }
                        Some(matched_ord_ids) => {
                            assert_eq!(match_result.executed_price_, p_exp_exec_price);
                            assert_eq!(&match_result.matched_order_ids_, matched_ord_ids);
                        }
                    }
                }
            },
            Err(error_msg) => {
                println!("process event failed with error {error_msg}");
                assert!(false);
            }
        }
    }

    #[test]
    fn create_first_order() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };

        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        validate_result(&result, 0, 0.0, None);
    }

    #[test]
    fn qty_match_simple_order() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };

        let mut matched_order_ids = Vec::new();
        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        matched_order_ids.push("1".to_string());
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        validate_result(&result, 200, order.price_, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("3"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("4"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        matched_order_ids.clear();
        matched_order_ids.push("3".to_string());
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        validate_result(&result, 200, order.price_, Some(&matched_order_ids));
    }

    #[test]
    fn qty_macth_test_partial_match() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };
        let mut matched_order_ids = Vec::new();

        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200 added to book, exected 0;
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 100,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //100 partially executed, 100 buy left in book
        matched_order_ids.clear();
        matched_order_ids.push("1".to_string());
        validate_result(&result, 100, order.price_, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("3"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //100 executed, 100 sell id 3 left in book
        validate_result(&result, 100, order.price_, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("4"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 100,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //100 executed, nothing left in book
        matched_order_ids.clear();
        matched_order_ids.push("3".to_string());
        validate_result(&result, 100, order.price_, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("5"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200 buy added in book, nothing executed
        matched_order_ids.clear();
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("6"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200 buy sell matched, nothin left in book
        matched_order_ids.push("5".to_string());
        validate_result(&result, 200, order.price_, Some(&matched_order_ids));
    }

    #[test]
    fn mkt_order_match_simple() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };
        let mut matched_order_ids = Vec::new();

        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched
        matched_order_ids.push("1".to_string());
        validate_result(&result, 200, 100.0, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("3"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("4"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //another 200@100 added into book but not matched
        validate_result(&result, 0, 0.0, None);
    }

    #[test]
    fn mkt_order_match_time() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };
        let mut matched_order_ids = Vec::new();
        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //Another 200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("3"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched 200@100
        matched_order_ids.push("1".to_string());
        validate_result(&result, 200, 100.0, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("4"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched 200@100
        matched_order_ids.clear();
        matched_order_ids.push("2".to_string());
        validate_result(&result, 200, 100.0, Some(&matched_order_ids));
    }

    #[test]
    fn mkt_order_match_price() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };
        let mut matched_order_ids = Vec::new();
        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200@101 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 101.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //Another 200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("3"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        matched_order_ids.push("2".to_string());
        validate_result(&result, 200, 101.0, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("4"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched
        matched_order_ids.clear();
        matched_order_ids.push("1".to_string());
        validate_result(&result, 200, 100.0, Some(&matched_order_ids));
    }

    #[test]
    fn mkt_order_match_price_sell_buy() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };
        let mut matched_order_ids = Vec::new();
        let mut order = Order {
            id_: String::from("1"),
            price_: 102.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //200@101 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("2"),
            price_: 101.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //Another 200@100 buy added to book
        validate_result(&result, 0, 0.0, None);

        let mut order = Order {
            id_: String::from("3"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        matched_order_ids.push("2".to_string());
        validate_result(&result, 200, 101.0, Some(&matched_order_ids));

        let mut order = Order {
            id_: String::from("4"),
            price_: 0.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Mkt,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched
        matched_order_ids.clear();
        matched_order_ids.push("1".to_string());
        validate_result(&result, 200, 102.0, Some(&matched_order_ids));
    }

    #[test]
    fn cancel_order_simple() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };

        //New order
        let mut order = Order {
            id_: String::from("1"),
            price_: 100.1,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        let mut matched_order_ids = Vec::new();
        validate_result(&result, 00, 0.0, Some(&matched_order_ids));

        //Cancel order id 1 , execqty 0 no erro
        let mut order = Order {
            id_: String::from("1"),
            price_: 100.1,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };

        let result = process_event(EventType::Cxl, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        matched_order_ids.clear();
        validate_result(&result, 00, 0.0, Some(&matched_order_ids));

        //Sending matching order to 1, but it should have been removed so no exec qty
        let mut order = Order {
            id_: String::from("2"),
            price_: 100.1,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };

        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        matched_order_ids.clear();
        validate_result(&result, 00, 0.0, Some(&matched_order_ids));

        //Sending matching order to 2, it should get executed
        let mut order = Order {
            id_: String::from("3"),
            price_: 100.1,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };

        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        matched_order_ids.push(String::from("2"));
        validate_result(&result, 200, 100.1, Some(&matched_order_ids));
    }

    #[test]
    fn simple_replace_order() {
        let mut order_book_collection = MatchingEngine {
            order_book_by_symbol_: HashMap::new(),
        };

        //New order
        let mut order = Order {
            id_: String::from("1"),
            price_: 100.1,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        let mut matched_order_ids = Vec::new();
        validate_result(&result, 00, 0.0, Some(&matched_order_ids));

        //Replace order id 1, make price less aggressive
        let mut order = Order {
            id_: String::from("1"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Buy,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::Rpl, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        matched_order_ids.clear();
        validate_result(&result, 00, 0.0, Some(&matched_order_ids));

        //sending sell order with less aggressive price so it does not match
        let mut order = Order {
            id_: String::from("2"),
            price_: 100.1,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(EventType::New, &mut order, &mut order_book_collection);
        //mkt matched to best price which is 100 at this time
        matched_order_ids.clear();
        validate_result(&result, 00, 0.0, Some(&matched_order_ids));

        //Replace above sell order with more aggressive price so it does match
        let mut order = Order {
            id_: String::from("2"),
            price_: 100.0,
            symbol_: String::from("REL"),
            qty_: 200,
            side_: OrderSide::Sell,
            type_: OrderType::Limit,
            entry_time_: std::time::SystemTime::now(),
        };
        let result = process_event(
          EventType::Rpl, 
          &mut order, 
          &mut order_book_collection
        );
        
        //mkt matched to best price which is 100 at this time
        matched_order_ids.push(String::from("1"));
        validate_result(&result, 200, 100.0, Some(&matched_order_ids));
    }
}
