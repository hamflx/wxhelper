use std::path::Path;

use anyhow::{anyhow, Result};
use log::{error, info};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use sysinfo::System;
use tempfile::tempdir;
use windows::Win32::{
    Foundation::BOOL,
    System::Threading::{OpenProcess, PROCESS_ALL_ACCESS},
};

const PAYLOAD: &[u8] = include_bytes!(env!("CARGO_CDYLIB_FILE_WX_PAYLOAD_wx-payload"));

fn main() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    info!("Search WeChat.exe ...");

    match start() {
        Ok(_) => info!("Exit"),
        Err(err) => error!("Error: {err}"),
    }
}

fn start() -> Result<()> {
    let dir = tempdir()?;
    let dll_path = dir.path().join("wx-extension.dll");
    std::fs::write(&dll_path, PAYLOAD)?;

    let mut sys = System::new_all();
    sys.refresh_processes();
    let process = sys
        .processes_by_name("WeChat.exe")
        .next()
        .ok_or_else(|| anyhow!("No WeChat process found"))?;
    let pid = process.pid().as_u32();
    let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, BOOL(0), pid) }.unwrap();
    let handle = injectors::process::ProcessHandle::from_handle(handle.0);
    handle
        .inject_to_process(
            &None,
            std::str::from_utf8(dll_path.as_os_str().as_encoded_bytes())?,
        )
        .map_err(|_| std::io::Error::last_os_error())?;

    Ok(())
}
