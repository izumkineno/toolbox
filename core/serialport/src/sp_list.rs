use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct SerialInfo {
    pub name: String,
    pub state: bool,
}

pub fn list_available_ports() -> Result<Vec<SerialInfo>> {
    let mut ports_detail: Vec<SerialInfo> = vec![];
    for p in serialport::available_ports()? {

        // todo 处理检查慢的原因
        // let mut state = false;
        // if serialport::new(&p.port_name, 115200).open().is_ok() {
        //     state = true;
        // }

        let state = true;

        ports_detail.push(SerialInfo {
            name: p.port_name,
            state,
        });
    }

    Ok(ports_detail)
}
