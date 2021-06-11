use serde_derive::{Serialize, Deserialize};
use std::time;
use surf;
use std::collections::HashMap;
use surf::http::Method;
use lazy_static::lazy_static;
use async_std::sync::{Arc, Mutex};
use async_std::task::block_on;
use super::agent::Agent;
use super::health::Health;

/// Client provides a client to the Consul API
#[derive(Default, Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct Client {
    // pub headers: Option<http_types::Headers>,
    pub config: Config
}

impl Client {
    pub async fn set_config(config: Config) {
        let client = CLIENT.clone();
        let mut s = block_on(client.lock());
        s.config = config;
    }

    pub async fn set_config_address(address: &'static  str) {
        let client = CLIENT.clone();
        let mut s = block_on(client.lock());
        s.config.Address = address;
    }

    pub async fn agent(self) -> Agent {
        let mut a = Agent::default();
        a.c = Some(self);
        a
    }

    pub async fn health(self) -> Health {
        let mut a = Health::default();
        a.c = Some(self);
        a
    }
}

lazy_static! {
    pub static ref CLIENT: Arc<Mutex<Client>> = {
        let mut client = Client::default();
        client.config.Address = "http://0.0.0.0:8500";
        Arc::new(Mutex::new(client))
    };
}



/// Config is used to configure the creation of a client
#[derive(Default, Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct Config {
    /// Address is the address of the Consul server
    pub Address: &'static str,

    /// Scheme is the URI scheme for the Consul server
    pub Scheme: &'static str,

    /// Datacenter to use. If not provided, the default agent datacenter is used.
    pub Datacenter: &'static str,

    /// Transport is the Transport to use for the http client.
    /// pub Transport: surf::Client,
    /// HttpClient is the client to use. Default will be
    /// used if not provided.
    /// pub HttpClient: Option<surf::Client>,

    /// HttpAuth is the auth info to use for http access.

    /// pub HttpAuth: Option<http_types::auth::BasicAuth>,

    /// WaitTime limits how long a Watch will block. If not provided,
    /// the agent default values will be used.
    pub WaitTime: time::Duration,

    /// Token is used to provide a per-request ACL token
    /// which overrides the agent's default token.
    pub Token: &'static str,

    /// TokenFile is a file containing the current token to use for this client.
    /// If provided it is read once at startup and never again.
    pub TokenFile: &'static str,

    /// Namespace is the name of the namespace to send along for the request
    /// when no other Namespace is present in the QueryOptions
    pub Namespace: &'static str,

    pub TLSConfig: TLSConfig,
}

/// TLSConfig is used to generate a TLSClientConfig that's useful for talking to
/// Consul using TLS.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TLSConfig {
    /// Address is the optional address of the Consul server. The port, if any
    /// will be removed from here and this will be set to the ServerName of the
    /// resulting config.
    pub Address: &'static str,

    /// CAFile is the optional path to the CA certificate used for Consul
    /// communication, defaults to the system bundle if not specified.
    pub CAFile: &'static str,

    /// CAPath is the optional path to a directory of CA certificates to use for
    /// Consul communication, defaults to the system bundle if not specified.
    pub CAPath: &'static str,

    /// CAPem is the optional PEM-encoded CA certificate used for Consul
    /// communication, defaults to the system bundle if not specified.
    pub CAPem: &'static str,

    /// CertFile is the optional path to the certificate for Consul
    /// communication. If this is set then you need to also set KeyFile.
    pub CertFile: &'static str,

    /// CertPEM is the optional PEM-encoded certificate for Consul
    /// communication. If this is set then you need to also set KeyPEM.
    pub CertPEM: &'static str,

    /// KeyFile is the optional path to the private key for Consul communication.
    /// If this is set then you need to also set CertFile.
    pub KeyFile: &'static str,

    /// KeyPEM is the optional PEM-encoded private key for Consul communication.
    /// If this is set then you need to also set CertPEM.
    pub KeyPEM: &'static str,

    /// InsecureSkipVerify if set to true will disable TLS host verification.
    pub InsecureSkipVerify: bool,
}

