mod server;
mod wx;

use std::fs::File;

use log::{error, info, LevelFilter};
use simplelog::{Config, WriteLogger};

use crate::server::start_http_server;

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
    let rt = actix_web::rt::Runtime::new().map_err(|err| format!("{err}"))?;
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<&'static str>(1);
    let server = start_http_server(sender).unwrap();
    let handle = server.handle();
    let server_task = rt.spawn(server);
    let shutdown_task = rt.spawn(async move {
        let _ = receiver.recv().await;
        handle.stop(true).await;
    });
    let (r, _) = rt.block_on(async { tokio::join!(server_task, shutdown_task) });
    r.map_err(|err| format!("{err}"))
        .and_then(|r| r.map_err(|err| format!("{err}")))
}
