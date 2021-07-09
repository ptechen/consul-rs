use super::catalog;
use super::config_entry;
use super::health;
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// ServiceKind is the kind of service being registered.
type ServiceKind = String;

lazy_static! {
    /// SERVICE_KIND_TYPICAL is a typical, classic Consul service. This is
    /// represented by the absence of a value. This was chosen for ease of
    /// backwards compatibility: existing services in the catalog would
    /// default to the typical service.
    pub static ref SERVICE_KIND_TYPICAL: ServiceKind = {
        String::new()
    };
    /// SERVICE_KIND_CONNECT_PROXY is a proxy for the Connect feature. This
    /// service proxies another service within Consul and speaks the connect
    /// protocol.
    pub static ref SERVICE_KIND_CONNECT_PROXY: ServiceKind = {
        String::from("connect-proxy")
    };

    /// SERVICE_KIND_MESH_GATEWAY is a Mesh Gateway for the Connect feature. This
    /// service will proxy connections based off the SNI header set by other
    /// connect proxies
    pub static ref SERVICE_KIND_MESH_GATEWAY: ServiceKind = {
        String::from("mesh-gateway")
    };

    /// SERVICE_KIND_TERMINATING_GATEWAY is a Terminating Gateway for the Connect
    /// feature. This service will proxy connections to services outside the mesh.
    pub static ref SERVICE_KIND_TERMINATING_GATEWAY: ServiceKind = {
        String::from("terminating-gateway")
    };
}

/// UpstreamDestType is the type of upstream discovery mechanism.
type UpstreamDestType = String;

lazy_static! {
    /// UpstreamDestTypeService discovers instances via healthy service lookup.
    pub static ref UPSTREAM_DEST_TYPE_SERVICE: UpstreamDestType = {
        String::from("service")
    };

    /// UpstreamDestTypePreparedQuery discovers instances via prepared query execution.
    pub static ref UPSTREAM_DEST_TYPE_PREPARED_QUERY: UpstreamDestType = {
        String::from("prepared_query")
    };
}

/// AgentCheck represents a check known to the api
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AgentCheck {
    pub Node: Option<String>,
    pub CheckID: Option<String>,
    pub Name: Option<String>,
    pub Status: Option<String>,
    pub Notes: Option<String>,
    pub Output: Option<String>,
    pub ServiceID: Option<String>,
    pub ServiceName: Option<String>,
    pub Type: Option<String>,
    pub Definition: Option<health::HealthCheckDefinition>,
    pub Namespace: String,
}

/// Filter
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub filter: String,
}

/// AgentWeights represent optional weights for a service
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AgentWeights {
    pub Passing: Option<usize>,
    pub Warning: Option<usize>,
}

/// AgentService represents a service known to the agent
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AgentService {
    pub Kind: Option<ServiceKind>,
    pub ID: Option<String>,
    pub Service: Option<String>,
    pub Tags: Option<Vec<String>>,
    pub Meta: Option<HashMap<String, String>>,
    pub Port: Option<usize>,
    pub Address: Option<String>,
    pub TaggedAddresses: Option<HashMap<String, catalog::ServiceAddress>>,
    pub Weights: Option<AgentWeights>,
    pub EnableTagOverride: Option<bool>,
    pub CreateIndex: Option<u64>,
    pub ModifyIndex: Option<u64>,
    pub ContentHash: Option<String>,
    pub Proxy: Option<AgentServiceConnectProxyConfig>,
    pub Connect: Option<AgentServiceConnect>,
    /// NOTE: If we ever set the ContentHash outside of singular service lookup then we may need
    /// to include the Namespace in the hash. When we do, then we are in for lots of fun with test.
    /// For now though, ignoring it works well enough.
    pub Namespace: Option<String>,
    /// Datacenter is only ever returned and is ignored if presented.
    pub Datacenter: Option<String>,
}

/// AgentServiceChecksInfo returns information about a Service and its checks
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AgentServiceChecksInfo {
    pub AggregatedStatus: Option<String>,
    pub Service: Option<AgentService>,
    pub Checks: Option<health::HealthChecks>,
}

/// AgentServiceConnect represents the Connect configuration of a service.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AgentServiceConnect {
    pub Native: Option<bool>,
    pub SidecarService: Box<Option<AgentServiceRegistration>>,
}

/// AgentServiceConnectProxyConfig is the proxy configuration in a connect-proxy
/// ServiceDefinition or response.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AgentServiceConnectProxyConfig {
    pub DestinationServiceName: Option<String>,
    pub DestinationServiceID: Option<String>,
    pub LocalServiceAddress: Option<String>,
    pub LocalServicePort: Option<String>,
    pub Mode: Option<config_entry::ProxyMode>,
    pub TransparentProxy: Option<String>,
    pub Config: Option<HashMap<String, Value>>,
    pub Upstreams: Option<Vec<Upstream>>,
    pub MeshGateway: Option<config_entry::MeshGatewayConfig>,
    pub Expose: Option<config_entry::ExposeConfig>,
}

