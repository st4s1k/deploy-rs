// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::app::ScenarioAppState;
use commands::{
    clear_log, execute_scenario, get_config_path, get_log, get_required_variables, load_config,
    save_state, update_required_variables,
};
use std::sync::Mutex;
use tauri::Manager;

mod app;
mod commands;
mod lifecycle;
mod shared;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let mut state = ScenarioAppState::new(app.app_handle());
            state.load_state();
            app.manage(Mutex::new(state));
            Ok(())
        })
        .on_window_event(on_window_event)
        .invoke_handler(tauri::generate_handler![
            save_state,
            get_config_path,
            get_log,
            clear_log,
            load_config,
            get_required_variables,
            update_required_variables,
            execute_scenario
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn on_window_event(event: tauri::GlobalWindowEvent) {
    if let tauri::WindowEvent::Destroyed { .. } = event.event() {
        let app_handle = event.window().app_handle();
        let state = app_handle.state::<Mutex<ScenarioAppState>>();
        let mut state = state.lock().unwrap();
        state.save_state();
    }
}
