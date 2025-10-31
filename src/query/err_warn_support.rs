#[derive(Debug)]
pub struct Message {
    pub msg_type: MessageSeverity,
    pub msg_content: String,
    pub byte_pos: usize,
    pub source_phase_index: usize,
}

#[derive(Debug)]
pub enum MessageSeverity {
    Warning,
    Error
}

pub trait MessageSink {
    fn send(&self, msg: Message);
}

impl<T: MessageSink> MessageSink for &T {
    fn send(&self, msg: Message) {
        T::send(self, msg);
    }
}

pub struct IgnoreMessages;


impl MessageSink for IgnoreMessages {
    fn send(&self, msg: Message) {
        drop(msg);
    }
}

#[cfg(debug_assertions)]
pub struct DebugPrintMessages;

#[cfg(debug_assertions)]
impl MessageSink for DebugPrintMessages {
    fn send(&self, msg: Message) {
        eprintln!("QUERY LANG {:?} at character {}: {}", msg.msg_type, msg.byte_pos, msg.msg_content);
    }
}