use crate::stuff::data_types::{Message, OrderState, ReceivedMessage};
use crate::stuff::prompt::Prompt;
use crate::stuff::repository::Repository;
use crate::stuff::transport::Transport;

pub trait MessageHandler {
    async fn handle(&mut self, message: Message);
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
                OrderState::SizeRequested { paper, .. } => {
                    self.send_size_request(chat_id, paper).await;
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
                OrderState::RaperRequested {
                    customer_name,
                    files,
                    ..
                } => {
                    let paper_type: usize = message.message.parse().unwrap_or(0);
                    if paper_type > 0 && paper_type <= self.prompt.paper_vec.len() {
                        let paper = self.prompt.paper_vec[paper_type - 1].clone();
                        self.send_size_request(chat_id.clone(), &paper).await;
                        let new_state = OrderState::SizeRequested {
                            chat_id,
                            customer_name: customer_name.clone(),
                            paper,
                            files: files.clone(),
                        };
                        self.repository.set_order(new_state);
                    } else {
                        self.send_paper_request(chat_id).await;
                    }
                }

                OrderState::SizeRequested {
                    customer_name,
                    paper,
                    files,
                    ..
                } => {
                    let size: usize = message.message.parse().unwrap_or(0);
                    let sizes = self.prompt.sizes_vec(paper);
                    if size > 0 && size <= sizes.len() {
                        let size = sizes[size - 1].clone();
                        self.send_ready_request(chat_id.clone()).await;
                        let new_state = OrderState::SizeSelected {
                            chat_id,
                            customer_name: customer_name.clone(),
                            paper: paper.clone(),
                            size,
                            files: files.clone(),
                        };
                        self.repository.set_order(new_state);
                    } else {
                        self.send_size_request(chat_id.clone(), &paper).await;
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
            println!("Order updated in repo {:#?}", self.repository);
        } else {
            self.repository.set_order(OrderState::from_txt_msg(message));
            println!("Order created in repo {:#?}", self.repository);
            self.send_paper_request(chat_id).await;
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
}
