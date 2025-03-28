use crate::data_types::SendMessage;

#[derive(Debug, Clone)]
pub enum Message {
    Text(SendMessage),
    Image(SendMessage),
    Empty
}
