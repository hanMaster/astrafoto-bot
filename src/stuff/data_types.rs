use crate::stuff::error::{Error, Result};
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

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
        repeats: i32,
        last_msg_time: SystemTime,
    },
    SizeRequested {
        chat_id: String,
        customer_name: String,
        paper: String,
        files: Vec<String>,
        repeats: i32,
        last_msg_time: SystemTime,
    },
    SizeSelected {
        chat_id: String,
        customer_name: String,
        paper: String,
        size: String,
        price: i32,
        files: Vec<String>,
        repeats: i32,
        last_msg_time: SystemTime,
    },
}

impl OrderState {
    pub fn from_img_msg(msg: ReceivedMessage) -> OrderState {
        OrderState::RaperRequested {
            chat_id: msg.chat_id,
            customer_name: msg.customer_name,
            files: vec![msg.message],
            repeats: 0,
            last_msg_time: SystemTime::now(),
        }
    }

    pub fn from_txt_msg(msg: ReceivedMessage) -> OrderState {
        OrderState::RaperRequested {
            chat_id: msg.chat_id,
            customer_name: msg.customer_name,
            files: vec![],
            repeats: 0,
            last_msg_time: SystemTime::now(),
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

    pub fn last_time_sec(&self) -> u64 {
        match self {
            OrderState::RaperRequested { last_msg_time, .. } => {
                last_msg_time.elapsed().unwrap().as_secs()
            }
            OrderState::SizeRequested { last_msg_time, .. } => {
                last_msg_time.elapsed().unwrap().as_secs()
            }
            OrderState::SizeSelected { last_msg_time, .. } => {
                last_msg_time.elapsed().unwrap().as_secs()
            }
        }
    }

    pub fn repeats(&self) -> i32 {
        match self {
            OrderState::RaperRequested { repeats, .. } => *repeats,
            OrderState::SizeRequested { repeats, .. } => *repeats,
            OrderState::SizeSelected { repeats, .. } => *repeats,
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

    pub fn have_files(&self) -> bool {
        match self {
            OrderState::RaperRequested { files, .. } => !files.is_empty(),
            OrderState::SizeRequested { files, .. } => !files.is_empty(),
            OrderState::SizeSelected { files, .. } => !files.is_empty(),
        }
    }

    pub fn into_order_with_paper(self, paper: String) -> Result<OrderState> {
        match self {
            OrderState::RaperRequested {
                chat_id,
                customer_name,
                files,
                ..
            } => Ok(OrderState::SizeRequested {
                chat_id,
                customer_name,
                paper,
                files,
                repeats: 0,
                last_msg_time: SystemTime::now(),
            }),
            OrderState::SizeRequested { .. } => Err(Error::OrderWrongState),
            OrderState::SizeSelected { .. } => Err(Error::OrderWrongState),
        }
    }

    pub fn into_order_with_size(self, size: String, price: i32) -> Result<OrderState> {
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
                price,
                files,
                repeats: 0,
                last_msg_time: SystemTime::now(),
            }),
            OrderState::SizeSelected { .. } => Err(Error::OrderWrongState),
        }
    }

    pub fn requested(&mut self) {
        match self {
            OrderState::RaperRequested {
                repeats,
                last_msg_time,
                ..
            } => {
                *repeats += 1;
                *last_msg_time = SystemTime::now();
            }
            OrderState::SizeRequested {
                repeats,
                last_msg_time,
                ..
            } => {
                *repeats += 1;
                *last_msg_time = SystemTime::now();
            }
            OrderState::SizeSelected {
                repeats,
                last_msg_time,
                ..
            } => {
                *repeats += 1;
                *last_msg_time = SystemTime::now();
            }
        }
    }
}

impl Display for OrderState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderState::RaperRequested { .. } => {
                unimplemented!()
            }
            OrderState::SizeRequested { .. } => {
                unimplemented!()
            }
            OrderState::SizeSelected {
                chat_id,
                customer_name,
                paper,
                size,
                files,
                ..
            } => {
                let phone = chat_id.split('@').collect::<Vec<&str>>()[0];
                write!(
                    f,
                    "Телефон: {phone}\nИмя: {}\nТип бумаги: {}\nРазмер: {}\nФайлы: {:?}",
                    customer_name, paper, size, files
                )
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OrderMessage {
    pub phone: String,
    pub name: String,
    pub paper_type: String,
    pub paper_size: String,
    pub price: i32,
    pub files: Vec<String>,
}

impl From<OrderState> for OrderMessage {
    fn from(order: OrderState) -> Self {
        match order {
            OrderState::RaperRequested { .. } => {
                unreachable!()
            }
            OrderState::SizeRequested { .. } => {
                unreachable!()
            }
            OrderState::SizeSelected {
                chat_id,
                customer_name,
                paper,
                size,
                price,
                files,
                ..
            } => {
                let phone = chat_id.split('@').collect::<Vec<&str>>()[0];
                Self {
                    phone: phone.to_string(),
                    name: customer_name,
                    paper_type: paper,
                    paper_size: size,
                    price,
                    files,
                }
            }
        }
    }
}
