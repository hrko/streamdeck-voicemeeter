use crate::{domain::Edition, settings::Target};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSnapshot {
    pub label: String,
    pub gain: f32,
    pub levels: [f32; 2],
    pub mute: bool,
    pub flags: BTreeMap<String, bool>,
}

/// No SDK objects cross this boundary. All calls are made by one worker thread.
pub trait Backend {
    fn connect(&mut self) -> anyhow::Result<Option<Edition>>;
    fn poll(&mut self) -> anyhow::Result<()>;
    fn channel(&mut self, target: Target) -> anyhow::Result<ChannelSnapshot>;
    fn set_gain(&mut self, target: Target, value: f32) -> anyhow::Result<()>;
    fn set_mute(&mut self, target: Target, value: bool) -> anyhow::Result<()>;
    fn macro_state(&mut self, id: u8) -> anyhow::Result<bool>;
    fn set_macro(&mut self, id: u8, value: bool) -> anyhow::Result<()>;
}
