use anyhow::{anyhow, Result};
use log::{error, info};
use retour::static_detour;

use crate::offsets::OFFSET_ON_RECV_MESSAGE;

static_detour! {
  static Test: /* extern "X" */ fn(i64,i64,i64);
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
            |a, b, c| {
                info!("Call on recv: 0x{:x}, 0x{:x}, 0x{:x}", a, b, c);
                Test.call(a, b, c)
            },
        )?
    };
    unsafe { Test.enable() }?;

    info!("hook installed");

    Ok(HookGuard {})
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
