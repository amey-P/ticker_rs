use crate::position::{Position, Transaction};
use crate::scrip::Scrip;
use crate::redis_utils::RedisScrip;
use crate::tickers::Ticker;
use crate::error::Error;
use chrono::prelude::*;

// =============================================================================
//                                Single Order
// =============================================================================

#[derive(Clone, Debug)]
pub enum OrderType {
    MarketOrder,
    LimitOrder(f64),
}

#[derive(Clone, Debug)]
pub struct Order {
    pub scrip: Scrip,
    pub quantity: i32,
    pub order_type: OrderType,
}

impl Order {
    pub fn new(scrip: Scrip, quantity: i32, order_type: OrderType) -> Self {
        Self {
            scrip,
            quantity,
            order_type,
        }
    }

    pub fn available_quantity(&self, ticker: Ticker, buy: bool) -> i32 {
        let orders = match buy {
            true => ticker.depth.ask,
            false => ticker.depth.bid,
        };
        orders.iter().fold(0, |y, x| y + (x.quantity as i32))
    }

    pub fn avg_price(&self) -> Result<f64, Error> {
        let ticker = self.scrip.updated_ticker();
        let mut residual = self.quantity.abs();
        let mut total_amount = 0.0;
        let mut filled_depth = 0;
        let orders = match self.quantity > 0 {
            true => ticker.depth.ask,
            false => ticker.depth.bid,
        };
        if orders.is_empty() { return Err(Error::EmptyDepth); }

        while residual != 0 {
            let order = orders
                .get(filled_depth).ok_or(Error::InsufficientDepth(residual))?;
            let available_quantity = order.quantity;
            if residual > (available_quantity as i32) {
                total_amount += (available_quantity as f64) * order.price;
                residual -= available_quantity as i32;
            } else {
                total_amount += (residual as f64) * order.price;
                residual = 0;
            }

            filled_depth += 1;
        }

        Ok(total_amount / (self.quantity.abs() as f64))
    }

    // Converts an order to a transaction
    // If order is LimitOrder, it blindly executes at the limit price.
    // If order id MarketOrder, it goes through the depth and executes.
    pub fn to_transaction(&self) -> Result<Transaction, Error> {
        let avg_price = match self.order_type {
            OrderType::MarketOrder => self.avg_price()?,
            OrderType::LimitOrder(price) => price,
        };

        let transaction = Transaction {
            scrip: self.scrip.clone(),
            quantity: self.quantity,
            avg_price,
            exec_time: Local::now(),
        };

        Ok(transaction)
    }

    pub fn execute(&self) -> Result<Transaction, Error> {
        // =======================================
        // Some subroutine to hand order to Broker
        // =======================================
        self.to_transaction()
    }

    pub fn margin(&self) -> Result<f64, Error> {
        match self.order_type {
            OrderType::MarketOrder => self.avg_price(),
            OrderType::LimitOrder(p) => Ok(p),
        }
    }
}

// =============================================================================
//                                Basket Orders
// =============================================================================

#[derive(Clone, Debug)]
pub enum BasketOrderType {
    AllOrNone,
}

#[derive(Clone, Debug)]
pub struct BasketOrder {
    pub basket_order_type: BasketOrderType,
    pub orders: Vec<Order>,
}

impl BasketOrder {
    pub fn add_order(&mut self, order: Order) {
        self.orders.push(order);
    }

    pub fn to_position(&self) -> Result<Position, Error> {
        let mut position: Position = Default::default();
        self.orders.iter().try_for_each(|x| {
            let transaction = match x.to_transaction() {
                Ok(t) => t,
                Err(e) => return Err(e),
            };
            position.add_transaction(transaction);
            Ok(())
        })?;

        Ok(position)
    }

    pub fn execute(&self) -> Result<Position, Error> {
        let mut position: Position = Default::default();
        self.orders.iter().try_for_each(|x| {
            let transaction = match x.execute() {
                Ok(t) => t,
                Err(e) => return Err(e),
            };
            position.add_transaction(transaction);
            Ok(())
        })?;

        Ok(position)
    }

    pub fn margin(&self) -> Result<f64, Error> {
        self.orders.iter().try_fold(0.0, |x, y| Ok(x + y.margin()?))
    }

    pub fn extend(&mut self, other: Self, basket_order_type: Option<BasketOrderType>) {
        self.orders.extend(other.orders);
        if let Some(ot) = basket_order_type {
            self.basket_order_type = ot;
        }
    }
}

// =============================================================================
//                                  Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::*;
    
    #[test]
    fn limited_marketorder_buy() {
        let scrip = StockScrip::new("TEST", "NSE", "C");
        let buy_5 = Order {
            scrip: scrip::Scrip::Stock(scrip.clone()),
            quantity: 8,
            order_type: OrderType::MarketOrder,
        };
        assert_eq!(buy_5.avg_price(), Ok(401.5175));
    }

    #[test]
    fn limited_marketorder_sell() {
        let scrip = StockScrip::new("TEST", "NSE", "C");
        let sell_5 = Order {
            scrip: scrip::Scrip::Stock(scrip.clone()),
            quantity: -8,
            order_type: OrderType::MarketOrder,
        };
        assert_eq!(sell_5.avg_price(), Ok(400.1425));
    }

    #[test]
    fn unlimited_marketorder_buy() {
        let scrip = StockScrip::new("TEST", "NSE", "C");
        let buy_all = Order {
            scrip: scrip::Scrip::Stock(scrip.clone()),
            quantity: 20,
            order_type: OrderType::MarketOrder,
        };
        assert_eq!(buy_all.avg_price(), Err(error::Error::InsufficientDepth(6)));
    }

    #[test]
    fn unlimited_marketorder_sell() {
        let scrip = StockScrip::new("TEST", "NSE", "C");
        let sell_all = Order {
            scrip: scrip::Scrip::Stock(scrip.clone()),
            quantity: -20,
            order_type: OrderType::MarketOrder,
        };
        assert_eq!(sell_all.avg_price(), Err(error::Error::InsufficientDepth(6)));
    }

}
