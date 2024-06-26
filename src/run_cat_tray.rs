use std::collections::HashMap;
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuId, MenuItem, Submenu},
    TrayIcon, TrayIconBuilder,
};
use winit::event::WindowEvent;
use winit::window::{WindowAttributes, WindowLevel};
use winit::{application::ApplicationHandler, event_loop::EventLoopProxy};

use crate::{
    cpu::monitor_cpu_usage,
    err::RunCatTrayError,
    event::{send_menu_event, RunCatTrayEvent, EVENT_LOOP_PROXY},
    icon_resource::{IconResource, IconResourcePath},
    util::current_exe_dir,
};

pub(crate) const RESOURCE_PATH: &str = "config/resource.toml";

pub(crate) const DEFAULT_ICON_NAME: &str = "cat";

pub(crate) struct RunCatTray {
    tray_menu: Menu,
    editor_menu_item: MenuItem,
    exit_menu_item: MenuItem,
    characters_menu_item: Submenu,
    theme_menu_item: Submenu,
    tray_icon: Option<TrayIcon>,
    curr_theme: dark_light::Mode,
    auto_theme: bool,
    curr_icon_resource: Option<(String, IconResource)>,
    icon_resource_paths: HashMap<String, IconResourcePath>,

    editor: Option<winit::window::Window>,
}

impl RunCatTray {
    pub fn new() -> Result<Self, RunCatTrayError> {
        let open_editor_menu_item = MenuItem::new("Open editor", true, None);

        let auto = CheckMenuItem::with_id("AutoTheme", "Auto", false, true, None);
        let dark = CheckMenuItem::with_id("DarkTheme", "Dark", true, false, None);
        let light = CheckMenuItem::with_id("LightTheme", "Light", true, false, None);
        let theme =
            Submenu::with_id_and_items("Theme", "Theme", true, &[&auto, &dark, &light]).unwrap();

        let characters = Submenu::new("Characters", true);
        let exit = MenuItem::new("Exit", true, None);

        let paths = IconResourcePath::load(current_exe_dir()?.join(RESOURCE_PATH))?;

        paths.keys().for_each(|k| {
            let checked = k == DEFAULT_ICON_NAME;
            let item = CheckMenuItem::with_id(k, k, !checked, checked, None);
            characters.append(&item).unwrap();
        });

        let mut tray = Self {
            tray_menu: Menu::with_items(&[&open_editor_menu_item, &characters, &theme, &exit])
                .unwrap(),

            editor_menu_item: open_editor_menu_item,

            auto_theme: true,
            curr_theme: dark_light::detect(),
            theme_menu_item: theme,

            exit_menu_item: exit,
            characters_menu_item: characters,

            tray_icon: None,
            curr_icon_resource: None,
            icon_resource_paths: paths,

            editor: None,
        };

        tray.load_icon_by_name(DEFAULT_ICON_NAME)?;

        Ok(tray)
    }

    fn load_icon_by_name(&mut self, name: &str) -> Result<(), RunCatTrayError> {
        self.curr_icon_resource = if let Some((k, v)) = self.icon_resource_paths.get_key_value(name)
        {
            Some((k.to_string(), IconResource::load(&v.light, &v.dark)?))
        } else {
            None
        };

        Ok(())
    }

    fn on_theme_changed(&mut self) {
        if let Some(tray_icon) = self.tray_icon.as_ref() {
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

    pub(crate) fn with_event_loop_proxy(f: impl FnOnce(&EventLoopProxy<RunCatTrayEvent>)) {
        if let Some(proxy) = EVENT_LOOP_PROXY.lock().as_ref() {
            f(proxy);
        }
    }
}

impl ApplicationHandler<RunCatTrayEvent> for RunCatTray {
    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _cause: winit::event::StartCause,
    ) {
        let mode = dark_light::detect();

        if self.curr_theme != mode {
            RunCatTray::with_event_loop_proxy(|proxy| {
                proxy
                    .send_event(RunCatTrayEvent::SystemThemeChanged(mode))
                    .unwrap();
            });
        }
    }

    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.tray_icon = Some(
            TrayIconBuilder::new()
                .with_menu(Box::new(self.tray_menu.clone()))
                .with_tooltip("RunCat")
                .with_title("RunCat")
                .build()
                .unwrap(),
        );

        self.on_theme_changed();

        send_menu_event();
        monitor_cpu_usage();
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
                } else if ev.id == self.editor_menu_item.id() {
                    self.editor = Some(
                        event_loop
                            .create_window(
                                WindowAttributes::default()
                                    .with_window_level(WindowLevel::AlwaysOnTop),
                            )
                            .unwrap(),
                    );
                } else {
                    for item in self.theme_menu_item.items() {
                        if let Some(el) = item.as_check_menuitem() {
                            if ev.id() == el.id() {
                                el.set_checked(true);
                                el.set_enabled(false);
                            } else {
                                el.set_checked(false);
                                el.set_enabled(true);
                            }
                        }
                    }

                    if ev.id() == &MenuId::new("AutoTheme") {
                        self.auto_theme = true;
                        self.curr_theme = dark_light::detect();
                    } else {
                        self.auto_theme = false;
                        self.curr_theme = if ev.id() == &MenuId::new("DarkTheme") {
                            dark_light::Mode::Dark
                        } else {
                            dark_light::Mode::Light
                        };
                    }

                    self.on_theme_changed();

                    for item in self.characters_menu_item.items() {
                        if let Some(el) = item.as_check_menuitem() {
                            if ev.id() != el.id() {
                                el.set_checked(false);
                                el.set_enabled(true);
                            } else {
                                el.set_checked(true);
                                el.set_enabled(false);

                                if self.load_icon_by_name(ev.id().0.as_ref()).is_err() {
                                    // error
                                }
                            }
                        }
                    }
                }
            }
            RunCatTrayEvent::SystemThemeChanged(mode) => {
                if self.auto_theme {
                    self.curr_theme = mode;
                    self.on_theme_changed();
                }
            }
            RunCatTrayEvent::ChangeIconIndexEvent(i) => {
                if let Some(tray_icon) = self.tray_icon.as_mut() {
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

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: WindowEvent,
    ) {
        if _event == WindowEvent::CloseRequested {
            // set editor to None, then Window will be dropped/closed.
            self.editor = None;
        }
    }
}
