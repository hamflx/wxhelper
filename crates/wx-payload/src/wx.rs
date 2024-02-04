use std::marker::PhantomData;

use log::info;
use widestring::{U16Str, U16String};
use windows::Win32::{
    Foundation::BOOL,
    System::Threading::{OpenProcess, PROCESS_ALL_ACCESS},
};

type SendTextMsg = extern "system" fn(u64, u64, u64, u64, u64, u64, u64, u64) -> u64;
type GetSendMessageMgr = extern "system" fn() -> u64;

#[repr(C)]
pub(crate) struct WeChatString<'a> {
    ptr: *const u16,
    pub(crate) length: u32,
    pub(crate) max_length: u32,
    c_ptr: u64,
    c_len: u32,
    phantom: PhantomData<&'a U16Str>,
}

impl<'a> WeChatString<'a> {
    fn new(text: &'a U16Str) -> Self {
        Self {
            ptr: text.as_ptr(),
            length: text.len() as _,
            max_length: text.len() as _,
            c_ptr: 0,
            c_len: 0,
            phantom: PhantomData,
        }
    }
}

pub fn send_text_message(message: &str) -> Result<(), String> {
    let mut sys = sysinfo::System::new_all();
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
        .join("[3.9.5.81]")
        .join("WeChatWin.dll");
    info!("WeChatWin.dll => {}", we_chat_win_path.display());

    let lib = injectors::library::Library::from_filename(
        we_chat_win_path
            .to_str()
            .ok_or_else(|| format!("No we_chat_win_path"))?,
    )
    .map_err(|err| match err.code() {
        Some(code) => format!("{}", std::io::Error::from_raw_os_error(code as _)),
        None => format!("{err}"),
    })?;
    let lib_base = lib.module_base();
    info!("lib_base => 0x{:x}", lib_base);

    let kSendTextMsg = 0xfcd8d0;
    let kGetSendMessageMgr = 0x8c00e0;
    let send: SendTextMsg = unsafe { std::mem::transmute(lib_base + kSendTextMsg) };
    let mgr: GetSendMessageMgr = unsafe { std::mem::transmute(lib_base + kGetSendMessageMgr) };
    info!("send => {:?}", send);
    info!("mgr => {:?}", mgr);

    mgr();
    info!("mgr success");

    let chat_msg = [0u8; 0x460];
    let temp = [0u64; 3];
    let to_user = U16String::from_str("filehelper");
    let to_user = WeChatString::new(to_user.as_ustr());
    let text_msg = U16String::from_str(message);
    let text_msg = WeChatString::new(text_msg.as_ustr());
    send(
        chat_msg.as_ptr() as _,
        &to_user as *const _ as _,
        &text_msg as *const _ as _,
        temp.as_ptr() as _,
        1,
        1,
        0,
        0,
    );
    info!("send success");

    Ok(())
}