/// newRequest is used to create a new request
impl Client {
    pub async fn new_request(self, method: Method, path: String) -> surf::Result<surf::Request> {
        let url = format!("{}{}", self.config.Address, path);
        let uri = surf::Url::parse(&url)?;
        let mut req = surf::Request::new(method, uri);

        let mut body: HashMap<String, String> = HashMap::new();

        if self.config.Datacenter != "" {
            body.insert("dc".to_string(), self.config.Datacenter.to_string());
        };
        if self.config.Namespace != "" {
            body.insert("ns".to_string(), self.config.Namespace.to_string());
        };

        if self.config.WaitTime.as_secs() > 0 {
            let wait_time_secs = self.config.WaitTime.as_secs();
            let wait_time_str = wait_time_secs.to_string();
            body.insert("wait".to_string(), wait_time_str);
        };

        if self.config.Token != "" {
            body.insert("X-Consul-Token".to_string(), self.config.Token.to_string());
        };

        req.body_json(&body)?;

        Ok(req)
    }

}

/// QueryOptions are used to parameterize a query
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct QueryOptions {
    /// Namespace overrides the `default` namespace
    /// Note: Namespaces are available only in Consul Enterprise
    pub Namespace: Option<String>,

    /// Providing a datacenter overwrites the DC provided
    /// by the Config
    pub Datacenter: Option<String>,

    /// AllowStale allows any Consul server (non-leader) to service
    /// a read. This allows for lower latency and higher throughput
    pub AllowStale: Option<bool>,

    /// RequireConsistent forces the read to be fully consistent.
    /// This is more expensive but prevents ever performing a stale
    /// read.
    pub RequireConsistent: Option<bool>,

    /// UseCache requests that the agent cache results locally. See
    /// https:///www.consul.io/api/features/caching.html for more details on the
    /// semantics.
    pub UseCache: Option<bool>,

    /// MaxAge limits how old a cached value will be returned if UseCache is true.
    /// If there is a cached response that is older than the MaxAge, it is treated
    /// as a cache miss and a new fetch invoked. If the fetch fails, the error is
    /// returned. Clients that wish to allow for stale results on error can set
    /// StaleIfError to a longer duration to change this behavior. It is ignored
    /// if the endpoint supports background refresh caching. See
    /// https:///www.consul.io/api/features/caching.html for more details.
    pub MaxAge: Option<time::Duration>,

    /// StaleIfError specifies how stale the client will accept a cached response
    /// if the servers are unavailable to fetch a fresh one. Only makes sense when
    /// UseCache is true and MaxAge is set to a lower, non-zero value. It is
    /// ignored if the endpoint supports background refresh caching. See
    /// https:///www.consul.io/api/features/caching.html for more details.
    pub StaleIfError: Option<time::Duration>,

    /// WaitIndex is used to enable a blocking query. Waits
    /// until the timeout or the next index is reached
    pub WaitIndex: Option<usize>,

    /// WaitHash is used by some endpoints instead of WaitIndex to perform blocking
    /// on state based on a hash of the response rather than a monotonic index.
    /// This is required when the state being blocked on is not stored in Raft, for
    /// example agent-local proxy configuration.
    pub WaitHash: Option<String>,

    /// WaitTime is used to bound the duration of a wait.
    /// Defaults to that of the Config, but can be overridden.
    pub WaitTime: Option<time::Duration>,

    /// Token is used to provide a per-request ACL token
    /// which overrides the agent's default token.
    pub Token: Option<String>,

    /// Near is used to provide a node name that will sort the results
    /// in ascending order based on the estimated round trip time from
    /// that node. Setting this to "_agent" will use the agent's node
    /// for the sort.
    pub Near: Option<String>,

    /// NodeMeta is used to filter results by nodes with the given
    /// metadata key/value pairs. Currently, only one key/value pair can
    /// be provided for filtering.
    pub NodeMeta: HashMap<String, String>,

    /// RelayFactor is used in keyring operations to cause responses to be
    /// relayed back to the sender through N other random nodes. Must be
    /// a value from 0 to 5 (inclusive).
    pub RelayFactor: Option<u8>,

    /// LocalOnly is used in keyring list operation to force the keyring
    /// query to only hit local servers (no WAN traffic).
    pub LocalOnly: Option<bool>,

    /// Connect filters prepared query execution to only include Connect-capable
    /// services. This currently affects prepared query execution.
    pub Connect: Option<bool>,

    /// Filter requests filtering data prior to it being returned. The string
    /// is a go-bexpr compatible expression.
    pub Filter: Option<String>,
}
