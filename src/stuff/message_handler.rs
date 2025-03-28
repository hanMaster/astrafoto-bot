use crate::stuff::data_types::{Message, OrderState};
use crate::stuff::repository::Repository;
use crate::stuff::transport::{Transport, WhatsApp};

pub trait MessageHandler {
    fn handle(&mut self, message: Message);
}

pub struct Handler<'a, R, T>
where
    R: Repository,
    T: Transport,
{
    repository: R,
    transport: &'a T,
}

impl<'a, R, T> Handler<'a, R, T>
where
    R: Repository,
    T: Transport,
{
    pub fn new(repository: R, transport: &'a T) -> Self {
        Self {
            repository,
            transport,
        }
    }
}

impl<R, T> MessageHandler for Handler<'_, R, T>
where
    R: Repository + std::fmt::Debug,
    T: Transport,
{
    fn handle(&mut self, message: Message) {
        match message {
            Message::Text(msg) => {
                let order_option = self.repository.get_order(&msg.chat_id);
                if let Some(order) = order_option {
                    match order {
                        OrderState::RaperRequested { .. } => {
                            // TODO send response with paper request
                        }
                        OrderState::SizeRequested { .. } => {
                            // TODO send response with size request
                        }
                        OrderState::SizeSelected { .. } => {
                            // TODO send response with finish request
                        }
                    }
                    // let mut updated = order.clone();
                    // updated.add_image(msg.message);
                    // self.repository.set_order(updated);
                    println!("Order updated in repo {:#?}", self.repository);
                } else {
                    self.repository.set_order(OrderState::from_txt_msg(msg));
                    println!("Order created in repo {:#?}", self.repository);
                    // TODO send response with paper request
                }
            }
            Message::Image(msg) => {
                let order_option = self.repository.get_order(&msg.chat_id);
                if let Some(order) = order_option {
                    match order {
                        OrderState::RaperRequested { .. } => {
                            // TODO send response with paper request
                        }
                        OrderState::SizeRequested { .. } => {
                            // TODO send response with size request
                        }
                        OrderState::SizeSelected { .. } => {
                            // TODO send response with finish request
                        }
                    }
                    let mut updated = order.clone();
                    updated.add_image(msg.message);
                    self.repository.set_order(updated);
                    println!("Order updated in repo {:#?}", self.repository);
                } else {
                    self.repository.set_order(OrderState::from_img_msg(msg));
                    println!("Order created in repo {:#?}", self.repository);
                    // TODO send response with paper request
                }
            }
            Message::Empty => {}
        }
    }
}
