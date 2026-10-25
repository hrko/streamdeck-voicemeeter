use crate::worker::{Request, WorkerClient};
use serde_json::Value;
use streamdeck_plugin::{PluginLifecycle, Result, async_trait};

pub fn refresh_rate(settings: &Value) -> u32 {
    match settings.get("refreshRate").and_then(Value::as_u64) {
        Some(fps @ (15 | 20 | 25 | 30)) => fps as u32,
        _ => 15,
    }
}
pub struct GlobalSettings(pub WorkerClient);
#[async_trait]
impl PluginLifecycle for GlobalSettings {
    async fn on_did_receive_global_settings(&mut self, settings: &Value) -> Result<()> {
        self.0
            .send(Request::RefreshRate(refresh_rate(settings)))
            .map_err(|e| streamdeck_plugin::Error::State(e.to_string()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn supports_only_allowed_rates_and_defaults_old_settings() {
        for fps in [15, 20, 25, 30] {
            assert_eq!(refresh_rate(&json!({"refreshRate":fps})), fps);
        }
        for value in [
            json!({}),
            json!({"refreshRate":60}),
            json!({"refreshRate":"30"}),
            json!({"refreshRate":20.5}),
        ] {
            assert_eq!(refresh_rate(&value), 15);
        }
    }
}
