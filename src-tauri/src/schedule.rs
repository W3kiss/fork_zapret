//! Time-based scheduler: connects / disconnects zapret at set times on chosen weekdays.

use crate::controller;
use crate::state::AppState;
use chrono::{Datelike, Local, Timelike};
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Remembers "<id>@<YYYYMMDDHHMM>" already fired, so a rule triggers once per minute.
        let mut fired: HashSet<String> = HashSet::new();
        loop {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let st = app.state::<AppState>();
            if st.updating.load(Ordering::SeqCst) || st.autopick_running.load(Ordering::SeqCst) {
                continue;
            }
            let now = Local::now();
            let hm = format!("{:02}:{:02}", now.hour(), now.minute());
            let stamp = now.format("%Y%m%d%H%M").to_string();
            let dow = now.weekday().num_days_from_sunday() as u8; // 0 = Sunday

            for s in st.cfg().schedules {
                if !s.enabled || s.time != hm {
                    continue;
                }
                if !s.days.is_empty() && !s.days.contains(&dow) {
                    continue;
                }
                let key = format!("{}@{}", s.id, stamp);
                if fired.contains(&key) {
                    continue;
                }
                fired.insert(key);
                let app2 = app.clone();
                let action = s.action.clone();
                tauri::async_runtime::spawn(async move {
                    if action == "disconnect" {
                        let _ = controller::disconnect(&app2, "schedule").await;
                    } else {
                        let _ = controller::connect(&app2, None, "schedule").await;
                    }
                });
            }

            // Keep the fired-set from growing forever.
            if fired.len() > 256 {
                fired.retain(|k| k.ends_with(&stamp));
            }
        }
    });
}
