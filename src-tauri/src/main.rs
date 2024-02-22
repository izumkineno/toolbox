// Prevents additional console Index on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command;
mod manage;

use tauri::Manager;
use window_shadows::set_shadow;
use crate::manage::{MsgHandles, SendHandles, Serials};
use crate::command::{set_recv_setting, connect, disconnect, get_serial_ports};

fn main() {

    tauri::Builder::default()
        .manage(Serials::new())
        .manage(SendHandles::new())
        .manage(MsgHandles::new())
        .invoke_handler(tauri::generate_handler![
            connect,
            disconnect,
            get_serial_ports,
            set_recv_setting,
        ])
        .setup(|app| {
            let window = app.get_window("multi_tools").unwrap();
            set_shadow(&window, true).expect("Unsupported platform!");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
