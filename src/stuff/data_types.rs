use crate::stuff::error::{Error, Result};

#[derive(Debug, Clone)]
pub enum Message {
    Text(ReceivedMessage),
    Image(ReceivedMessage),
    Empty,
}

#[derive(Debug, Clone)]
pub struct ReceivedMessage {
    pub chat_id: String,
    pub customer_name: String,
    pub message: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OrderState {
    RaperRequested {
        chat_id: String,
        customer_name: String,
        files: Vec<String>,
    },
    SizeRequested {
        chat_id: String,
        customer_name: String,
        paper: String,
        files: Vec<String>,
    },
    SizeSelected {
        chat_id: String,
        customer_name: String,
        paper: String,
        size: String,
        files: Vec<String>,
    },
}

impl OrderState {
    pub fn from_img_msg(msg: ReceivedMessage) -> OrderState {
        OrderState::RaperRequested {
            chat_id: msg.chat_id,
            customer_name: msg.customer_name,
            files: vec![msg.message],
        }
    }
    pub fn from_txt_msg(msg: ReceivedMessage) -> OrderState {
        OrderState::RaperRequested {
            chat_id: msg.chat_id,
            customer_name: msg.customer_name,
            files: vec![],
        }
    }
    pub fn get_chat_id(&self) -> String {
        match self {
            OrderState::RaperRequested { chat_id, .. } => chat_id.to_string(),
            OrderState::SizeRequested { chat_id, .. } => chat_id.to_string(),
            OrderState::SizeSelected { chat_id, .. } => chat_id.to_string(),
        }
    }
    pub fn get_paper(&self) -> &str {
        match self {
            OrderState::RaperRequested { .. } => "",
            OrderState::SizeRequested { paper, .. } => paper,
            OrderState::SizeSelected { .. } => "",
        }
    }

    pub fn add_image(&mut self, url: String) {
        match self {
            OrderState::RaperRequested { files, .. } => {
                files.push(url);
            }
            OrderState::SizeRequested { files, .. } => {
                files.push(url);
            }
            OrderState::SizeSelected { files, .. } => {
                files.push(url);
            }
        }
    }

    pub fn into_order_with_paper(self, paper: String) -> Result<OrderState> {
        match self {
            OrderState::RaperRequested {
                chat_id,
                customer_name,
                files,
            } => Ok(OrderState::SizeRequested {
                chat_id,
                customer_name,
                paper,
                files,
            }),
            OrderState::SizeRequested { .. } => Err(Error::OrderWrongState),
            OrderState::SizeSelected { .. } => Err(Error::OrderWrongState),
        }
    }

    pub fn into_order_with_size(self, size: String) -> Result<OrderState> {
        match self {
            OrderState::RaperRequested { .. } => Err(Error::OrderWrongState),
            OrderState::SizeRequested {
                chat_id,
                customer_name,
                paper,
                files,
                ..
            } => Ok(OrderState::SizeSelected {
                chat_id,
                customer_name,
                paper,
                size,
                files,
            }),
            OrderState::SizeSelected { .. } => Err(Error::OrderWrongState),
        }
    }
}
