use crate::stuff::data_types::OrderState;
use crate::stuff::error::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait Repository: Clone {
    fn get_order(&self, chat_id: &str) -> Option<OrderState>;
    fn get_orders(&self) -> HashMap<String, OrderState>;
    fn set_order(&mut self, state: OrderState);
    fn delete_order(&mut self, chat_id: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct OrderRepository {
    orders: Arc<Mutex<HashMap<String, OrderState>>>,
}

impl OrderRepository {
    pub fn new() -> OrderRepository {
        Self {
            orders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Repository for OrderRepository {
    fn get_order(&self, chat_id: &str) -> Option<OrderState> {
        self.orders.lock().unwrap().get(chat_id).cloned()
    }

    fn get_orders(&self) -> HashMap<String, OrderState> {
        self.orders.lock().unwrap().clone()
    }

    fn set_order(&mut self, state: OrderState) {
        self.orders
            .lock()
            .unwrap()
            .entry(state.get_chat_id())
            .and_modify(|v| *v = state.clone())
            .or_insert(state);
    }

    fn delete_order(&mut self, chat_id: &str) -> Result<()> {
        let res = self.orders.lock().unwrap().remove(chat_id);
        match res {
            None => Err(Error::OrderNotFound(chat_id.to_string())),
            Some(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stuff::timestamp::Timestamp;
    #[test]
    fn repo_update_order() {
        let mut repo = OrderRepository::new();
        let order = OrderState::RaperRequested {
            chat_id: "79146795551".to_string(),
            customer_name: "John".to_string(),
            files: vec![],
            repeats: 0,
            last_msg_time: Timestamp::now(),
        };
        repo.set_order(order);
        println!("Order update result: {:?}", repo);

        let order = OrderState::SizeRequested {
            chat_id: "79146795552".to_string(),
            customer_name: "Jane".to_string(),
            paper: "paper".to_string(),
            files: vec![],
            repeats: 0,
            last_msg_time: Timestamp::now(),
        };
        repo.set_order(order.clone());

        println!("Order update result: {:?}", repo);
        let saved = repo.get_order("79146795552").unwrap();
        assert_eq!(saved, order);
        {
            repo.delete_order("79146795552").unwrap();
            println!("Order delete result: {:?}", repo);
        }
        assert_eq!(repo.orders.lock().unwrap().len(), 1);
    }
}
