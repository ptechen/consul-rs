use serde_derive::{Deserialize, Serialize};

pub const ROUND_ROBIN: &'static str = "round_robin";
pub const RANDOM: &'static str = "random";

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct WatchService {
    pub service_name: String,
    pub tag: Option<String>,
    pub passing_only: Option<bool>,
}
