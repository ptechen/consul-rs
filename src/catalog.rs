use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Node {
    pub ID: Option<String>,
    pub Node: Option<String>,
    pub Address: Option<String>,
    pub Datacenter: Option<String>,
    pub TaggedAddresses: Option<HashMap<String, String>>,
    pub Meta: Option<HashMap<String, String>>,
    pub CreateIndex: u64,
    pub ModifyIndex: u64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ServiceAddress {
    pub Address: Option<String>,
    pub Port: Option<usize>,
}