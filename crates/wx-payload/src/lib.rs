use std::fs::File;

use log::{error, info, LevelFilter};
use simplelog::{Config, WriteLogger};
use sysinfo::System;
use windows::Win32::{
    Foundation::BOOL,
    System::Threading::{OpenProcess, PROCESS_ALL_ACCESS},
};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[no_mangle]
pub extern "system" fn enable_hook(_: usize) -> usize {
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("D:\\tmp\\my_rust_binary.log").unwrap(),
    )
    .unwrap();

    info!("inject successfully");

    match start() {
        Ok(_) => {}
        Err(err) => error!("Err: {err}"),
    }

    0
}

fn start() -> Result<(), String> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let process = sys
        .processes_by_name("WeChat.exe")
        .next()
        .ok_or_else(|| format!("No WeChat.exe"))?;
    let pid = process.pid().as_u32();
    info!("pid: {pid}");

    let handle =
        unsafe { OpenProcess(PROCESS_ALL_ACCESS, BOOL(0), pid) }.map_err(|err| format!("{err}"))?;
    let handle = injectors::process::ProcessHandle::from_handle(handle.0);

    let we_chat_win_path = process
        .exe()
        .ok_or_else(|| format!("No parent"))?
        .parent()
        .ok_or_else(|| format!("No parent"))?
        .join("[3.9.9.35]")
        .join("WeChatWin.dll");
    info!("WeChatWin.dll => {}", we_chat_win_path.display());

    let lib = injectors::library::Library::from_filename(
        we_chat_win_path
            .to_str()
            .ok_or_else(|| format!("No we_chat_win_path"))?,
    )
    .map_err(|err| format!("{err}"))?;
    let lib_base = lib.module_base();
    let kSendTextMsg = 0xfcd8d0;
    let send = lib_base + kSendTextMsg;
    info!("send => 0x{:x}", send);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
