use std::collections::HashMap;
use crate::stuff::data_types::OrderState;
use crate::stuff::error::{Error, Result};

pub trait Repository {
    fn get_order(&self, chat_id: &str) -> Option<OrderState>;
    fn set_order(&mut self, state: OrderState);
    fn delete_order(&mut self, chat_id: &str) -> Result<()>;
}

#[derive(Debug)]
pub struct OrderRepository {
    orders: HashMap<String, OrderState>,
}

impl OrderRepository {
    pub fn new() -> OrderRepository {
        Self {
            orders: HashMap::new(),
        }
    }
}

impl Repository for OrderRepository {
    fn get_order(&self, chat_id: &str) -> Option<OrderState> {
        self.orders.get(chat_id).cloned()
    }

    fn set_order(&mut self, state: OrderState) {
        let order = self.orders.get_mut(&state.get_chat_id());
        match order {
            Some(order) => {
                *order = state;
            }
            None => {
                self.orders.insert(state.get_chat_id(), state);
            }
        }
    }

    fn delete_order(&mut self, chat_id: &str) -> Result<()> {
        let res = self.orders.remove(chat_id);
        match res {
            None => Err(Error::OrderNotFound(chat_id.to_string())),
            Some(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repo_update_order() {
        let mut repo = OrderRepository::new();
        let order = OrderState::RaperRequested {
            chat_id: "79146795551".to_string(),
            customer_name: "John".to_string(),
            files: vec![],
        };
        repo.set_order(order);
        println!("Order update result: {:?}", repo);

        let order = OrderState::SizeRequested {
            chat_id: "79146795552".to_string(),
            customer_name: "Jane".to_string(),
            paper: "paper".to_string(),
            files: vec![],
        };
        repo.set_order(order.clone());

        println!("Order update result: {:?}", repo);
        let saved = repo.orders.get("79146795552").unwrap();
        assert_eq!(*saved, order);
        {
            repo.delete_order("79146795552").unwrap();
            println!("Order delete result: {:?}", repo);
        }
        assert_eq!(repo.orders.len(), 1);
    }
}
