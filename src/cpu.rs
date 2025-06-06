use crossbeam_channel::{Receiver, Sender};

use sysinfo::System;

use crate::{
    event::{RunCatTrayEvent, EVENT_LOOP_PROXY},
    icon_resource::MAX_RUN_ICON_INDEX,
};

pub(crate) fn monitor_cpu_usage() {
    let (cpu_tx, cpu_rx) = crossbeam_channel::unbounded();

    tokio::task::spawn(async move {
        send_cpu_usage(&cpu_tx).await;
    });
    tokio::task::spawn(async move {
        send_icon_index(&cpu_rx).await;
    });
}

pub(crate) async fn send_cpu_usage(cpu_tx: &Sender<f32>) {
    let mut sys = System::new();

    loop {
        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
        sys.refresh_cpu_usage();
        let cpu_usage = sys.global_cpu_usage();

        cpu_tx.send(cpu_usage).unwrap();
    }
}

pub(crate) async fn send_icon_index(cpu_rx: &Receiver<f32>) {
    let mut i = 0;
    let mut cpu_usage = 1.0;

    loop {
        if let Ok(usage) = cpu_rx.try_recv() {
            cpu_usage = usage;
        }

        let min = 20.0_f32.min(cpu_usage / 5.0);
        let max = 1.0_f32.max(min);

        i = if i >= MAX_RUN_ICON_INDEX { 0 } else { i + 1 };

        tokio::time::sleep(std::time::Duration::from_millis((200.0 / max) as u64)).await;

        if let Some(proxy) = EVENT_LOOP_PROXY.lock().as_ref() {
            proxy
                .send_event(RunCatTrayEvent::ChangeIconIndexEvent(i))
                .unwrap();
        }
    }
}
