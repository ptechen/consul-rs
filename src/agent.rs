use serde_derive::{Serialize, Deserialize};
use lazy_static::lazy_static;
use std::collections::HashMap;
use serde_json::{Value, Map};
use async_std::sync::{Arc, RwLock};
use surf::{Error, StatusCode};
use surf::http::Method;
use async_std::task::block_on;

use super::health;
use super::catalog;
use super::config_entry;
use super::api;

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

/// MEMBER_TAG_KEY_ACL_MODE is the key used to indicate what ACL mode the agent is
/// operating in. The values of this key will be one of the MemberACLMode constants
/// with the key not being present indicating ACLModeUnknown.
pub const MEMBER_TAG_KEY_ACL_MODE: &str = "acls";

/// MEMBER_TAG_KEY_ROLE is the key used to indicate that the member is a server or not.
pub const MEMBER_TAG_KEY_ROLE: &str = "role";

/// MEMBER_TAG_VALUE_ROLE_SERVER is the value of the MemberTagKeyRole used to indicate
/// that the member represents a Consul server.
pub const MEMBER_TAG_VALUE_ROLE_SERVER: &str = "consul";

/// MEMBER_TAG_KEY_SEGMENT is the key name of the tag used to indicate which network
/// segment this member is in.
/// Network Segments are a Consul Enterprise feature.
pub const MEMBER_TAG_KEY_SEGMENT: &str = "segment";

/// MemberTagKeyBootstrap is the key name of the tag used to indicate whether this
/// agent was started with the "bootstrap" configuration enabled
pub const MEMBER_TAG_KEY_BOOTSTRAP: &str = "bootstrap";

/// MEMBER_TAG_VALUE_BOOTSTRAP is the value of the MemberTagKeyBootstrap key when the
/// agent was started with the "bootstrap" configuration enabled.
pub const MEMBER_TAG_VALUE_BOOTSTRAP: &str = "1";

/// MEMBER_TAG_KEY_BOOTSTRAP_EXPECT is the key name of the tag used to indicate whether
/// this agent was started with the "bootstrap_expect" configuration set to a non-zero
/// value. The value of this key will be the string for of that configuration value.
pub const MEMBER_TAG_KEY_BOOTSTRAP_EXPECT: &str = "expect";

/// MEMBER_TAG_KEY_USE_TLS is the key name of the tag used to indicate whther this agent
/// was configured to use TLS.
pub const MEMBER_TAG_KEY_USE_TLS: &str = "use_tls";

/// MEMBER_TAG_VALUE_USE_TLS is the value of the MemberTagKeyUseTLS when the agent was
/// configured to use TLS. Any other value indicates that it was not setup in
/// that manner.
pub const MEMBER_TAG_VALUE_USE_TLS: &str = "1";

/// MEMBER_TAG_KEY_READ_REPLICA is the key used to indicate that the member is a read
/// replica server (will remain a Raft non-voter).
/// Read Replicas are a Consul Enterprise feature.
pub const MEMBER_TAG_KEY_READ_REPLICA: &str = "read_replica";

/// MEMBER_TAG_VALUE_READ_REPLICA is the value of the MemberTagKeyReadReplica key when
/// the member is in fact a read-replica. Any other value indicates that it is not.
/// Read Replicas are a Consul Enterprise feature.
pub const MEMBER_TAG_VALUE_READ_REPLICA: &str = "1";

pub type MemberACLMode = String;

lazy_static!(
    /// ACL_MODE_DISABLED indicates that ACLs are disabled for this agent
    pub static ref ACL_MODE_DISABLED:Arc<MemberACLMode> = {
        Arc::new(String::from("0"))
    };

    /// ACL_MODE_ENABLED indicates that ACLs are enabled and operating in new ACL
    /// mode (v1.4.0+ ACLs)
    pub static ref ACL_MODE_ENABLED:Arc<MemberACLMode>  = {
        Arc::new(String::from("1"))
    };

    /// ACL_MODE_LEGACY indicates that ACLs are enabled and operating in legacy mode.
    pub static ref ACL_MODE_LEGACY:Arc<MemberACLMode>  = {
        Arc::new(String::from("2"))
    };

    /// ACL_MODE_UNKNOWN is used to indicate that the AgentMember.Tags didn't advertise
    /// an ACL mode at all. This is the case for Consul versions before v1.4.0 and
    /// should be treated similarly to ACLModeLegacy.
    pub static ref ACL_MODE_UNKNOWN:Arc<MemberACLMode>  = {
        Arc::new(String::from("3"))
    };
);

