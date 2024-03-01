use std::mem::MaybeUninit;

use ropey::Rope;
use tokio::sync::mpsc::UnboundedSender;

static mut LOGGER: MaybeUninit<BufferLogger> = MaybeUninit::uninit();

pub struct BufferLogger {
    tx: UnboundedSender<Rope>,
}

impl log::Log for BufferLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let line = format!("{} - {}\n", record.level(), record.args());
        self.tx.send(Rope::from_str(&line)).ok();
    }

    fn flush(&self) {}
}

pub fn enable(tx: UnboundedSender<Rope>) {
    let logger = unsafe { LOGGER.write(BufferLogger { tx }) };

    log::set_logger(logger)
        .map(|_| log::set_max_level(log::LevelFilter::Trace))
        .ok();
}
