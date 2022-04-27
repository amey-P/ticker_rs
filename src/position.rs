use crate::scrip::{Scrip, RedisScrip};
use chrono::prelude::*;
use std::collections::HashMap;

pub struct Transaction {
    pub scrip: Scrip,
    pub quantity: i32,
    pub avg_price: f64,
    pub exec_time: DateTime<Local>,
}

#[derive(Default)]
pub struct Position {
    pub history: Vec<Transaction>,
    pub holding: HashMap<Scrip, (i32, f64)>,
}

impl Position {
    pub fn get_pnl(&self) -> f64 {
        self.holding.iter().fold(0.0, |x, (s, (q, p))| {
            x + (*q as f64) * (s.updated_ticker().ltp - p)
        })
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.update_holding(transaction.scrip.clone(), transaction.quantity, transaction.avg_price);
        self.history.push(transaction);
    }

    pub fn extend(&mut self, new_position: Position) {
        self.history.extend(new_position.history);
        new_position.holding.into_iter().for_each(|(s, (q, p))| self.update_holding(s, q, p));
    }

    pub fn sort_history(&mut self) {
        self.history.sort_by(|x, y| x.exec_time.partial_cmp(&y.exec_time).unwrap());
    }

    pub fn update_holding(&mut self, scrip: Scrip, quantity: i32, price: f64) {
        match self.holding.get_mut(&scrip) {
            Some(h) => {
                let total_cost =
                    (h.1 * (h.0 as f64)) + (quantity as f64 * price);
                let avg_price = total_cost / ((h.0 + quantity) as f64);
                h.0 += quantity;
                h.1 = avg_price;
            }
            None => {
                self.holding.insert(
                    scrip,
                    (quantity, price),
                );
            }
        };
    }
}
