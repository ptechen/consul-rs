#[allow(dead_code)]
use super::agent;
use super::catalog;
use async_std::sync::Arc;
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::time::Duration;

lazy_static!(
    /// HealthAny is special, and is used as a wild card, not as a specific state.
    #[derive(Debug)]
    pub static ref HEALTH_ANY: Arc<String> = {
        Arc::new(String::from("any"))
    };
    #[derive(Debug)]
    pub static ref HEALTH_PASSING:Arc<String> = {
        Arc::new(String::from("passing"))
    };
    #[derive(Debug)]
    pub static ref HEALTH_WARNING:Arc<String>  = {
        Arc::new(String::from("warning"))
    };
    #[derive(Debug)]
    pub static ref HEALTH_CRITICAL:Arc<String> = {
        Arc::new(String::from("critical"))
    };
    #[derive(Debug)]
    pub static ref HEALTH_MAINT:Arc<String> = {
        Arc::new(String::from("maintenance"))
    };
);

lazy_static!(
    #[derive(Debug)]
    static ref SERVICE_HEALTH:String = {
        String::from("service")
    };
    #[derive(Debug)]
    static ref CONNECT_HEALTH:String = {
        String::from("connect")
    };
    #[derive(Debug)]
    static ref INGRESS_HEALTH:String = {
        String::from("ingress")
    };
);

/// HealthCheck is used to represent a single check
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct HealthCheck {
    pub Node: Option<String>,
    pub CheckID: Option<String>,
    pub Name: Option<String>,
    pub Status: Option<String>,
    pub Notes: Option<String>,
    pub Output: Option<String>,
    pub ServiceID: Option<String>,
    pub ServiceName: Option<String>,
    pub ServiceTags: Option<Vec<String>>,
    pub Type: Option<String>,
    pub Namespace: Option<String>,
    pub Definition: Option<HealthCheckDefinition>,

    pub CreateIndex: Option<usize>,
    pub ModifyIndex: Option<usize>,
}

type ReadableDuration = Duration;

/// HealthCheckDefinition is used to store the details about a health check's execution.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct HealthCheckDefinition {
    pub HTTP: Option<String>,
    pub Header: Option<HashMap<String, Vec<String>>>,
    pub Method: Option<String>,
    pub Body: Option<String>,
    pub TLSServerName: Option<String>,
    pub TLSSkipVerify: Option<bool>,
    pub TCP: Option<String>,
    pub IntervalDuration: Option<Duration>,
    pub TimeoutDuration: Option<Duration>,
    pub DeregisterCriticalServiceAfterDuration: Option<Duration>,

    /// DEPRECATED in Consul 1.4.1. Use the above time.Duration fields instead.
    pub Interval: Option<ReadableDuration>,
    pub Timeout: Option<ReadableDuration>,
    pub DeregisterCriticalServiceAfter: Option<ReadableDuration>,
}

/// HealthChecks is a collection of HealthCheck structs.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct HealthChecks(Vec<HealthCheck>);

/// ServiceEntry is used for the health service endpoint
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ServiceEntry {
    pub Node: Option<catalog::Node>,
    pub Service: Option<agent::AgentService>,
    pub Checks: Option<HealthChecks>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAddress {
    pub index: u64,
    pub address: Vec<String>,
    pub address_link: LinkedList<String>,
}
