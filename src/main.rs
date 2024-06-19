#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cpu;
mod err;
mod event;
mod icon_resource;
mod run_cat_tray;
mod util;
mod gif;

use err::RunCatTrayError;
use event::{RunCatTrayEvent, EVENT_LOOP_PROXY};
use run_cat_tray::RunCatTray;

use winit::event_loop::{ControlFlow, EventLoop};

#[tokio::main(worker_threads = 2)]
async fn main() -> Result<(), RunCatTrayError> {
    let event_loop = EventLoop::<RunCatTrayEvent>::with_user_event()
        .build()
        .map_err(|_| RunCatTrayError::RunAppFailed("Can't start the event loop".to_string()))?;

    event_loop.set_control_flow(ControlFlow::Wait);

    *EVENT_LOOP_PROXY.lock() = Some(event_loop.create_proxy());
    let mut app = RunCatTray::new()?;

    event_loop
        .run_app(&mut app)
        .map_err(|_| RunCatTrayError::RunAppFailed("RunCat app start failed.".to_string()))?;

    Ok(())
}
