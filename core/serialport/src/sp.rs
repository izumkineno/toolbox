use std::io::Read;
use std::sync::{Arc, LockResult, RwLock};
use std::time::Duration;
use serialport::{SerialPort, SerialPortBuilder};
use anyhow::{Context, format_err, Result};
use tokio::io;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub fn rw_error_handler<T>(result: LockResult<T>) -> Result<T> {
    match result {
        Ok(t) => Ok(t),
        Err(e) => {
            return Err(format_err!(e.to_string()))
        }
    }
}

pub struct Serial {
    pub config: SerialPortBuilder,
    port: Option<Box<dyn SerialPort>>,
    connected: Arc<RwLock<bool>>,
    buffer_size: usize,
}

impl Serial {
    pub fn new<'a>(path: impl Into<std::borrow::Cow<'a, str>>, baud_rate: u32) -> Self {
        Self {
            config: serialport::new(path, baud_rate).timeout(Duration::from_millis(100)),
            port: None,
            connected: Arc::new(RwLock::new(false)),
            buffer_size: 2048,
        }
    }

    pub fn set_buffer_size(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
    }

    pub fn connect(mut self) -> Result<Self> {
        let port = self.config.clone().open()?;
        self.port = Some(port);
        // 初始化连接状态
        let mut connected = rw_error_handler(self.connected.write())?;
        *connected = true;
        drop(connected);

        Ok(self)
    }

    pub fn disconnect(&self) -> Result<()> {
        let mut connected = rw_error_handler(self.connected.write())?;
        *connected = false;
        Ok(())
    }

    pub fn is_connected(&self) -> Result<bool> {
        let connected = rw_error_handler(self.connected.read())?;
        Ok(*connected)
    }

    fn get_port_info(&self) -> Result<(Box<dyn SerialPort>, Arc<RwLock<bool>>, usize)> {
        // 判断是否连接
        if !self.is_connected()? {
            return Err(format_err!("串口未连接"))
        }
        // 获取连接状态
        let connected = self.connected.clone();
        // 获取缓冲区大小
        let buffer_size = self.buffer_size;
        // 获取接口
        let port = self.port.as_ref()
            .context("串口未打开")?
            .try_clone()?;
        Ok((port, connected, buffer_size))
    }
}

impl Serial {
    pub fn thread_recv_init_async(&self) -> Result<Receiver<(Vec<u8>, usize)>> {
        let (mut port, connected, buffer_size) = self.get_port_info()?;
        // 创建通道
        let (tx, rx) = channel::<(Vec<u8>, usize)>(buffer_size);
        // 开启线程
        tokio::spawn(async move {
            let mut buf = vec![0u8; buffer_size];
            loop {
                // 读取连接状态
                if let Ok(conn) = rw_error_handler(connected.read()) {
                    if !*conn {
                        break
                    }
                }
                // 读取数据
                match port.read(&mut buf) {
                    Ok(bytes) => {
                        if bytes > 0 {
                            if tx.send((buf.clone(), bytes)).await.is_err() {
                                eprintln!("接收线程转发数据失败")
                            };
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                        tokio::time::sleep(Duration::from_nanos(10)).await;
                        continue
                    },
                    Err(e) => eprintln!("接收线程读取数据失败：{:?}", e),
                }
            }
        });

        Ok(rx)
    }

    pub fn thread_send_init_async(&self) -> Result<Sender<Vec<u8>>> {
        let (mut port, connected, buffer_size) = self.get_port_info()?;
        // 创建通道
        let (tx, mut rx) = channel::<Vec<u8>>(buffer_size);
        // 开启线程
        tokio::spawn(async move {
            loop {
                if let Ok(lock) = connected.read() {
                    if !*lock {
                        break;
                    }
                }
                match rx.try_recv() {
                    Ok(msg) => {
                        if port.write(msg.as_slice()).is_err() {
                            eprintln!("发送线程发送数据失败")
                        };
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                        tokio::time::sleep(Duration::from_nanos(10)).await;
                        continue;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
        });
        Ok(tx)
    }

    pub fn thread_recv_init(&self, get_recv_timeout: Duration) -> Result<std::sync::mpsc::Receiver<(Vec<u8>, usize)>> {
        let (mut port, connected, buffer_size) = self.get_port_info()?;
        // 创建通道
        let (tx, rx) = std::sync::mpsc::channel::<(Vec<u8>, usize)>();
        // 开启线程
        std::thread::spawn( move ||{
            let mut buf = vec![0u8; buffer_size];
            loop {
                // 读取连接状态
                if let Ok(conn) = rw_error_handler(connected.read()) {
                    if !*conn {
                        break
                    }
                }
                // 读取数据
                match port.read(&mut buf) {
                    Ok(bytes) => {
                        if bytes > 0 {
                            if tx.send((buf.clone(), bytes)).is_err() {
                                eprintln!("接收线程转发数据失败")
                            };
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                        std::thread::sleep(get_recv_timeout);
                        continue
                    },
                    Err(e) => eprintln!("接收线程读取数据失败：{:?}", e),
                }
            }
        });

        Ok(rx)
    }

    pub fn thread_send_init(&self) -> Result<std::sync::mpsc::Sender<Vec<u8>>> {
        let (mut port, connected, _) = self.get_port_info()?;
        // 创建通道
        let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
        // 开启线程
        std::thread::spawn( move || {

            loop {
                if let Ok(lock) = connected.read() {
                    if !*lock {
                        break;
                    }
                }
                match rx.try_recv() {
                    Ok(msg) => {
                        if port.write(msg.as_slice()).is_err() {
                            eprintln!("发送线程发送数据失败")
                        };
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        std::thread::sleep(Duration::from_nanos(10));
                        continue;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
        });
        Ok(tx)
    }

    pub fn send_once(&self, msg: Vec<u8>) -> Result<()> {
        let (mut port, _, _) = self.get_port_info()?;
        if port.write(msg.as_slice()).is_err() {
            eprintln!("发送数据失败")
        }
        Ok(())
    }
}
