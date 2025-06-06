use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;
use tray_icon::menu::MenuEvent;
use winit::event_loop::EventLoopProxy;

pub(crate) static EVENT_LOOP_PROXY: Lazy<Arc<Mutex<Option<EventLoopProxy<RunCatTrayEvent>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

#[derive(Debug)]
pub(crate) enum RunCatTrayEvent {
    TrayMenuEvent(MenuEvent),
    SystemThemeChanged(dark_light::Mode),
    ChangeIconIndexEvent(usize),
}

pub(crate) fn send_menu_event() {
    MenuEvent::set_event_handler(Some(move |f| {
        if let Some(proxy) = EVENT_LOOP_PROXY.lock().as_ref() {
            proxy.send_event(RunCatTrayEvent::TrayMenuEvent(f)).unwrap();
        }
    }));
}
