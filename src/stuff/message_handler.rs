use crate::config::config;
use crate::stuff::data_types::{Message, OrderState, ReceivedMessage};
use crate::stuff::error::{Error, Result};
use crate::stuff::prompt::Prompt;
use crate::stuff::repository::Repository;
use crate::stuff::transport::Transport;
use log::{error, info};

pub trait MessageHandler {
    async fn handle(&mut self, message: Message) -> Result<()>;
    async fn handle_awaits(&mut self) -> Result<()>;
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

    async fn handle_image_message(&mut self, message: ReceivedMessage) -> Result<()> {
        let order_option = self.repository.get_order(&message.chat_id);
        if let Some(order) = order_option {
            let mut updated = order.clone();
            updated.add_image(message.message);
            self.send_receive_file_confirmation(updated.get_chat_id(), updated.files_count())
                .await;
            self.repository.set_order(updated);
            info!("Order updated in repo {:#?}", self.repository);
        } else {
            let new_order = OrderState::from_img_msg(message);
            self.send_receive_file_confirmation(new_order.get_chat_id(), new_order.files_count())
                .await;
            self.repository.set_order(new_order);
            info!("Order created in repo {:#?}", self.repository);
        }
        Ok(())
    }

    async fn handle_text_message(&mut self, message: ReceivedMessage) -> Result<()> {
        let chat_id = message.chat_id.clone();
        let order_option = self.repository.get_order(&message.chat_id);
        if let Some(order) = order_option {
            // Клиент пожелал отменить заказ
            if message.message.to_lowercase().contains("отмен") {
                self.repository.delete_order(&chat_id)?;
                self.send_cancel(chat_id).await;
                return Ok(());
            }

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
                            error!("Paper size invalid: {:?}", paper);
                            self.send_size_request(chat_id.clone(), &paper).await;
                        }
                        _ => {}
                    }
                }

                OrderState::SizeSelected { .. } => {
                    if message.message.to_lowercase().contains("готов") && order.have_files() {
                        self.send_wait_request(chat_id.clone()).await;
                        let res = self.transport.send_order(order).await;
                        self.repository.delete_order(&chat_id)?;
                        match res {
                            Ok(order_id) => {
                                info!("Order from {} DONE with id {}", chat_id, order_id);
                                self.send_final_request(chat_id, order_id).await;
                            }
                            Err(_) => {
                                self.send_error_request(chat_id).await;
                            }
                        }
                    } else {
                        self.send_ready_request(chat_id).await;
                    }
                }
            }
            info!("Order updated {:#?}", self.repository);
        } else {
            self.repository.set_order(OrderState::from_txt_msg(message));
            info!("Order created {:#?}", self.repository);
            self.send_paper_request(chat_id).await;
        }
        Ok(())
    }

    fn try_set_paper(&mut self, o: OrderState, message: ReceivedMessage) -> Result<String> {
        let paper_type: usize = message.message.parse()?;
        let paper_opt = self.prompt.try_get_paper(paper_type - 1);
        info!("paper_opt {:?}", paper_opt);
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
        let size_opt = self.prompt.try_get_size(o.get_paper(), size_type - 1);
        info!("size_opt {:?}", size_opt);
        match size_opt {
            None => Err(Error::SizeInvalid(paper)),
            Some((size, price)) => {
                let new_state = o.into_order_with_size(size, price)?;
                self.repository.set_order(new_state);
                Ok(())
            }
        }
    }

    async fn send_receive_file_confirmation(&self, chat_id: String, count: usize) {
        let res = self
            .transport
            .send_message(chat_id, format!("От Вас получено файлов: {}", count))
            .await;
        if let Err(e) = res {
            error!("Error sending paper request: {}", e);
        };
    }

    async fn send_paper_request(&self, chat_id: String) {
        let res = self
            .transport
            .send_message(chat_id, self.prompt.paper_prompt())
            .await;
        if let Err(e) = res {
            error!("Error sending paper request: {}", e);
        };
    }

    async fn send_size_request(&self, chat_id: String, paper: &str) {
        let res = self
            .transport
            .send_message(chat_id, self.prompt.size_prompt(paper))
            .await;
        if let Err(e) = res {
            error!("Error sending size request: {}", e);
        };
    }

    async fn send_ready_request(&self, chat_id: String) {
        let res = self
            .transport
            .send_message(chat_id, self.prompt.ready_prompt())
            .await;
        if let Err(e) = res {
            error!("Error sending ready request: {}", e);
        };
    }

    async fn send_wait_request(&self, chat_id: String) {
        let res = self
            .transport
            .send_message(
                chat_id,
                "Пожалуйста ожидайте, Ваш заказ обрабатывается".to_string(),
            )
            .await;
        if let Err(e) = res {
            error!("Error sending final request: {}", e);
        };
    }

    async fn send_final_request(&self, chat_id: String, order_id: String) {
        let res = self
            .transport
            .send_message(chat_id, self.prompt.final_prompt(order_id))
            .await;
        if let Err(e) = res {
            error!("Error sending final request: {}", e);
        };
    }

    async fn send_error_request(&self, chat_id: String) {
        let res = self
            .transport
            .send_message(
                chat_id,
                "Не удалось отправить Ваш заказ, попробуйте еще раз".to_string(),
            )
            .await;
        if let Err(e) = res {
            error!("Error sending final request: {}", e);
        };
    }

    async fn send_cancel(&self, chat_id: String) {
        let res = self
            .transport
            .send_message(chat_id, "Ваш заказ отменен".to_string())
            .await;
        if let Err(e) = res {
            error!("Error sending final request: {}", e);
        };
    }
}