/// AgentServiceRegistration is used to register a new service
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AgentServiceRegistration {
    pub Kind: Option<ServiceKind>,
    pub ID: Option<String>,
    pub Name: Option<String>,
    pub Tags: Option<Vec<String>>,
    pub Port: Option<usize>,
    pub Address: Option<String>,
    pub TaggedAddresses: Option<HashMap<String, catalog::ServiceAddress>>,
    pub EnableTagOverride: Option<bool>,
    pub Meta: Option<HashMap<String, String>>,
    pub Weights: Option<AgentWeights>,
    pub Check: Option<AgentServiceCheck>,
    pub Checks: Option<AgentServiceChecks>,
    pub Proxy: Option<AgentServiceConnectProxyConfig>,
    pub Connect: Option<AgentServiceConnect>,
    // pub Namespace: Option<String>,
}

/// ServiceRegisterOpts is used to pass extra options to the service register.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ServiceRegisterOpts {
    ///Missing healthchecks will be deleted from the agent.
    ///Using this parameter allows to idempotently register a service and its checks without
    ///having to manually deregister checks.
    #[serde(rename = "replace-existing-checks")]
    pub ReplaceExistingChecks: bool,
}

/// Upstream is the response structure for a proxy upstream configuration.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Upstream {
    pub DestinationType: Option<UpstreamDestType>,
    pub DestinationNamespace: Option<String>,
    pub DestinationName: Option<String>,
    pub Datacenter: Option<String>,
    pub LocalBindAddress: Option<String>,
    pub LocalBindPort: Option<usize>,
    pub Config: HashMap<String, Value>,
    pub MeshGateway: Option<config_entry::MeshGatewayConfig>,
    pub CentrallyConfigured: Option<bool>,
}
//
type AgentServiceChecks = Vec<AgentServiceCheck>;

/// AgentServiceCheck is used to define a node or service level check
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AgentServiceCheck {
    pub CheckID: Option<String>,
    pub Name: Option<String>,
    pub Args: Option<Vec<String>>,
    pub DockerContainerID: Option<String>,
    /// Only supported for Docker.
    pub Shell: Option<String>,
    pub Interval: Option<String>,
    pub Timeout: Option<String>,
    pub TTL: Option<String>,
    pub HTTP: Option<String>,
    pub Header: Option<HashMap<String, String>>,
    pub Method: Option<String>,
    pub Body: Option<String>,
    pub TCP: Option<String>,
    pub Status: Option<String>,
    pub Notes: Option<String>,
    pub TLSServerName: Option<String>,
    pub TLSSkipVerify: Option<bool>,
    pub GRPC: Option<String>,
    pub GRPCUseTLS: Option<bool>,
    pub AliasNode: Option<String>,
    pub AliasService: Option<String>,
    pub SuccessBeforePassing: Option<i64>,
    pub FailuresBeforeCritical: Option<i64>,

    /// In Consul 0.7 and later, checks that are associated with a service
    /// may also contain this optional DeregisterCriticalServiceAfter field,
    /// which is a timeout in the same Go time format as Interval and TTL. If
    /// a check is in the critical state for more than this configured value,
    /// then its associated service (and all of its associated checks) will
    /// automatically be deregistered.
    pub DeregisterCriticalServiceAfter: Option<String>,
}

/// Metrics info is used to store different types of metric values from the agent.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct MetricsInfo {
    pub Timestamp: Option<String>,
    pub Gauges: Option<Vec<GaugeValue>>,
    pub Points: Option<Vec<PointValue>>,
    pub Counters: Option<Vec<SampledValue>>,
    pub Samples: Option<Vec<SampledValue>>,
}

/// GaugeValue stores one value that is updated as time goes on, such as
/// the amount of memory allocated.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct GaugeValue {
    pub Name: Option<String>,
    pub Value: Option<f32>,
    pub Labels: Option<HashMap<String, String>>,
}

/// PointValue holds a series of points for a metric.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PointValue {
    pub Name: Option<String>,
    pub Points: Option<Vec<f32>>,
}

/// SampledValue stores info about a metric that is incremented over time,
/// such as the number of requests to an HTTP endpoint.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct SampledValue {
    pub Name: Option<String>,
    pub Count: Option<i64>,
    pub Sum: Option<f64>,
    pub Min: Option<f64>,
    pub Max: Option<f64>,
    pub Mean: Option<f64>,
    pub Stddev: Option<f64>,
    pub Labels: Option<HashMap<String, String>>,
}
