use crate::stuff::transport::Transport;
use log::{Metadata, Record};

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
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    async fn log(&self, record: &Record) {
        self.transport.log(record.args().to_string()).await;
    }

    fn flush(&self) {}
}
