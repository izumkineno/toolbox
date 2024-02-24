use std::sync::mpsc::{channel, TryRecvError};
use std::thread;
use std::time::Duration;
use anyhow::{Context, Result};
use tauri::{ Manager };
use multi_tools_serialport::sp::Serial;
use multi_tools_serialport::sp_list::{list_available_ports, SerialInfo};
use serde_json::{json, Value};
use crate::manage::{MsgCode, MsgHandle, MsgHandles, SendHandles, Serials};

macro_rules! catch_error_to_string {
    ($func:ident, $( $x:expr ),*) => {
        match $func($( $x ),*).await {
            Ok(value) => Ok(value),
            Err(err) => {
                println!("{}", err.to_string());
                Err(err.to_string())
            }
        }
    };
}

// todo 将删除和断开分离

fn update_msg(app_handle: &tauri::AppHandle, id: &String, msg_send: Option<&Vec<u8>>) {
    let msg_handles = app_handle.state::<MsgHandles>();

    let msg = match msg_handles.0.lock().unwrap().get_mut(id)  {
        None => {
            json!({
                "recv_count": 0,
                "send_count": 0,
                "msg": "获取串口数据失败"
            })
        }
        Some(x) => {
            if let Some(m) = msg_send {
                x.add_send_count(m);
            }
            let recv_count = x.recv_count;
            let send_count = x.send_count;
            let msg_ = x.recv_buffer_to_string();
            json!({
                "recv_count": recv_count,
                "send_count": send_count,
                "msg": msg_
            })
        }
    };
    app_handle.emit_all(&format!("recv_{id}"), msg).unwrap_or_default();
}

async fn _connect(app_handle: tauri::AppHandle, id: &str, port: impl AsRef<str>, br: u32) -> Result<String> {
    let p = Serial::new(port.as_ref(), br).connect()?;
    // 10微秒收一次数据
    let timeout = Duration::from_micros(1);
    let recv = p.thread_recv_init(timeout)?;
    let send = p.thread_send_init()?;

    // 暂存 统计句柄
    let msg_handle = MsgHandle::new();
    let msg_handles = app_handle.state::<MsgHandles>();
    let mut msg_handles_guard = msg_handles.0.lock().unwrap();
    msg_handles_guard.insert(id.to_string(), msg_handle);

    // 创建接收线程
    let id_str = id.to_string();
    let app_handle_clone = app_handle.clone();
    thread::spawn(move || {
        let msg_handles = app_handle_clone.state::<MsgHandles>();
        loop {
            match recv.try_recv() {
                Ok((v, s)) => {
                    if let Some(x) = msg_handles.0.lock().unwrap().get_mut(&id_str) {
                        x.add_buffer(v[0..s].to_vec());
                    }
                    update_msg(&app_handle_clone, &id_str, None);
                    // dbg!(msg_handles.0.lock().unwrap());
                },
                Err(TryRecvError::Empty) => {
                    thread::sleep(Duration::from_micros(1));
                    continue;
                }
                Err(TryRecvError::Disconnected) => {
                    break;
                }
            }
        }
    });

    // 构建发送监听事件 todo 可变发送字符编码，默认utf8
    let id_str = id.to_string();
    let app_handle_clone = app_handle.clone();
    let app_handle_clone_1 = app_handle.clone();
    let send_e = app_handle_clone.listen_global(format!("send_{id_str}"), move |event| {
        let v = event.payload().unwrap();
        let v_json: Value = serde_json::from_str(v).unwrap();
        let v_json = v_json.as_object().unwrap();

        // 获取发送类型
        let msg_type = v_json.get("type").unwrap().as_str().unwrap_or("");
        // 是否循环
        let msg_loop = v_json.get("loop").unwrap().as_bool().unwrap_or(false);
        // 获取发送内容
        let msg = match msg_type {
            "hex" => {
                v_json.get("msg").unwrap().as_array().unwrap()
                    .iter().map(|v| v.as_number().unwrap().as_u64().unwrap() as u8).collect::<Vec<u8>>()
            }
            _ => {
                let msg = v_json.get("msg").unwrap().as_str();
                if let Some(msg) = msg {
                    Vec::from(msg.as_bytes())
                } else {
                    Vec::new()
                }
            }
        };

        // dbg!(&v_json, &msg_type, &msg_loop, &msg);


        if msg_loop {
            let delay = v_json.get("loop_time").unwrap().as_u64().unwrap_or(100);
            // dbg!(delay);
            let id_str = id_str.clone();
            let send = send.clone();
            let (tx, rx) = channel();
            let app_handle_clone_2 = app_handle_clone_1.clone();
            // 建立关闭循环监听事件
            app_handle_clone_1.once_global(format!("send_loop_{id_str}"), move |_| {
                tx.send(true).unwrap();
            });
            thread::spawn(move || loop {
                let app = app_handle_clone_2.clone();
                match rx.try_recv() {
                    Ok(v) => {
                        if v { break; }
                    }
                    Err(TryRecvError::Empty) => {
                        thread::sleep(Duration::from_millis(delay));
                        update_msg(&app, &id_str, Some(&msg));
                        send.send(msg.clone()).unwrap_or_default();
                        continue;
                    }
                    Err(TryRecvError::Disconnected) => {
                        break;
                    }
                }
            });
        } else {
            update_msg(&app_handle_clone_1, &id_str, Some(&msg));
            send.send(msg).unwrap_or_default();
        }

    });

    // 暂存 发送句柄，用于关闭发送事件监听
    let send_handles = app_handle.state::<SendHandles>();
    let mut send_handles = send_handles.0.lock().unwrap();
    send_handles.insert(id.to_string(), send_e);

    // 暂存 串口句柄，用于断开连接
    let serials = app_handle.state::<Serials>();
    let mut serials = serials.0.lock().unwrap();
    serials.insert(id.to_string(), p);

    Ok(format!("{} 连接成功", port.as_ref()))
}
#[tauri::command]
pub async fn connect(app_handle: tauri::AppHandle, id: &str, port: &str, br: u32) -> Result<String, String> {
    catch_error_to_string!(_connect, app_handle, id, port, br)
}


