#[allow(dead_code)]
use serde_derive::{Serialize, Deserialize};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::time::Duration;
use regex::Regex;
use async_std::sync::Arc;
use super::catalog;
use super::api;
use super::agent;

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

lazy_static!(
    /// NODE_MAINT is the special key set by a node in maintenance mode.
    #[derive(Debug)]
    pub static ref NODE_MAINT:Arc<String> = {
        Arc::new(String::from("_node_maintenance"))
    };
    #[derive(Debug)]
    /// SERVICE_MAINT_PREFIX is the prefix for a service in maintenance mode.
    pub static ref SERVICE_MAINT_PREFIX:Arc<String> = {
        Arc::new(String::from("_service_maintenance:"))
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
    pub Header: HashMap<String, Vec<String>>,
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

/// AggregatedStatus returns the "best" status for the list of health checks.
/// Because a given entry may have many service and node-level health checks
/// attached, this function determines the best representative of the status as
/// as single string using the following heuristic:
///
///  maintenance > critical > warning > passing
///

impl HealthChecks {
    pub async fn aggregates_status(&self) -> String {
        let mut passing: bool = false;
        let mut warning: bool = false;
        let mut critical: bool = false;
        let mut maintenance: bool = false;
        for check in self.0.iter() {
            if check.CheckID.is_some() {
                let id = check.CheckID.as_ref().unwrap();
                let pat = format!("^{:?}", &*SERVICE_MAINT_PREFIX.clone());
                let re = Regex::new(&pat).unwrap();
                let node_main = &*NODE_MAINT.clone();
                if id == node_main || re.is_match(&id) {
                    maintenance = true;
                    continue;
                }
            }
            if check.Status.is_some() {
                let status = check.Status.as_ref().unwrap();
                let p = &*HEALTH_PASSING.clone();
                let w = &*HEALTH_WARNING.clone();
                if status == p {
                    passing = true
                } else if status == w {
                    warning = true
                } else if status == &*HEALTH_CRITICAL.clone() {
                    critical = true
                } else {
                    return String::new();
                }
            } else {
                return String::new();
            }
        }

        return if maintenance {
            let s = &*HEALTH_MAINT.clone();
            s.into()
        } else if critical {
            let s = &*HEALTH_CRITICAL.clone();
            s.into()
        } else if warning {
            let s= &*HEALTH_WARNING.clone();
            s.into()
        } else if passing {
            let s = &*HEALTH_PASSING.clone();
            s.into()
        } else {
            let s = &*HEALTH_PASSING.clone();
            s.into()
        };
    }
}

/// ServiceEntry is used for the health service endpoint
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ServiceEntry  {
    pub Node: Option<catalog::Node>,
    pub Service: Option<agent::AgentService>,
    pub Checks:  Option<HealthChecks>
}

// Health can be used to query the Health endpoints
#[derive(Default, Debug)]
struct Health {
    c: api::Client,
}

#[test]
fn test() {
    let s = Duration::new(1234,1234);
    println!("{:?}", s)
}