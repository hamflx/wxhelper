use std::path::Path;

use sysinfo::System;
use windows::Win32::{
    Foundation::BOOL,
    System::Threading::{OpenProcess, PROCESS_ALL_ACCESS},
};

fn main() {
    let mut sys = System::new_all();

    sys.refresh_processes();

    let process = sys.processes_by_name("WeChat.exe").next().unwrap();
    let pid = process.pid().as_u32();

    let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, BOOL(0), pid) }.unwrap();
    let handle = injectors::process::ProcessHandle::from_handle(handle.0);
    handle
        .inject_to_process(&None, env!("CARGO_CDYLIB_FILE_WX_PAYLOAD_wx-payload"))
        .map_err(|_| std::io::Error::last_os_error())
        .unwrap();
}
