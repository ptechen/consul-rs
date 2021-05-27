use lazy_static::lazy_static;
use serde_derive::{Serialize, Deserialize};
pub type ProxyMode = String;

lazy_static!(
    /// ProxyModeDefault represents no specific mode and should
	/// be used to indicate that a different layer of the configuration
	/// chain should take precedence
    pub static ref PROXY_MODE_DEFAULT: ProxyMode = {
        String::new()
    };

    /// ProxyModeTransparent represents that inbound and outbound application
	/// traffic is being captured and redirected through the proxy.
    pub static ref PROXY_MODE_TRANSPARENT: ProxyMode = {
        String::from("transparent")
    };

    /// ProxyModeDirect represents that the proxy's listeners must be dialed directly
	/// by the local application and other proxies.
    pub static ref PROXY_MODE_DIRECT: ProxyMode = {
        String::from("direct")
    };
);

pub type MeshGatewayMode = String;

lazy_static!(
    /// MeshGatewayModeDefault represents no specific mode and should
    /// be used to indicate that a different layer of the configuration
    /// chain should take precedence
    pub static ref MESH_GATEWAY_MODE_DEFAULT: MeshGatewayMode = {
        String::new()
    };

    /// MeshGatewayModeNone represents that the Upstream Connect connections
    /// should be direct and not flow through a mesh gateway.
    pub static ref MESH_GATEWAY_MODE_NONE: MeshGatewayMode = {
        String::from("none")
    };

    /// MeshGatewayModeLocal represents that the Upstream Connect connections
    /// should be made to a mesh gateway in the local datacenter.
    pub static ref MESH_GATEWAY_MODE_LOCAL: MeshGatewayMode = {
        String::from("local")
    };

    /// MeshGatewayModeRemote represents that the Upstream Connect connections
    /// should be made to a mesh gateway in a remote datacenter.
    pub static ref MESH_GATEWAY_MODE_REMOTE: MeshGatewayMode = {
        String::from("remote")
    };
);

/// MeshGatewayConfig controls how Mesh Gateways are used for upstream Connect services
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct MeshGatewayConfig {
    // Mode is the mode that should be used for the upstream connection.
    pub Mode: Option<MeshGatewayMode>,
}

/// ExposeConfig describes HTTP paths to expose through Envoy outside of Connect.
/// Users can expose individual paths and/or all HTTP/GRPC paths for checks.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ExposeConfig {
    /// Checks defines whether paths associated with Consul checks will be exposed.
    /// This flag triggers exposing all HTTP and GRPC check paths registered for the service.
    pub Checks: Option<bool>,

    /// Paths is the list of paths exposed through the proxy.
    pub Paths: Option<Vec<ExposePath>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ExposePath  {
    /// ListenerPort defines the port of the proxy's listener for exposed paths.
    pub ListenerPort: Option<usize>,

    /// Path is the path to expose through the proxy, ie. "/metrics."
    pub Path: Option<String>,

    /// LocalPathPort is the port that the service is listening on for the given path.
    pub LocalPathPort: Option<usize>,

    /// Protocol describes the upstream's service protocol.
    /// Valid values are "http" and "http2", defaults to "http"
    pub Protocol: Option<String>,

    /// ParsedFromCheck is set if this path was parsed from a registered check
    pub ParsedFromCheck: Option<bool>,
}