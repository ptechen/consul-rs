#[allow(dead_code)]
use super::agent;
use super::catalog;
use async_std::sync::{Arc};
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

// lazy_static!(
//     /// NODE_MAINT is the special key set by a node in maintenance mode.
//     #[derive(Debug)]
//     pub static ref NODE_MAINT:Arc<String> = {
//         Arc::new(String::from("_node_maintenance"))
//     };
//     #[derive(Debug)]
//     /// SERVICE_MAINT_PREFIX is the prefix for a service in maintenance mode.
//     pub static ref SERVICE_MAINT_PREFIX:Arc<String> = {
//         Arc::new(String::from("_service_maintenance:"))
//     };
// );

// lazy_static! {
//     pub static ref HEALTH: Arc<RwLock<Health>> = {
//         let client = api::CLIENT.clone();
//         let lock = block_on(client.read());
//         let health = block_on(lock.health());
//         Arc::new(RwLock::new(health))
//     };
//     pub static ref SERVICES_ADDRESS: Arc<RwLock<HashMap<String, ServiceAddress>>> = {
//         let services_address: HashMap<String, ServiceAddress> = HashMap::new();
//         Arc::new(RwLock::new(services_address))
//     };
// }

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

/// AggregatedStatus returns the "best" status for the list of health checks.
/// Because a given entry may have many service and node-level health checks
/// attached, this function determines the best representative of the status as
/// as single string using the following heuristic:
///
///  maintenance > critical > warning > passing
///

// impl HealthChecks {
//     pub async fn aggregates_status(&self) -> String {
//         let mut passing: bool = false;
//         let mut warning: bool = false;
//         let mut critical: bool = false;
//         let mut maintenance: bool = false;
//         for check in self.0.iter() {
//             if check.CheckID.is_some() {
//                 let id = check.CheckID.as_ref().unwrap();
//                 let pat = format!("^{:?}", &*SERVICE_MAINT_PREFIX.clone());
//                 let re = Regex::new(&pat).unwrap();
//                 let node_main = &*NODE_MAINT.clone();
//                 if id == node_main || re.is_match(&id) {
//                     maintenance = true;
//                     continue;
//                 }
//             }
//             if check.Status.is_some() {
//                 let status = check.Status.as_ref().unwrap();
//                 let p = &*HEALTH_PASSING.clone();
//                 let w = &*HEALTH_WARNING.clone();
//                 if status == p {
//                     passing = true
//                 } else if status == w {
//                     warning = true
//                 } else if status == &*HEALTH_CRITICAL.clone() {
//                     critical = true
//                 } else {
//                     return String::new();
//                 }
//             } else {
//                 return String::new();
//             }
//         }
//
//         return if maintenance {
//             let s = &*HEALTH_MAINT.clone();
//             s.into()
//         } else if critical {
//             let s = &*HEALTH_CRITICAL.clone();
//             s.into()
//         } else if warning {
//             let s = &*HEALTH_WARNING.clone();
//             s.into()
//         } else if passing {
//             let s = &*HEALTH_PASSING.clone();
//             s.into()
//         } else {
//             let s = &*HEALTH_PASSING.clone();
//             s.into()
//         };
//     }
// }

/// ServiceEntry is used for the health service endpoint
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ServiceEntry {
    pub Node: Option<catalog::Node>,
    pub Service: Option<agent::AgentService>,
    pub Checks: Option<HealthChecks>,
}

// Health can be used to query the Health endpoints
// #[derive(Default, Debug)]
// pub struct Health {
//     pub c: Option<api::Client>,
// }

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Tag {
    pub tag: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Index {
    pub index: u64,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct WaitPassing {
    pub wait: String,
    pub passing: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAddress {
    pub index: u64,
    pub address: Vec<String>,
    pub address_link: LinkedList<String>,
}

// impl ServiceAddress {
//     pub async fn rand_policy(&self) -> String {
//         let range = self.address.len();
//         if range == 0 {
//             return String::new();
//         };
//         let mut r = rand::thread_rng();
//         let idx: usize = r.gen_range(0..range);
//         let address = self.address.get(idx).unwrap();
//         String::from(address)
//     }
// }
// 
// impl Health {
//     pub async fn reload_client() {
//         let client = api::CLIENT.clone();
//         let client = client.read().await;
//         let s = client.health().await;
//         let health = HEALTH.clone();
//         let mut health = health.write().await;
//         *health = s;
//     }
// }
//
// /// QueryMeta is used to return meta data about a query
// #[derive(Default, Debug, Clone, Serialize, Deserialize)]
// #[allow(non_snake_case)]
// pub struct QueryMeta {
//     /// LastIndex. This can be used as a WaitIndex to perform
//     /// a blocking query
//     pub LastIndex: Option<u64>,
//
//     /// LastContentHash. This can be used as a WaitHash to perform a blocking query
//     /// for endpoints that support hash-based blocking. Endpoints that do not
//     /// support it will return an empty hash.
//     pub LastContentHash: Option<String>,
//
//     /// Time of last contact from the leader for the
//     /// server servicing the request
//     pub LastContact: Option<Duration>,
//
//     /// Is there a known leader
//     pub KnownLeader: Option<bool>,
//
//     /// How long did the request take
//     pub RequestTime: Option<Duration>,
//
//     /// Is address translation enabled for HTTP responses on this agent
//     pub AddressTranslationEnabled: Option<bool>,
//
//     /// CacheHit is true if the result was served from agent-local cache.
//     pub CacheHit: Option<bool>,
//
//     /// CacheAge is set if request was ?cached and indicates how stale the cached
//     /// response is.
//     pub CacheAge: Option<Duration>,
//
//     /// DefaultACLPolicy is used to control the ACL interaction when there is no
//     /// defined policy. This can be "allow" which means ACLs are used to
//     /// deny-list, or "deny" which means ACLs are allow-lists.
//     pub DefaultACLPolicy: Option<String>,
// }
//
//
//
