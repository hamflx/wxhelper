use anyhow::{anyhow, Result};
use log::{error, info};
use retour::static_detour;
use widestring::{U16Str, U16String};

use crate::{offsets::OFFSET_ON_RECV_MESSAGE, wx::WeChatString};

static_detour! {
  static Test: /* extern "X" */ fn(i64, *const RecvParams, i64);
}

pub(crate) fn install_recv_hooks() -> Result<HookGuard> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_processes();
    let process = sys
        .processes_by_name("WeChat.exe")
        .next()
        .ok_or_else(|| anyhow!("No WeChat.exe"))?;
    let pid = process.pid().as_u32();
    info!("pid: {pid}");

    let we_chat_win_path = process
        .exe()
        .ok_or_else(|| anyhow!("No parent"))?
        .parent()
        .ok_or_else(|| anyhow!("No parent"))?
        .join("[3.9.5.81]")
        .join("WeChatWin.dll");
    info!("WeChatWin.dll => {}", we_chat_win_path.display());

    let lib = injectors::library::Library::from_filename(
        we_chat_win_path
            .to_str()
            .ok_or_else(|| anyhow!("No we_chat_win_path"))?,
    )
    .map_err(|err| match err.code() {
        Some(code) => anyhow!("{}", std::io::Error::from_raw_os_error(code as _)),
        None => anyhow!("{err}"),
    })?;
    let lib_base = lib.module_base();
    unsafe {
        Test.initialize(
            std::mem::transmute(lib_base + OFFSET_ON_RECV_MESSAGE),
            |a, recv_params, c| {
                info!("Call on recv: 0x{:x}, {:?}, 0x{:x}", a, recv_params, c);
                let params = recv_params.read();
                let from_user = params.from_user.read().to_string();
                let content = params.content.read().to_string();
                let full_content = params.full_content.read().to_string();
                info!("from_user: {}", from_user);
                info!("content: {}", content);
                info!("full_content: {}", full_content);
                Test.call(a, recv_params, c)
            },
        )?
    };
    unsafe { Test.enable() }?;

    info!("hook installed");

    Ok(HookGuard {})
}

#[repr(C)]
struct RecvParams {
    f1: usize,
    f2: usize,
    f3: usize,
    from_user: *const SkBuiltInString,
    f4: usize,
    to_user: *const SkBuiltInString,
    content: *const SkBuiltInString,
    f5: usize,
    f6: usize,
    signature: usize,
    full_content: *const WeChatStr,
}

#[repr(C)]
struct SkBuiltInString {
    f1: usize,
    inner_string: *const WeChatStr,
}

impl ToString for SkBuiltInString {
    fn to_string(&self) -> String {
        if self.inner_string.is_null() {
            String::new()
        } else {
            unsafe { self.inner_string.read().to_string() }
        }
    }
}

#[repr(C)]
struct WeChatStr {
    ptr: *const u8,
    f2: usize,
    len: usize,
    max_len: usize,
}

impl ToString for WeChatStr {
    fn to_string(&self) -> String {
        info!(
            "WeChatStr: {:?}, 0x{:x}, 0x{:x}, 0x{:x}",
            self.ptr, self.f2, self.len, self.max_len
        );
        if self.max_len | 0xf == 0xf {
            String::from_utf8_lossy(unsafe {
                std::slice::from_raw_parts(&self.ptr as *const _ as *const u8, self.len)
            })
            .into()
        } else {
            String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(self.ptr, self.len) })
                .into()
        }
    }
}

pub(crate) struct HookGuard {}

impl Drop for HookGuard {
    fn drop(&mut self) {
        match unsafe { Test.disable() } {
            Ok(_) => {
                info!("hook uninstalled")
            }
            Err(err) => error!("Unable to disable hook: {}", err),
        }
    }
}
