#![allow(dead_code, unused_variables, unused_imports)]
#![windows_subsystem = "windows"]

mod err;
mod util;

use err::RunCatTrayError;
use util::{current_exe_dir, load_icon};

use crossbeam_channel::{Receiver, Sender};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    path::{self, Path},
    sync::Arc,
};
use sysinfo::System;
use tray_icon::{
    menu::{IsMenuItem, Menu, MenuEvent, MenuEventReceiver, MenuId, MenuItem},
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent, TrayIconEventReceiver,
};
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::Theme,
};

static EVENT_LOOP_PROXY: Lazy<Arc<Mutex<Option<EventLoopProxy<RunCatTrayEvent>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

const MAX_CAT_INDEX: usize = 4;

pub(crate) struct RunCatTray {
    tray_menu: Menu,
    exit_menu_item: MenuItem,
    auto_fit_theme_menu_item: MenuItem,
    toggle_theme_menu_item: MenuItem,
    // menu_items: Vec<(MenuItem, fn(e: MenuEvent))>,
    tary_icon: Option<TrayIcon>,
    curr_theme: dark_light::Mode,

    curr_icon_resource: Option<(String, RunIconResource)>,

    auto_fit_theme: bool,

    icon_resource: HashMap<String, RunIconResourcePath>,
}

#[derive(Debug)]
pub(crate) enum RunCatTrayEvent {
    TrayMenuEvent(MenuEvent),
    SystemThemeChanged(dark_light::Mode),
    CpuUsageRaiseTrayIconChangeEvent(usize),
}

impl RunIconResource {
    fn load(
        light_paths: &[String],
        dark_paths: &[String],
    ) -> Result<RunIconResource, RunCatTrayError> {
        let base = current_exe_dir()?;

        let mut light_icon = vec![];
        let mut dark_icon = vec![];

        for p in light_paths {
            let icon = load_icon(base.join(p))?;
            light_icon.push(icon);
        }

        for p in dark_paths {
            let icon = load_icon(base.join(p))?;
            dark_icon.push(icon);
        }

        Ok(RunIconResource {
            light: light_icon,
            dark: dark_icon,
        })
    }
}

impl RunCatTray {
    fn new() -> Result<Self, RunCatTrayError> {
        let auto_fit_theme = MenuItem::new("auto fit theme: true", true, None);
        let toggle_theme = MenuItem::new("toggle theme", false, None);
        let exit = MenuItem::new("exit", true, None);

        let (cpu_tx, cpu_rx) = crossbeam_channel::unbounded::<f32>();

        let mut icon_resource = load_resource()?;

        let curr_icon_resource = if !icon_resource.is_empty() {
            if let Some((k, v)) = icon_resource.remove_entry("cat") {
                Some((
                    k,
                    RunIconResource::load(v.light.as_slice(), v.dark.as_slice())?,
                ))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            tray_menu: Menu::with_items(&[&auto_fit_theme, &toggle_theme, &exit]).unwrap(),
            auto_fit_theme_menu_item: auto_fit_theme,
            toggle_theme_menu_item: toggle_theme,
            exit_menu_item: exit,
            // menu_items: vec![],
            tary_icon: None,

            curr_theme: dark_light::detect(),

            curr_icon_resource,

            auto_fit_theme: true,

            icon_resource,
        })
    }

    // fn add_menu_item(&mut self, item: MenuItem, handler: fn(e: MenuEvent)) {
    //     self.menu_items.push((item, handler));
    // }

    fn change_tray_icon(&self, i: usize) {}

    fn on_theme_changed(&mut self) {
        if let Some(tray_icon) = self.tary_icon.as_mut() {
            if let Some(c) = self.curr_icon_resource.clone() {
                let icon = if self.curr_theme == dark_light::Mode::Dark {
                    c.1.dark[0].clone()
                } else {
                    c.1.light[0].clone()
                };

                tray_icon.set_icon(Some(icon)).unwrap();
            }
        }
    }
}

fn send_cpu_usage(sys: &mut System, cpu_tx: &Sender<f32>) {
    loop {
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_usage();
        let cpu_usage = sys.global_cpu_info().cpu_usage();

        cpu_tx.send(cpu_usage).unwrap();
    }
}

fn modify_tray_icon(cpu_rx: &Receiver<f32>) {
    let mut i = 0;
    let mut usage_cache = 1.0;

    loop {
        let cpu_usage = if let Ok(usage) = cpu_rx.try_recv() {
            usage_cache = usage;
            usage
        } else {
            usage_cache
        };

        let cmp_f = [20.0, cpu_usage / 5.0];
        let min = cmp_f.iter().fold(f32::NAN, |m, v| v.min(m));
        let cmp_f = [1.0, min];
        let max = cmp_f.iter().fold(f32::NAN, |m, v| v.max(m));
        std::thread::sleep(std::time::Duration::from_millis((200.0 / max) as u64));

        i = if i >= MAX_CAT_INDEX { 0 } else { i + 1 };

        if let Some(proxy) = EVENT_LOOP_PROXY.lock().as_ref() {
            proxy
                .send_event(RunCatTrayEvent::CpuUsageRaiseTrayIconChangeEvent(i))
                .unwrap();
        }
    }
}

impl RunCatTray {
    fn moditor_cpu_usage(&mut self) {
        let mut sys = System::new();
        let (cpu_tx, cpu_rx) = crossbeam_channel::unbounded();

        tokio::task::spawn(async move {
            send_cpu_usage(&mut sys, &cpu_tx);
        });
        tokio::task::spawn(async move {
            modify_tray_icon(&cpu_rx);
        });
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

        self.moditor_cpu_usage();
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
            RunCatTrayEvent::CpuUsageRaiseTrayIconChangeEvent(i) => {
                if let Some(tray_icon) = self.tary_icon.as_mut() {
                    if let Some(c) = self.curr_icon_resource.clone() {
                        let icon = if self.curr_theme == dark_light::Mode::Dark {
                            c.1.dark[i].clone()
                        } else {
                            c.1.light[i].clone()
                        };

                        tray_icon.set_icon(Some(icon)).unwrap();
                    }
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

#[derive(Clone)]
pub struct RunIconResource {
    pub dark: Vec<Icon>,
    pub light: Vec<Icon>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunIconResourcePath {
    dark: Vec<String>,
    light: Vec<String>,
}

fn load_resource() -> Result<HashMap<String, RunIconResourcePath>, RunCatTrayError> {
    let base = current_exe_dir()?.join("config/resource.toml");

    let k = config::Config::builder()
        .add_source(config::File::with_name(base.to_str().unwrap()))
        .build()
        .map_err(|e| {
            println!("{:?}", e);

            RunCatTrayError::FileError(
                "File \"resource.toml\" is not found/invalid toml file. Please check.",
            )
        })?;

    Ok(k.get::<HashMap<String, RunIconResourcePath>>("resource")
        .map_err(|e| RunCatTrayError::FileError("Invalid resource file. Please check it out."))?)
}

#[tokio::main(worker_threads = 2)]
async fn main() -> Result<(), RunCatTrayError> {
    let event_loop = EventLoop::<RunCatTrayEvent>::with_user_event()
        .build()
        .map_err(|e| RunCatTrayError::RunAppFailed("Can't start the event loop"))?;

    *EVENT_LOOP_PROXY.lock() = Some(event_loop.create_proxy());
    let mut app = RunCatTray::new()?;

    event_loop
        .run_app(&mut app)
        .map_err(|e| RunCatTrayError::RunAppFailed("RunCat app start failed."))?;

    Ok(())
}