/// AgentMember represents a cluster member known to the agent
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AgentMember {
    pub Name: Option<String>,
    pub Addr: Option<String>,
    pub Port: Option<u16>,
    pub Tags: Option<HashMap<String, String>>,
    /// Status of the Member which corresponds to  github.com/hashicorp/serf/serf.MemberStatus
    /// Value is one of:
    ///   AgentMemberNone    = 0
    ///	  AgentMemberAlive   = 1
    ///	  AgentMemberLeaving = 2
    ///	  AgentMemberLeft    = 3
    ///	  AgentMemberFailed  = 4
    pub Status: Option<isize>,
    pub ProtocolMin: Option<u8>,
    pub ProtocolMax: Option<u8>,
    pub ProtocolCur: Option<u8>,
    pub DelegateMin: Option<u8>,
    pub DelegateMax: Option<u8>,
    pub DelegateCur: Option<u8>,
}

/// ACLMode returns the ACL mode this agent is operating in.
impl AgentMember {
    /// the key may not have existed but then an
    /// empty string will be returned and we will
    /// handle that in the default case of the switch
    pub async fn acl_mode(&self) -> MemberACLMode {
        return if self.Tags.is_some() {
            let tags = self.Tags.as_ref().unwrap();
            let mode = tags.get(MEMBER_TAG_KEY_ACL_MODE);
            return if mode.is_some() {
                let tag = mode.unwrap();
                if tag == &ACL_MODE_DISABLED.clone().to_string() {
                    ACL_MODE_DISABLED.clone().to_string()
                } else if tag == &ACL_MODE_ENABLED.clone().to_string() {
                    ACL_MODE_ENABLED.clone().to_string()
                } else if tag == &ACL_MODE_LEGACY.clone().to_string() {
                    ACL_MODE_LEGACY.clone().to_string()
                } else {
                    ACL_MODE_UNKNOWN.clone().to_string()
                }
            } else {
                ACL_MODE_UNKNOWN.clone().to_string()
            };
        } else {
            ACL_MODE_UNKNOWN.clone().to_string()
        };
    }

    /// IsConsulServer returns true when this member is a Consul server.
    pub async fn is_consul_server(&self) -> bool {
        return if self.Tags.is_some() {
            let tags = self.Tags.as_ref().unwrap();
            let key = &*MEMBER_TAG_KEY_ROLE.clone();
            let res = tags.get(key);
            return if res.is_some() {
                let tag = res.unwrap();
                if tag == &*MEMBER_TAG_VALUE_ROLE_SERVER.clone() {
                    true
                } else {
                    false
                }
            } else {
                false
            };
        } else {
            false
        };
    }
}

lazy_static!(
    /// ALL_SEGMENTS is used to select for all segments in MembersOpts.
    pub static ref ALL_SEGMENTS: Arc<String> = {
        Arc::new(String::from("_all"))
    };
);

/// MembersOpts is used for querying member information.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct MembersOpts {
    /// WAN is whether to show members from the WAN.
    pub WAN: Option<bool>,

    /// Segment is the LAN segment to show members for. Setting this to the
    /// AllSegments value above will show members in all segments.
    pub Segment: Option<String>,
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

/// ConnectProxyConfig is the response structure for agent-local proxy
/// configuration.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ConnectProxyConfig {
    pub ProxyServiceID: Option<String>,
    pub TargetServiceID: Option<String>,
    pub TargetServiceName: Option<String>,
    pub ContentHash: Option<String>,
    pub Config: HashMap<String, Value>,
    pub Upstreams: Vec<Upstream>,
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

/// Agent can be used to query the Agent endpoints
#[derive(Default, Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct Agent {
    pub c: Option<api::Client>,
    /// cache the node name
    nodeName: Option<&'static str>,
}

