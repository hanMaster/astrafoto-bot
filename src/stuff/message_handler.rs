use crate::stuff::data_types::{Message, OrderState, ReceivedMessage};
use crate::stuff::error::{Error, Result};
use crate::stuff::prompt::Prompt;
use crate::stuff::repository::Repository;
use crate::stuff::transport::Transport;

pub trait MessageHandler {
    async fn handle(&mut self, message: Message);
    async fn handle_awaits(&mut self);
}

pub struct Handler<'a, R, T>
where
    R: Repository,
    T: Transport,
{
    repository: R,
    transport: &'a T,
    prompt: Prompt,
}

impl<'a, R, T> Handler<'a, R, T>
where
    R: Repository + std::fmt::Debug,
    T: Transport,
{
    pub fn new(repository: R, transport: &'a T) -> Self {
        Self {
            repository,
            transport,
            prompt: Prompt::new(),
        }
    }

    async fn handle_image_message(&mut self, message: ReceivedMessage) {
        let chat_id = message.chat_id.clone();
        let order_option = self.repository.get_order(&message.chat_id);
        if let Some(order) = order_option {
            match order {
                OrderState::RaperRequested { .. } => {
                    self.send_paper_request(chat_id).await;
                }
                OrderState::SizeRequested { .. } => {
                    self.send_size_request(chat_id, order.get_paper()).await;
                }
                OrderState::SizeSelected { .. } => {
                    self.send_ready_request(chat_id).await;
                }
            }
            let mut updated = order.clone();
            updated.add_image(message.message);
            self.repository.set_order(updated);
            println!("Order updated in repo {:#?}", self.repository);
        } else {
            self.repository.set_order(OrderState::from_img_msg(message));
            println!("Order created in repo {:#?}", self.repository);
            self.send_paper_request(chat_id).await;
        }
    }

    async fn handle_text_message(&mut self, message: ReceivedMessage) {
        let chat_id = message.chat_id.clone();
        let order_option = self.repository.get_order(&message.chat_id);
        if let Some(order) = order_option {
            match order {
                OrderState::RaperRequested { .. } => {
                    let res = self.try_set_paper(order, message);
                    match res {
                        Ok(paper) => {
                            self.send_size_request(chat_id.clone(), &paper).await;
                        }
                        Err(_) => {
                            self.send_paper_request(chat_id).await;
                        }
                    }
                }

                OrderState::SizeRequested { .. } => {
                    let res = self.try_set_size(order, message);
                    match res {
                        Ok(_) => {
                            self.send_ready_request(chat_id.clone()).await;
                        }
                        Err(Error::SizeInvalid(paper)) => {
                            self.send_size_request(chat_id.clone(), &paper).await;
                        }
                        _ => {}
                    }
                }

                OrderState::SizeSelected { files, .. } => {
                    if message.message.to_lowercase().eq("готово") && !files.is_empty() {
                        // TODO send order to microservice
                        let _ = self.repository.delete_order(&chat_id);
                        self.send_final_request(chat_id).await;
                    } else {
                        self.send_ready_request(chat_id).await;
                    }
                }
            }
            println!("Order updated {:#?}", self.repository);
        } else {
            self.repository.set_order(OrderState::from_txt_msg(message));
            println!("Order created {:#?}", self.repository);
            self.send_paper_request(chat_id).await;
        }
    }

    fn try_set_paper(&mut self, o: OrderState, message: ReceivedMessage) -> Result<String> {
        let paper_type: usize = message.message.parse()?;
        let paper_opt = self.prompt.try_get_paper(paper_type);
        match paper_opt {
            None => Err(Error::PaperInvalid),
            Some(paper) => {
                let new_state = o.into_order_with_paper(paper.clone())?;
                self.repository.set_order(new_state);
                Ok(paper)
            }
        }
    }

    fn try_set_size(&mut self, o: OrderState, message: ReceivedMessage) -> Result<()> {
        let size_type: usize = message.message.parse()?;
        let paper = o.get_paper().to_string();
        let size_opt = self.prompt.try_get_size(o.get_paper(), size_type);
        match size_opt {
            None => Err(Error::SizeInvalid(paper)),
            Some(size) => {
                let new_state = o.into_order_with_size(size)?;
                self.repository.set_order(new_state);
                Ok(())
            }
        }
    }

    async fn send_paper_request(&self, chat_id: String) {
        let res = self
            .transport
            .send_message(chat_id, self.prompt.paper_prompt())
            .await;
        if let Err(e) = res {
            eprintln!("Error sending paper request: {}", e);
        };
    }

    async fn send_size_request(&self, chat_id: String, paper: &str) {
        let res = self
            .transport
            .send_message(chat_id, self.prompt.size_prompt(paper))
            .await;
        if let Err(e) = res {
            eprintln!("Error sending size request: {}", e);
        };
    }

    async fn send_ready_request(&self, chat_id: String) {
        let res = self
            .transport
            .send_message(chat_id, self.prompt.ready_prompt())
            .await;
        if let Err(e) = res {
            eprintln!("Error sending ready request: {}", e);
        };
    }

    async fn send_final_request(&self, chat_id: String) {
        let res = self
            .transport
            .send_message(chat_id, self.prompt.final_prompt())
            .await;
        if let Err(e) = res {
            eprintln!("Error sending final request: {}", e);
        };
    }
}

impl<R, T> MessageHandler for Handler<'_, R, T>
where
    R: Repository + std::fmt::Debug,
    T: Transport,
{
    async fn handle(&mut self, message: Message) {
        match message {
            Message::Text(msg) => {
                self.handle_text_message(msg).await;
            }
            Message::Image(msg) => {
                self.handle_image_message(msg).await;
            }
            Message::Empty => {}
        }
    }

    async fn handle_awaits(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::stuff::repository::OrderRepository;
    use crate::stuff::transport::MockTransport;
    #[tokio::test]
    async fn test_handle_text() {
        let repo = OrderRepository::new();
        let transport = MockTransport;
        let mut handler = Handler::new(repo, &transport);
        let msg = transport.receive_message().await.unwrap();
        handler.handle(msg).await;
        println!("{:#?}", handler.repository);

        let paper_answer = ReceivedMessage{
            chat_id: "79146795555@c.us".to_string(),
            customer_name: "Andrey".to_string(),
            message: "1".to_string(),
        };
        handler.handle_text_message(paper_answer).await;
        println!("{:#?}", handler.repository);

        let size_answer = ReceivedMessage{
            chat_id: "79146795555@c.us".to_string(),
            customer_name: "Andrey".to_string(),
            message: "1".to_string(),
        };
        handler.handle_text_message(size_answer).await;
        println!("{:#?}", handler.repository);
    }
}
