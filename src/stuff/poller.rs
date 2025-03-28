use crate::stuff::transport::Transport;
use crate::stuff::error::Result;

pub struct Poller<T: Transport> where T: Transport {
    transport: T,
}
impl<T: Transport> Poller<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub async fn start_polling(&self) -> Result<()> {
        loop {
            let msg = self.transport.receive_message().await?;
            println!("Message: {:?}", msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::stuff::transport::WhatsApp;
    use super::*;

    #[tokio::test]
    async fn test_poll() {
        let transport = WhatsApp::new();
        let res = Poller::new(transport).start_polling().await;
    }
}