async fn _disconnect(app_handle: tauri::AppHandle, id: &str) -> Result<String> {
    let serials = app_handle.state::<Serials>();
    let mut serials = serials.0.lock().unwrap();

    let send_handles = app_handle.state::<SendHandles>();
    let mut send_handles = send_handles.0.lock().unwrap();

    let msg_handles = app_handle.state::<MsgHandles>();
    let mut msg_handles = msg_handles.0.lock().unwrap();

    let p = serials.get(id).context("未找到串口")?;
    if p.is_connected()? {
        p.disconnect()?;
        // 取消监听事件
        app_handle.unlisten(send_handles.get(id).unwrap().to_owned());
        // 移除发送句柄
        send_handles.remove(id);
        // 移除统计句柄
        msg_handles.remove(id);
        // 移除串口句柄
        serials.remove(id);
    }
    Ok("断开成功".to_string())
}
#[tauri::command]
pub async fn disconnect(app_handle: tauri::AppHandle, id: &str) -> Result<String, String> {
    catch_error_to_string!(_disconnect, app_handle, id)
}


#[tauri::command]
pub async fn get_serial_ports() -> Result<Vec<SerialInfo>, String> {
    match list_available_ports() {
        Ok(v) => Ok(v),
        Err(e) => Err(e.to_string())
    }
}


// 控制命令
pub async fn _set_recv_setting(app_handle: tauri::AppHandle, id: String, item: u32, value: i64) -> Result<()> {

    {
        let msg_handles = app_handle.state::<MsgHandles>();
        let mut msg_handle = msg_handles.0.lock().unwrap();
        let msg_handle = msg_handle.get_mut(&id).context("未找到指定id")?;

        match item {
            // 0 清理缓冲区， 1 清理接收计数， 2 清理发送计数
            0 => {
                match value {
                    0 => msg_handle.clear_buffer(),
                    1 => msg_handle.clear_count(true),
                    2 => msg_handle.clear_count(false),
                    _ => {}
                }
            }
            // 显示时间
            101 => {
                match value {
                    1 => msg_handle.set_display_show_time(true),
                    _ => msg_handle.set_display_show_time(false)
                }
            }
            // 显示十六进制
            102 => {
                match value {
                    1 => msg_handle.set_display_hex(true),
                    _ => msg_handle.set_display_hex(false)
                }
            }
            // 显示编码
            103 => {
                match value {
                    1 => msg_handle.set_display_code(MsgCode::GBK),
                    _ => msg_handle.set_display_code(MsgCode::UTF8)
                }
            }
            104 => {
                match value {
                    x if x < 0 => msg_handle.set_recv_len(None),
                    x if x >= 0 => msg_handle.set_recv_len(Some(x as usize as u32)),
                    _ => msg_handle.set_recv_len(None)
                }
            }
            _ => {}
        }
    }

    update_msg(&app_handle, &id, None);

    Ok(())
}
#[tauri::command]
pub async fn set_recv_setting(app_handle: tauri::AppHandle, id: String, item: u32, value: i64) -> Result<(), String> {
    catch_error_to_string!(_set_recv_setting, app_handle, id, item, value)
}