use std::io;

pub trait TextSender {
    fn send_text(&self, text: String, extracr:&str) -> io::Result<()>;
}