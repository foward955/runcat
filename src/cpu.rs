use crossbeam_channel::{Receiver, Sender};

use sysinfo::System;

use crate::{
    event::{RunCatTrayEvent, EVENT_LOOP_PROXY},
    icon_resource::MAX_RUN_ICON_INDEX,
};

pub(crate) fn send_cpu_usage(cpu_tx: &Sender<f32>) {
    let mut sys = System::new();

    loop {
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_usage();
        let cpu_usage = sys.global_cpu_info().cpu_usage();

        cpu_tx.send(cpu_usage).unwrap();
    }
}

pub(crate) fn modify_tray_icon(cpu_rx: &Receiver<f32>) {
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

        i = if i >= MAX_RUN_ICON_INDEX { 0 } else { i + 1 };

        if let Some(proxy) = EVENT_LOOP_PROXY.lock().as_ref() {
            proxy
                .send_event(RunCatTrayEvent::CpuUsageRaiseTrayIconChangeEvent(i))
                .unwrap();
        }
    }
}
