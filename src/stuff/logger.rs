use crate::stuff::transport::Transport;
use log::{Level, Metadata, Record};

pub struct Logger<'a, T>
where
    T: Transport + 'a,
{
    transport: &'a T,
}

impl<'a, T> Logger<T>
where
    T: Transport + 'a,
{
    pub fn new(transport: &'a T) -> Self {
        Logger { transport }
    }
}

impl<'a, T> log::Log for Logger<T>
where
    T: Transport + 'a,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    async fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{} - {}", record.level(), record.args());
            self.transport.log(msg).await;
        }
    }

    fn flush(&self) {}
}
