#![allow(dead_code, unused_variables, unused_imports)]

use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tray_icon::{
    menu::{IsMenuItem, Menu, MenuEvent, MenuEventReceiver, MenuId, MenuItem},
    TrayIcon, TrayIconBuilder, TrayIconEvent, TrayIconEventReceiver,
};
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::Theme,
};

static EVENT_LOOP_PROXY: Lazy<Arc<Mutex<Option<EventLoopProxy<RunCatTrayEvent>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub(crate) struct RunCatTray {
    tray_menu: Menu,
    exit_menu_item: MenuItem,
    auto_fit_theme_menu_item: MenuItem,
    toggle_theme_menu_item: MenuItem,
    // menu_items: Vec<(MenuItem, fn(e: MenuEvent))>,
    icon_path: (&'static str, &'static str),
    tary_icon: Option<TrayIcon>,
    curr_theme: dark_light::Mode,
    auto_fit_theme: bool,
}

#[derive(Debug)]
pub(crate) enum RunCatTrayEvent {
    TrayMenuEvent(MenuEvent),
    SystemThemeChanged(dark_light::Mode),
}

impl RunCatTray {
    fn new(icon_path: (&'static str, &'static str)) -> Self {
        let auto_fit_theme = MenuItem::new("auto fit theme: true", true, None);
        let toggle_theme = MenuItem::new("toggle theme", false, None);
        let exit = MenuItem::new("exit", true, None);

        Self {
            tray_menu: Menu::with_items(&[&auto_fit_theme, &toggle_theme, &exit]).unwrap(),
            auto_fit_theme_menu_item: auto_fit_theme,
            toggle_theme_menu_item: toggle_theme,
            exit_menu_item: exit,
            // menu_items: vec![],
            icon_path,
            tary_icon: None,

            curr_theme: dark_light::detect(),
            auto_fit_theme: true,
        }
    }

    // fn add_menu_item(&mut self, item: MenuItem, handler: fn(e: MenuEvent)) {
    //     self.menu_items.push((item, handler));
    // }

    fn on_theme_changed(&mut self) {
        if let Some(tray_icon) = self.tary_icon.as_mut() {
            let icon = if self.curr_theme == dark_light::Mode::Dark {
                load_icon(std::path::Path::new(self.icon_path.0))
            } else {
                load_icon(std::path::Path::new(self.icon_path.1))
            };
            println!("ddd: {:?}", self.curr_theme);
            tray_icon.set_icon(Some(icon)).unwrap();
        }
    }
}

impl ApplicationHandler<RunCatTrayEvent> for RunCatTray {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // event_loop.set_control_flow(ControlFlow::WaitUntil(
        //     std::time::Instant::now() + std::time::Duration::from_millis(16),
        // ));

        event_loop.set_control_flow(ControlFlow::Wait);

        MenuEvent::set_event_handler(Some(move |f| {
            if let Some(proxy) = EVENT_LOOP_PROXY.lock().as_ref() {
                proxy.send_event(RunCatTrayEvent::TrayMenuEvent(f)).unwrap();
            }
        }));

        // for item in self.menu_items.iter() {
        //     self.tray_menu.prepend(&item.0).unwrap();
        // }

        self.tary_icon = Some(
            TrayIconBuilder::new()
                .with_menu(Box::new(self.tray_menu.clone()))
                .with_tooltip("runcat")
                .with_title("runcat")
                .build()
                .unwrap(),
        );

        self.on_theme_changed();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: RunCatTrayEvent,
    ) {
        match event {
            RunCatTrayEvent::TrayMenuEvent(ev) => {
                if ev.id == self.exit_menu_item.id() {
                    event_loop.exit();
                } else if ev.id == self.toggle_theme_menu_item.id() {
                    if self.curr_theme == dark_light::Mode::Dark {
                        self.curr_theme = dark_light::Mode::Light;
                    } else {
                        self.curr_theme = dark_light::Mode::Dark;
                    }
                    self.on_theme_changed();
                } else if ev.id == self.auto_fit_theme_menu_item.id() {
                    let text = if self.auto_fit_theme {
                        self.toggle_theme_menu_item.set_enabled(true);
                        format!("auto fit theme: false")
                    } else {
                        self.toggle_theme_menu_item.set_enabled(false);
                        format!("auto fit theme: true")
                    };

                    self.auto_fit_theme = !self.auto_fit_theme;
                    self.auto_fit_theme_menu_item.set_text(text);
                    self.on_theme_changed();
                }
                // else {
                //     if let Some(find) = self.menu_items.iter().find(|x| *x.0.id() == ev.id) {
                //         find.1(ev);
                //     }
                // }
            }
            RunCatTrayEvent::SystemThemeChanged(m) => {
                if self.auto_fit_theme {
                    self.curr_theme = m;
                    self.on_theme_changed();
                }
            }
        }
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        let mode = dark_light::detect();

        if self.curr_theme != mode {
            if let Some(proxy) = EVENT_LOOP_PROXY.lock().as_ref() {
                proxy
                    .send_event(RunCatTrayEvent::SystemThemeChanged(mode))
                    .unwrap();
            }
        }
    }
}

fn main() {
    let path = (
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/cat/dark_cat_0.ico"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/cat/light_cat_0.ico"),
    );

    let event_loop = EventLoop::<RunCatTrayEvent>::with_user_event()
        .build()
        .expect("can't start the event loop");

    *EVENT_LOOP_PROXY.lock() = Some(event_loop.create_proxy());

    let mut app = RunCatTray::new(path);

    event_loop.run_app(&mut app).unwrap();
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