impl<R, T> MessageHandler for Handler<'_, R, T>
where
    R: Repository + std::fmt::Debug,
    T: Transport,
{
    async fn handle(&mut self, message: Message) -> Result<()> {
        match message {
            Message::Text(msg) => {
                self.handle_text_message(msg).await?;
            }
            Message::Image(msg) => {
                self.handle_image_message(msg).await?;
            }
            Message::Empty => {}
        }
        Ok(())
    }

    async fn handle_awaits(&mut self) -> Result<()> {
        let orders = self.repository.get_orders();
        let mut orders_to_remove = vec![];
        for (_, o) in orders {
            match o.have_files() {
                true => {
                    if o.repeats() < config().REPEAT_COUNT
                        && o.last_time_sec() > config().REPEAT_TIMEOUT
                    {
                        let mut clonned = o.clone();
                        clonned.requested();
                        self.repository.set_order(clonned);
                        match o {
                            OrderState::RaperRequested { .. } => {
                                self.send_paper_request(o.get_chat_id()).await;
                            }
                            OrderState::SizeRequested { .. } => {
                                self.send_size_request(o.get_chat_id(), o.get_paper()).await;
                            }
                            OrderState::SizeSelected { .. } => {
                                self.send_ready_request(o.get_chat_id()).await;
                            }
                        }
                    } else if o.repeats() < config().REPEAT_COUNT
                        && o.last_time_sec() < config().REPEAT_TIMEOUT
                    {
                    } else {
                        orders_to_remove.push(o.get_chat_id());
                    }
                }
                false => {
                    if o.last_time_sec() > config().NO_FILES_TIMEOUT {
                        orders_to_remove.push(o.get_chat_id());
                    }
                }
            }
        }
        for chat_id in orders_to_remove {
            self.repository.delete_order(&chat_id)?;
            self.transport
                .send_message(
                    chat_id.clone(),
                    "Заказ отменен, из-за длительного ожидания".to_string(),
                )
                .await?;
        }
        Ok(())
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
        let res = handler.handle(msg).await;
        assert!(res.is_ok());
        println!("{:#?}", handler.repository);

        let paper_answer = ReceivedMessage {
            chat_id: "79146795555@c.us".to_string(),
            customer_name: "Andrey".to_string(),
            message: "1".to_string(),
        };
        let res = handler.handle_text_message(paper_answer).await;
        assert!(res.is_ok());
        println!("{:#?}", handler.repository);

        let size_answer = ReceivedMessage {
            chat_id: "79146795555@c.us".to_string(),
            customer_name: "Andrey".to_string(),
            message: "1".to_string(),
        };
        let res = handler.handle_text_message(size_answer).await;
        assert!(res.is_ok());
        println!("{:#?}", handler.repository);

        let size_answer = ReceivedMessage {
            chat_id: "79146795555@c.us".to_string(),
            customer_name: "Andrey".to_string(),
            message: "Отмените".to_string(),
        };
        let res = handler.handle_text_message(size_answer).await;
        assert!(res.is_ok());
        println!("{:#?}", handler.repository);
    }
}
