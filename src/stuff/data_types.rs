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
    pub fn get_chat_id(&self) -> String {
        match self {
            OrderState::RaperRequested { chat_id, .. } => chat_id.to_string(),
            OrderState::SizeRequested { chat_id, .. } => chat_id.to_string(),
            OrderState::SizeSelected { chat_id, .. } => chat_id.to_string(),
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
}
