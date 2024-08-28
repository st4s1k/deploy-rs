use crate::{lifecycle::LifecycleHandler, shared::SEPARATOR};
use scenario_rs::{config::ScenarioConfig, scenario::Scenario};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, path::PathBuf, str::FromStr};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScenarioAppStateConfig {
    config_path: String,
    output_log: String,
}

impl From<&ScenarioAppState> for ScenarioAppStateConfig {
    fn from(state: &ScenarioAppState) -> Self {
        Self {
            config_path: state.config_path.clone(),
            output_log: state.output_log.clone(),
        }
    }
}

pub struct ScenarioAppState {
    pub(crate) config_path: String,
    pub(crate) output_log: String,
    pub(crate) app_handle: AppHandle,
    pub(crate) config: Option<ScenarioConfig>,
    pub(crate) is_executing: bool,
}

impl ScenarioAppState {
    const STATE_FILE_PATH: &'static str = "scenario-app-state.json";

    pub fn new(app: AppHandle) -> Self {
        Self {
            config_path: String::new(),
            output_log: String::new(),
            app_handle: app,
            config: None,
            is_executing: false,
        }
    }

    pub fn load_state(&mut self) {
        if let Ok(json) = std::fs::read_to_string(Self::STATE_FILE_PATH) {
            if let Ok(loaded_state) = serde_json::from_str::<ScenarioAppStateConfig>(&json) {
                let config_path = loaded_state.config_path;
                self.config_path = config_path.clone();
                self.output_log = loaded_state.output_log;
                self.load_config(config_path.as_str());
            }
        }
    }

    pub fn save_state(&mut self) {
        let state = ScenarioAppStateConfig::from(self.deref());
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            if let Err(error) = std::fs::write(Self::STATE_FILE_PATH, json) {
                self.log_message(format!(
                    "{SEPARATOR}\nFailed to save state: {error}\n{SEPARATOR}\n"
                ));
                let _ = self.app_handle.emit_all("log-update", ());
            }
        }
    }

    pub fn load_config(&mut self, config_path: &str) {
        let Ok(config_path) = PathBuf::from_str(config_path) else {
            self.log_message(format!("{SEPARATOR}\nInvalid config path\n{SEPARATOR}\n"));
            return;
        };

        match ScenarioConfig::try_from(config_path) {
            Ok(config) => {
                self.log_message(format!(
                    "{SEPARATOR}\nScenario config loaded\n{SEPARATOR}\n"
                ));
                let _ = self.app_handle.emit_all("log-update", ());
                self.config = Some(config);
            }
            Err(e) => {
                self.log_message(format!(
                    "{SEPARATOR}\nFailed to load scenario config: {e}\n{SEPARATOR}\n"
                ));
                let _ = self.app_handle.emit_all("log-update", ());
            }
        }
    }

    pub fn execute_scenario(&mut self) {
        let Some(config) = &self.config else {
            self.log_message(format!(
                "{SEPARATOR}\nNo scenario config file loaded\n{SEPARATOR}\n"
            ));
            let _ = self.app_handle.emit_all("log-update", ());
            return;
        };

        let lifecycle_handler = LifecycleHandler::try_initialize(self.app_handle.clone());

        let scenario = match Scenario::new(config.clone()) {
            Ok(scenario) => {
                self.log_message(format!("{SEPARATOR}\nScenario loaded\n{SEPARATOR}\n"));
                scenario
            }
            Err(e) => {
                self.log_message(format!(
                    "{SEPARATOR}\nFailed to load scenario: {e}\n{SEPARATOR}\n"
                ));
                return;
            }
        };

        self.is_executing = true;

        match scenario.execute_with_lifecycle(lifecycle_handler) {
            Ok(_) => self.log_message(format!(
                "{SEPARATOR}\nScenario completed successfully!\n{SEPARATOR}\n"
            )),
            Err(e) => self.log_message(format!("{SEPARATOR}\nScenario failed: {e}\n{SEPARATOR}\n")),
        }

        self.is_executing = false;
    }

    fn log_message(&mut self, message: String) {
        self.output_log.push_str(&message);
        let _ = self.app_handle.emit_all("log-update", ());
    }

    pub fn clear_log(&mut self) {
        self.output_log.clear();
        let _ = self.app_handle.emit_all("log-update", ());
    }
}
