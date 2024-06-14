use std::collections::HashMap;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};
use winit::application::ApplicationHandler;

use crate::{
    cpu::{modify_tray_icon, send_cpu_usage},
    err::RunCatTrayError,
    event::{RunCatTrayEvent, EVENT_LOOP_PROXY},
    icon_resource::{RunIconResource, RunIconResourcePath},
    util::current_exe_dir,
};

pub(crate) const RESOURCE_PATH: &str = "config/resource.toml";

pub(crate) const DEFAULT_ICON_NAME: &str = "cat";

pub(crate) struct RunCatTray {
    tray_menu: Menu,
    exit_menu_item: MenuItem,
    auto_theme_menu_item: MenuItem,
    toggle_theme_menu_item: MenuItem,
    // menu_items: Vec<(MenuItem, fn(e: MenuEvent))>,
    tary_icon: Option<TrayIcon>,
    curr_theme: dark_light::Mode,
    auto_theme: bool,
    curr_icon_resource: Option<(String, RunIconResource)>,
    icon_resource_paths: HashMap<String, RunIconResourcePath>,
}

impl RunCatTray {
    pub fn new() -> Result<Self, RunCatTrayError> {
        let auto_fit_theme = MenuItem::new("Auto theme: true", true, None);
        let toggle_theme = MenuItem::new("Toggle theme", false, None);
        let exit = MenuItem::new("Exit", true, None);

        let mut tray = Self {
            tray_menu: Menu::with_items(&[&auto_fit_theme, &toggle_theme, &exit]).unwrap(),
            auto_theme_menu_item: auto_fit_theme,
            toggle_theme_menu_item: toggle_theme,
            exit_menu_item: exit,
            // menu_items: vec![],
            tary_icon: None,
            curr_theme: dark_light::detect(),
            curr_icon_resource: None,
            auto_theme: true,
            icon_resource_paths: RunIconResourcePath::load(current_exe_dir()?.join(RESOURCE_PATH))?,
        };

        tray.load_icon_by_name(DEFAULT_ICON_NAME)?;

        Ok(tray)
    }

    fn load_icon_by_name(&mut self, name: &str) -> Result<(), RunCatTrayError> {
        self.curr_icon_resource = if let Some((k, v)) = self.icon_resource_paths.remove_entry(name)
        {
            Some((k, RunIconResource::load(&v.light, &v.dark)?))
        } else {
            None
        };

        Ok(())
    }

    // fn add_menu_item(&mut self, item: MenuItem, handler: fn(e: MenuEvent)) {
    //     self.menu_items.push((item, handler));
    // }

    // fn change_tray_icon(&self, i: usize) {}

    fn on_theme_changed(&mut self) {
        if let Some(tray_icon) = self.tary_icon.as_mut() {
            if let Some((_, resource)) = self.curr_icon_resource.as_ref() {
                let icon = if self.curr_theme == dark_light::Mode::Dark {
                    resource.dark[0].clone()
                } else {
                    resource.light[0].clone()
                };

                tray_icon.set_icon(Some(icon)).unwrap();
            }
        }
    }

    fn send_menu_event(&self) {
        MenuEvent::set_event_handler(Some(move |f| {
            if let Some(proxy) = EVENT_LOOP_PROXY.lock().as_ref() {
                proxy.send_event(RunCatTrayEvent::TrayMenuEvent(f)).unwrap();
            }
        }));
    }
}

impl RunCatTray {
    fn moditor_cpu_usage(&mut self) {
        let (cpu_tx, cpu_rx) = crossbeam_channel::unbounded();

        tokio::task::spawn(async move {
            send_cpu_usage(&cpu_tx);
        });
        tokio::task::spawn(async move {
            modify_tray_icon(&cpu_rx);
        });
    }
}

impl ApplicationHandler<RunCatTrayEvent> for RunCatTray {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
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

        self.send_menu_event();
        self.on_theme_changed();
        self.moditor_cpu_usage();
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
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
                    self.curr_theme = if self.curr_theme == dark_light::Mode::Dark {
                        dark_light::Mode::Light
                    } else {
                        dark_light::Mode::Dark
                    };

                    self.on_theme_changed();
                } else if ev.id == self.auto_theme_menu_item.id() {
                    self.auto_theme = !self.auto_theme;

                    self.toggle_theme_menu_item.set_enabled(!self.auto_theme);

                    self.auto_theme_menu_item
                        .set_text(format!("Auto theme: {}", self.auto_theme));

                    self.on_theme_changed();
                }
                // else {
                //     if let Some(find) = self.menu_items.iter().find(|x| *x.0.id() == ev.id) {
                //         find.1(ev);
                //     }
                // }
            }
            RunCatTrayEvent::SystemThemeChanged(m) => {
                if self.auto_theme {
                    self.curr_theme = m;
                    self.on_theme_changed();
                }
            }
            RunCatTrayEvent::CpuUsageRaiseTrayIconChangeEvent(i) => {
                if let Some(tray_icon) = self.tary_icon.as_mut() {
                    if let Some((_, resource)) = self.curr_icon_resource.as_ref() {
                        let icon = if self.curr_theme == dark_light::Mode::Dark {
                            resource.dark[i].clone()
                        } else {
                            resource.light[i].clone()
                        };

                        tray_icon.set_icon(Some(icon)).unwrap();
                    }
                }
            }
        }
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _cause: winit::event::StartCause,
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