lazy_static! {
    pub static ref AGENT: Arc<RwLock<Agent>> = {
        let client = api::CLIENT.clone();
        let lock = block_on(client.read());
        let agent = block_on(lock.agent());
        Arc::new(RwLock::new(agent))
    };
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

impl Agent {
    pub async fn reload_client(){
        let client = api::CLIENT.clone();
        let client = client.read().await;
        let s = client.agent().await;
        let agent = AGENT.clone();
        let mut agent = agent.write().await;
        *agent = s;
    }

    /// my_self is used to query the agent we are speaking to for information about itself
    pub async fn my_self(&self) -> surf::Result<Map<String, Value>> {
        if self.c.is_some() {
            let client = self.c.unwrap();
            let req = client.new_request(Method::Get,
                                         "/v1/agent/self".to_string()).await?;
            let client = surf::Client::new();
            let mut res = client.send(req).await?;
            let body: Map<String, Value> = res.body_json().await?;
            Ok(body)
        } else {
            Err(Error::from_str(StatusCode::BadRequest, "client init err"))
        }
    }

    /// host is used to retrieve information about the host the
    /// agent is running on such as CPU, memory, and disk. Requires
    /// a operator:read ACL token.
    pub async fn host(&self) -> surf::Result<HashMap<String, Value>> {
        let client = self.c.unwrap();
        let req = client.new_request(Method::Get,
                                     "/v1/agent/host".to_string()).await?;
        let client = surf::Client::new();
        let mut res = client.send(req).await?;
        let body: HashMap<String, Value> = res.body_json().await?;
        Ok(body)
    }

    /// metrics is used to query the agent we are speaking to for
    /// its current internal metric data
    pub async fn metrics(&self) -> surf::Result<MetricsInfo> {
        if self.c.is_some() {
            let client = self.c.unwrap();
            let req = client.new_request(Method::Get,
                                         "/v1/agent/metrics".to_string()).await?;
            let client = surf::Client::new();
            let mut res = client.send(req).await?;
            let body: MetricsInfo = res.body_json().await?;
            Ok(body)
        } else {
            Err(Error::from_str(StatusCode::BadRequest, "client init err"))
        }
    }

    /// reload triggers a configuration reload for the agent we are connected to.
    pub async fn reload(&self) -> surf::Result<()> {
        if self.c.is_some() {
            let client = self.c.unwrap();
            let req = client.new_request(Method::Put,
                                         "/v1/agent/reload".to_string()).await?;
            let client = surf::Client::new();
            client.send(req).await?;
            Ok(())
        } else {
            Err(Error::from_str(400, "client init err"))
        }
    }

    async fn node(&self) -> surf::Result<String> {
        let info = self.my_self().await?;
        let config = info.get("Config").expect("Config key is not exist");
        let name = config.get("NodeName").expect("node name is not exist");
        let name = name.to_string();
        Ok(name)
    }

    /// node_name is used to get the node name of the agent
    pub async fn node_name(&self) -> surf::Result<String> {
        if self.nodeName.is_some() {
            let node_name = self.nodeName.unwrap();
            if node_name != "" {
                Ok(node_name.to_string())
            } else {
                let node_name = self.node().await?;
                Ok(node_name)
            }
        } else {
            let node_name = self.node().await?;
            Ok(node_name)
        }
    }

    /// checks returns the locally registered checks
    ///
    /// ```
    /// use async_std::task::block_on;
    /// use consul_rs::api;
    /// use consul_rs::agent;
    /// let lock = agent::AGENT.clone();
    /// let agent = block_on(lock.read());
    /// let res = block_on(agent.checks()).unwrap();
    /// println!("{:?}",res);
    /// ```
    pub async fn checks(&self) -> surf::Result<HashMap<String, AgentCheck>> {
        let val = self.checks_with_filter(Filter::default()).await?;
        Ok(val)
    }

    /// checks_with_filter returns a subset of the locally registered checks that match
    /// the given filter expression
    pub async fn checks_with_filter(&self, filter: Filter) -> surf::Result<HashMap<String, AgentCheck>> {
        let val = self.checks_with_filter_opts(filter,
                                               None).await?;
        Ok(val)
    }

    /// checks_with_filter_opts returns a subset of the locally registered checks that match
    /// the given filter expression and QueryOptions.
    pub async fn checks_with_filter_opts(&self,
                                         filter: Filter,
                                         opts: Option<api::QueryOptions>) -> surf::Result<HashMap<String, AgentCheck>> {
        if self.c.is_some() {
            let client = self.c.unwrap();
            let mut req = client.new_request(Method::Get,
                                             "/v1/agent/checks".to_string()).await?;
            if opts.is_some() {
                req.set_query(&opts.unwrap())?;
            }
            if filter.filter != "" {
                req.set_query(&filter)?;
            };

            let client = surf::Client::new();
            let mut res = client.send(req).await?;
            let body: HashMap<String, AgentCheck> = res.body_json().await?;
            Ok(body)
        } else {
            Err(Error::from_str(StatusCode::BadRequest, "client init err"))
        }
    }

    /// services returns the locally registered services
    ///
    /// ```
    /// use async_std::task::block_on;
    /// use consul_rs;
    /// let lock = consul_rs::agent::AGENT.clone();
    /// let agent = block_on(lock.read());
    /// let res = block_on(agent.services()).unwrap();
    /// println!("{:?}", res);
    /// ```
    pub async fn services(&self) -> surf::Result<HashMap<String, AgentService>> {
        return self.services_with_filter(Filter::default()).await;
    }

    /// services_with_filter returns a subset of the locally registered services that match
    /// the given filter expression
    pub async fn services_with_filter(&self, filter: Filter) -> surf::Result<HashMap<String, AgentService>> {
        return self.services_with_filter_opts(filter, None).await;
    }

    /// services_with_filter_opts returns a subset of the locally registered services that match
    /// the given filter expression and QueryOptions.
    pub async fn services_with_filter_opts(&self, filter: Filter, opts: Option<api::QueryOptions>) -> surf::Result<HashMap<String, AgentService>> {
        if self.c.is_some() {
            let client = self.c.unwrap();
            let mut req = client.new_request(Method::Get,
                                             "/v1/agent/services".to_string()).await?;
            if opts.is_some() {
                req.set_query(&opts.unwrap())?;
            }
            if filter.filter != "" {
                req.set_query(&filter)?;
            }
            let client = surf::Client::new();
            let mut res = client.send(req).await?;
            let body: HashMap<String, AgentService> = res.body_json().await?;
            Ok(body)
        } else {
            Err(Error::from_str(StatusCode::BadRequest, "client init err"))
        }
    }

    /// service_register is used to register a new service with
    /// the local agent
    ///
    /// ```
    /// use async_std::task::block_on;
    /// use consul_rs;
    /// let lock = consul_rs::agent::AGENT.clone();
    /// let agent = block_on(lock.read());
    /// let mut service = consul_rs::agent::AgentServiceRegistration::default();
    /// service.ID = Some("10".to_string());
    /// service.Name = Some("test".to_string());
    /// service.Address = Some("tttt".to_string());
    /// service.Port = Some(8080);
    /// let mut opts = consul_rs::agent::ServiceRegisterOpts::default();
    /// opts.ReplaceExistingChecks = true;
    /// let status_code = block_on(agent.service_register_opts(service, opts)).unwrap();
    /// assert_eq!(surf::StatusCode::Ok, status_code)
    /// ```
    pub async fn service_register(&self, service: AgentServiceRegistration) -> surf::Result<StatusCode> {
        let opts = ServiceRegisterOpts::default();
        let status = self.service_register_self(service, opts).await?;
        Ok(status)
    }

    /// service_register_opts is used to register a new service with
    /// the local agent and can be passed additional options.
    /// ```
    /// use async_std::task::block_on;
    /// use consul_rs;
    /// let lock = consul_rs::agent::AGENT.clone();
    /// let agent = block_on(lock.read());
    /// let mut service = consul_rs::agent::AgentServiceRegistration::default();
    /// service.ID = Some("10".to_string());
    /// service.Name = Some("test".to_string());
    /// service.Address = Some("tttt".to_string());
    /// service.Port = Some(8080);
    /// let mut opts = consul_rs::agent::ServiceRegisterOpts::default();
    /// opts.ReplaceExistingChecks = true;
    /// let status_code = block_on(agent.service_register_opts(service, opts)).unwrap();
    /// assert_eq!(surf::StatusCode::Ok, status_code)
    /// ```
    pub async fn service_register_opts(&self, service: AgentServiceRegistration, opts: ServiceRegisterOpts) -> surf::Result<StatusCode> {
        let status = self.service_register_self(service, opts).await?;
        Ok(status)
    }

    async fn service_register_self(&self, service: AgentServiceRegistration,
                                   opts: ServiceRegisterOpts) -> surf::Result<StatusCode> {
        if self.c.is_some() {
            let client = self.c.unwrap();
            let mut req = client.new_request(Method::Put,
                                             "/v1/agent/service/register".to_string()).await?;
            if opts.ReplaceExistingChecks == true {
                req.set_query(&opts)?;
            };
            req.body_json(&service)?;
            let client = surf::Client::new();
            let res = client.send(req).await?;
            Ok(res.status())
        } else {
            Err(Error::from_str(StatusCode::BadRequest, "client init err"))
        }
    }

    /// service_deregister is used to deregister a service with
    /// the local agent
    ///
    /// ```
    /// use async_std::task::block_on;
    /// use consul_rs::api;
    /// let lock = consul_rs::agent::AGENT.clone();
    /// let agent = block_on(lock.read());
    /// let status_code = block_on(agent.service_deregister("10".to_string())).unwrap();
    /// assert_eq!(surf::StatusCode::Ok, status_code)
    /// ```
    pub async fn service_deregister(&self, service_id: String) -> surf::Result<StatusCode> {
        if self.c.is_some() {
            let client = self.c.unwrap();
            let req = client.new_request(Method::Put,
                                         format!("/v1/agent/service/deregister/{}", service_id)).await?;
            let client = surf::Client::new();
            let res = client.send(req).await?;
            Ok(res.status())
        } else {
            Err(Error::from_str(StatusCode::BadRequest, "client init err"))
        }
    }

    /// service_deregister_opts is used to deregister a service with
    /// the local agent with QueryOptions.
    ///
    /// ```
    /// use async_std::task::block_on;
    /// use consul_rs;
    /// block_on(consul_rs::api::Client::set_config_address("http://0.0.0.0:8500"));
    /// block_on(consul_rs::agent::Agent::reload_client());
    /// let lock = consul_rs::agent::AGENT.clone();
    /// let agent = block_on(lock.read());
    /// let opts = consul_rs::api::QueryOptions::default();
    /// let status_code = block_on(agent.service_deregister_opts("10".to_string(), opts)).unwrap();
    /// assert_eq!(surf::StatusCode::Ok, status_code)
    /// ```
    pub async fn service_deregister_opts(&self, service_id: String, opts: api::QueryOptions) -> surf::Result<StatusCode> {
        if self.c.is_some() {
            let client = self.c.unwrap();
            let mut req = client.new_request(Method::Put,
                                             format!("/v1/agent/service/deregister/{}", service_id)).await?;
            req.set_query(&opts)?;
            let client = surf::Client::new();
            let res = client.send(req).await?;
            Ok(res.status())
        } else {
            Err(Error::from_str(StatusCode::BadRequest, "client init err"))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::agent;
    use crate::api;
    use async_std::task::block_on;
    #[test]
    fn test_my_self() {
        block_on(api::Client::set_config_address("http://0.0.0.0:8500"));
        block_on(agent::Agent::reload_client());
        let lock = agent::AGENT.clone();
        let agent = block_on(lock.read());
        let s = block_on(agent.my_self()).unwrap();
        println!("{:?}", s)
    }

    #[test]
    fn test_host() {
        let lock = agent::AGENT.clone();
        let agent = block_on(lock.read());
        let s = block_on(agent.host()).unwrap();
        println!("{:?}", s)
    }

    #[test]
    fn test_checks() {
        // block_on(api::Client::set_config_address("http://0.0.0.0:8500"));
        // block_on(Agent::reload_client());
        let lock = agent::AGENT.clone();
        let agent = block_on(lock.read());
        let s = block_on(agent.checks()).unwrap();
        println!("{:#?}", s);
    }

    #[test]
    fn test_services() {
        let lock = agent::AGENT.clone();
        let agent = block_on(lock.read());
        let res = block_on(agent.services()).unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn test_service_register() {
        let mut service = agent::AgentServiceRegistration::default();
        service.ID = Some("10".to_string());
        service.Name = Some("test".to_string());
        service.Address = Some("tttt".to_string());
        service.Port = Some(8080);
        let mut opts = agent::ServiceRegisterOpts::default();
        opts.ReplaceExistingChecks = true;
        let lock = agent::AGENT.clone();
        let agent = block_on(lock.read());
        let s = block_on(agent.service_register_opts(service, opts)).unwrap();
        println!("{}", s)
    }

    #[test]
    fn test_service_deregister() {
        let lock = agent::AGENT.clone();
        let agent = block_on(lock.read());
        let s = block_on(agent.service_deregister("10".to_string())).unwrap();
        println!("{}", s)
    }
}