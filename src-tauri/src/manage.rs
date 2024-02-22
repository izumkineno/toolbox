use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::EventHandler;
use multi_tools_serialport::sp::Serial;
use anyhow::Result;


// 串口句柄， key为串口UI实例ID
pub struct Serials(pub Arc<Mutex<HashMap<String, Serial>>>);

impl Serials {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}

// 发送句柄， key为串口UI实例ID
pub struct SendHandles(pub Arc<Mutex<HashMap<String, EventHandler>>>);

impl SendHandles {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}



#[derive(Clone, Debug)]
pub enum MsgCode {
    GBK,
    UTF8,
}

impl MsgCode {
    pub fn to_code_string(&self, msg: &Vec<u8>) -> Result<String> {
        Ok(
            match self {
                MsgCode::GBK => encoding_rs::GBK.decode(msg.as_slice()).0.to_string(),
                MsgCode::UTF8 => String::from_utf8(msg.clone())?,
            }
        )
    }
}

#[derive(Clone, Debug)]
struct BufferTime {
    buffer: Vec<u8>,
    time: String,
}

#[derive(Clone, Debug)]
pub struct MsgHandle {
    recv_buffer: Vec<BufferTime>,
    recv_show_time: bool,
    recv_hex: bool,
    recv_code: MsgCode,
    pub(crate) recv_count: u32,
    pub(crate) send_count: u32,
}

impl MsgHandle {
    pub fn new() -> Self {
        Self {
            recv_buffer: Vec::new(),
            recv_show_time: false,
            recv_hex: false,
            recv_code: MsgCode::UTF8,
            recv_count: 0,
            send_count: 0,
        }
    }

    pub fn set_display_show_time(&mut self, is_show: bool) {
        self.recv_show_time = is_show;
    }

    pub fn set_display_hex(&mut self, is_hex: bool) {
        self.recv_hex = is_hex;
    }

    pub fn set_display_code(&mut self, code: MsgCode) {
        self.recv_code = code;
    }

    pub fn clear_count(&mut self, is_recv: bool) {
        if is_recv {
            self.recv_count = 0;
        } else {
            self.send_count = 0;
        }
    }

    pub fn clear_buffer(&mut self) {
        self.recv_buffer.clear();
    }

    pub fn add_buffer(&mut self, buffer: Vec<u8>){
        self.recv_count += buffer.len() as u32;
        self.recv_buffer.push(BufferTime{
            buffer,
            time: format!("{}", chrono::Local::now().format("%H:%M:%S%.6f")),
        });
    }

    pub fn add_send_count(&mut self, buffer: &Vec<u8>) {
        self.send_count += buffer.len() as u32;
    }

    pub fn recv_buffer_to_string(&self) -> String {
        let show_time = self.recv_show_time;
        let hex = self.recv_hex;
        let buffer = self.recv_buffer.iter().map(|v| {
            match [show_time, hex] {
                [true, true] => {
                    format!("<strong>[{}]</strong>",v.time.clone()) + ": " + &v.buffer.iter().map(|v| format!("{:02X}", v)).collect::<Vec<String>>().join(" ") + "\r\n"
                },
                [true, false] => {
                    format!("<strong>[{}]</strong>",v.time.clone()) + ": " + &self.recv_code.to_code_string(&v.buffer).unwrap_or_default() + "\r\n"
                },
                [false, true] => {
                    v.buffer.iter().map(|v| format!("{:02X}", v)).collect::<Vec<String>>().join(" ")
                },
                [false, false] => {
                    self.recv_code.to_code_string(&v.buffer).unwrap_or_default()
                },
            }
        }).collect::<Vec<String>>();
        buffer.join(if hex && !show_time { " " } else { "" })
    }
}

// 统计句柄， key为串口UI实例ID
pub struct MsgHandles(pub Arc<Mutex<HashMap<String, MsgHandle>>>);

impl MsgHandles {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::thread;

    const BUFFER_UTF8: [u8;6] = [0xE5, 0x95, 0x8A, 0xE5, 0x95, 0x8A];
    const BUFFER_GBK: [u8;4] = [0xB0, 0xA1, 0xB0, 0xA1];

    #[test]
    fn test_add_buffer() {
        let mut handle = MsgHandle::new();
        handle.add_buffer(BUFFER_UTF8.to_vec());
        dbg!(&handle);
    }

    #[test]
    fn test_recv_buffer_to_string() {
        let mut handle = MsgHandle::new();
        handle.add_buffer(BUFFER_UTF8.to_vec());
        handle.add_buffer(BUFFER_GBK.to_vec());

        handle.set_display_show_time(true);
        handle.set_display_hex(true);
        dbg!(&handle.recv_buffer_to_string());
        handle.set_display_show_time(true);
        handle.set_display_hex(false);
        dbg!(&handle.recv_buffer_to_string());
        handle.set_display_show_time(false);
        handle.set_display_hex(true);
        dbg!(&handle.recv_buffer_to_string());
        handle.set_display_show_time(false);
        handle.set_display_hex(false);
        dbg!(&handle.recv_buffer_to_string());

        handle.set_display_hex(false);
        handle.set_display_code(MsgCode::GBK);
        dbg!(&handle.recv_buffer_to_string());

    }

    #[test]
    fn test_handles() {
        let mut handles = MsgHandles::new();
        handles.0.lock().unwrap().insert("test".to_string(), MsgHandle::new());

        let mut hand_clone = handles.0.clone();
        let t1 = thread::spawn(move || {
            if let Some(x) = hand_clone.lock().unwrap().get_mut("test") {
                x.add_buffer(BUFFER_UTF8.to_vec());
                x.recv_count = 98999;
                x.set_display_code(MsgCode::GBK);
            }
        });

        let _ = t1.join();

        dbg!(handles.0.lock().unwrap().get("test").unwrap());
    }
}