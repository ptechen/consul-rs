use super::agent::{AgentServiceRegistration, ServiceRegisterOpts};
use super::health::{WaitPassing, ServiceAddress, ServiceEntry, Tag};
use super::watch::WatchService;
use async_std::sync::{Arc, RwLock};
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::time;
use surf;
use surf::http::Method;
use surf::{Error, StatusCode};
use rand::Rng;
use crate::health::Index;

lazy_static!(
    pub static ref CONSUL_CONFIG:Arc<RwLock<ConsulConfig>> = {
        let consul_config = ConsulConfig::default();
        let consul_config = RwLock::new(consul_config);
        Arc::new(consul_config)
    };

    pub static ref SERVICES_ADDRESS:Arc<RwLock<HashMap<String, ServiceAddress>>> = {
        let hash_map = HashMap::new();
        let hash_map = RwLock::new(hash_map);
        Arc::new(hash_map)
    };
);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsulConfig {
    pub config: Option<Config>,
    pub watch_services: Option<Vec<WatchService>>,
}

impl Default for ConsulConfig {
    fn default() -> Self {
        let mut config = Config::default();
        config.address = Some(String::from( "http://127.0.0.1:8500"));
        ConsulConfig{config: Some(config), watch_services: None}
    }
}

impl ConsulConfig {
    pub async fn new_request(&self, method: Method, path: String) -> surf::Result<surf::Request> {
        let config = self.config.as_ref().expect("consul config is empty");
        let address = config.address.as_ref().expect("consul config address is empty");
        let url = format!("{}{}", address, path);
        let uri = surf::Url::parse(&url)?;
        let mut req = surf::Request::new(method, uri);
        req.set_header("Connection", "close");
        let mut body: HashMap<String, String> = HashMap::new();

        if config.datacenter.is_some() {
            body.insert(String::from("dc"), String::from(config.datacenter.as_ref().unwrap()));
        };
        if config.namespace.is_some() {
            body.insert(String::from("ns"), String::from(config.namespace.as_ref().unwrap()));
        };

        if config.wait_time.is_some() {
            let wait_time = config.wait_time.unwrap();
            let wait_time_secs = wait_time.as_secs();
            let wait_time_str = format!("{}s", wait_time_secs);
            body.insert(String::from("wait"), wait_time_str);
        }

        if config.token.is_some() {
            body.insert("X-Consul-Token".to_string(),String::from(config.token.as_ref().unwrap()));
        };

        req.body_json(&body)?;
        Ok(req)
    }

    /// service_register is used to register a new service with
    /// the local agent
    ///
    /// ```
    /// use consul_rs::api::CONSUL_CONFIG;
    /// use async_std::task::block_on;
    /// use consul_rs::agent::AgentServiceRegistration;
    /// let clone_consul = CONSUL_CONFIG.clone();
    /// let consul = block_on(clone_consul.read());
    /// let mut service = AgentServiceRegistration::default();
    /// service.ID = Some(String::from("321"));
    /// service.Name = Some(String::from("test"));
    /// service.Port = Some(8080);
    /// service.Address = Some(String::from("127.0.0.1"));
    /// let s = block_on(consul.service_register(&service)).unwrap();
    /// println!("{}", s);
    /// ```
    pub async fn service_register(
        &self,
        service: &AgentServiceRegistration,
    ) -> surf::Result<StatusCode> {
        let opts = ServiceRegisterOpts::default();
        let status = self.service_register_self(service, &opts).await?;
        Ok(status)
    }

    pub async fn service_register_opts(
        &self,
        service: &AgentServiceRegistration,
        opts: &ServiceRegisterOpts,
    ) -> surf::Result<StatusCode> {
        let status = self.service_register_self(service, opts).await?;
        Ok(status)
    }

    pub async fn service_register_self(
        &self,
        service: &AgentServiceRegistration,
        opts: &ServiceRegisterOpts,
    ) -> surf::Result<StatusCode> {
        if self.config.is_some() {
            let mut req = self
                .new_request(Method::Put, "/v1/agent/service/register".to_string())
                .await?;
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

    // service_deregister is used to register a new service with
    // the local agent
    //
    // ```
    // use consul_rs::api::CONSUL_CONFIG;
    // use async_std::task::block_on;
    // use consul_rs::ConsulTrait;
    // use consul_rs::agent::AgentServiceRegistration;
    // let clone_consul = CONSUL_CONFIG.clone();
    // let consul = block_on(clone_consul.read());
    // let service_id = String::from("321");
    // let s = block_on(consul.service_deregister(service_id)).unwrap();
    // println!("{}", s);
    // ```
    pub async fn service_deregister(&self, service_id: String) -> surf::Result<StatusCode> {
        if self.config.is_some() {
            let req = self
                .new_request(
                    Method::Put,
                    format!("/v1/agent/service/deregister/{}", service_id),
                )
                .await?;
            let client = surf::Client::new();
            let res = client.send(req).await?;
            Ok(res.status())
        } else {
            Err(Error::from_str(StatusCode::BadRequest, "client init err"))
        }
    }

    /// ```
    /// use consul_rs::api::CONSUL_CONFIG;
    /// use async_std::task::block_on;
    /// use consul_rs::agent::AgentServiceRegistration;
    /// use consul_rs::watch::WatchService;
    /// let clone_consul = CONSUL_CONFIG.clone();
    /// let mut consul = block_on(clone_consul.read());
    /// consul.watch_services = Some(vec![WatchService{}]);
    /// let s = block_on(consul.watch_services()).unwrap();
    /// println!("{}", s);
    /// ```
    pub async fn watch_services(&self) -> surf::Result<StatusCode> {
        if self.watch_services.is_some() {
            loop {
                let watch_services = self.watch_services.as_ref().unwrap();
                let mut service_await = vec![];

                for watch_service in watch_services.iter() {
                    service_await.push(self.get_address(watch_service))
                }
                let services_addresses =  SERVICES_ADDRESS.clone();
                let mut services_addresses = services_addresses.write().await;
                for v in service_await.into_iter() {
                    let (key,service_address) = v.await?;
                    services_addresses.insert(key, service_address);
                }
            }
        }
        Ok(surf::http::StatusCode::Ok)
    }

    async fn health_service(&self, watch_service: &WatchService) -> surf::Result<Vec<ServiceEntry>> {
        let path = format!("/v1/health/service/{}", watch_service.service_name);
        if self.config.is_some() {
            let mut req = self.new_request(Method::Get, path).await?;
            if watch_service.query.is_some() {
                let opts = watch_service.query.as_ref().unwrap();
                req.set_query(opts)?;
            };
            let default = String::new();
            let tag = watch_service.tag.as_ref().unwrap_or(&default);
            req.set_query(&Tag {tag: String::from(tag)})?;
            let services_addresses = SERVICES_ADDRESS.clone();
            let services_addresses = services_addresses.read().await;
            let key  = format!("{}{}",watch_service.service_name, tag);
            let service_address = services_addresses.get(&key);
            if service_address.is_some() {
                let service_address = service_address.unwrap();
                let index = service_address.index;
                req.set_query(&Index{index})?;
            }

            if watch_service.passing_only.is_some() {
                let passing = watch_service.passing_only.unwrap();
                if passing {
                    let config = self.config.as_ref().unwrap();
                    let wait = config.wait_time.as_ref().unwrap().as_secs();
                    let wait = format!("{}s", wait);
                    let query = WaitPassing {
                        passing: String::from("1"),
                        wait,
                    };
                    req.set_query(&query)?;
                }
            };
            let client = surf::Client::new();
            let mut res = client.send(req).await?;
            let out: Vec<ServiceEntry> = res.body_json().await?;
            Ok(out)
        } else {
            Err(Error::from_str(StatusCode::BadRequest, "client init err"))
        }
    }

    async fn get_address(&self, watch_service: &WatchService) -> surf::Result<(String,ServiceAddress)> {
        let entry = self.health_service(watch_service).await?;
        let mut service_addresses = vec![];
        let mut service_addresses_link = LinkedList::new();
        let mut index = 0;
        for val in entry.iter() {
            if val.Service.is_some() {
                let v = val.Service.as_ref().unwrap();
                if v.Address.is_some() && v.Port.is_some() {
                    let address = v.Address.as_ref().unwrap();
                    let port = v.Port.as_ref().unwrap();
                    let address = format!("{}:{}", address, port);
                    service_addresses.push(address.to_owned());
                    service_addresses_link.push_back(address);
                    index = v.CreateIndex.unwrap();
                };
            };
        };
        let mut tag = "" ;
        if watch_service.tag.is_some() {
            tag = watch_service.tag.as_ref().unwrap();
        };
        let key = format!("{}{}", watch_service.service_name, tag);
        let service_addresses = ServiceAddress {
            index,
            address: service_addresses,
            address_link: service_addresses_link,
        };

        Ok((key, service_addresses))
    }

    pub async fn random_policy(&self, service_name: &str, tag: &str) -> surf::Result<String> {
        let key = format!("{}{}", service_name, tag);
        let services_addresses = SERVICES_ADDRESS.clone();
        let services_addresses = services_addresses.read().await;
        let service_addresses = services_addresses.get(&key);
        if service_addresses.is_some() {
            let service_addresses = service_addresses.unwrap();
            let range = service_addresses.address.len();
            if range == 0 {
                return  Err(Error::from_str(StatusCode::BadRequest, "consul server address is empty"));
            };
            let mut r = rand::thread_rng();
            let idx: usize = r.gen_range(0..range);
            let address = service_addresses.address.get(idx).unwrap();
            return Ok(String::from(address));
        }
        Err(Error::from_str(StatusCode::BadRequest, "consul server address is empty"))
    }
}

/// Config is used to configure the creation of a client
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Config {
    /// Address is the address of the Consul server
    pub address: Option<String>,

    /// Scheme is the URI scheme for the Consul server
    pub scheme: Option<String>,

    /// Datacenter to use. If not provided, the default agent datacenter is used.
    pub datacenter: Option<String>,

    /// Transport is the Transport to use for the http client.
    /// pub Transport: surf::Client,
    /// HttpClient is the client to use. Default will be
    /// used if not provided.
    /// pub HttpClient: Option<surf::Client>,

    /// HttpAuth is the auth info to use for http access.

    /// pub HttpAuth: Option<http_types::auth::BasicAuth>,

    /// WaitTime limits how long a Watch will block. If not provided,
    /// the agent default values will be used.
    pub wait_time: Option<time::Duration>,

    /// Token is used to provide a per-request ACL token
    /// which overrides the agent's default token.
    pub token: Option<String>,

    /// TokenFile is a file containing the current token to use for this client.
    /// If provided it is read once at startup and never again.
    pub token_file: Option<String>,

    /// Namespace is the name of the namespace to send along for the request
    /// when no other Namespace is present in the QueryOptions
    pub namespace: Option<String>,

    pub tls_config: Option<TLSConfig>,
}

/// TLSConfig is used to generate a TLSClientConfig that's useful for talking to
/// Consul using TLS.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TLSConfig {
    /// Address is the optional address of the Consul server. The port, if any
    /// will be removed from here and this will be set to the ServerName of the
    /// resulting config.
    pub address: Option<String>,

    /// CAFile is the optional path to the CA certificate used for Consul
    /// communication, defaults to the system bundle if not specified.
    pub ca_file: Option<String>,

    /// CAPath is the optional path to a directory of CA certificates to use for
    /// Consul communication, defaults to the system bundle if not specified.
    pub ca_path: Option<String>,

    /// CAPem is the optional PEM-encoded CA certificate used for Consul
    /// communication, defaults to the system bundle if not specified.
    pub ca_pem: Option<String>,

    /// CertFile is the optional path to the certificate for Consul
    /// communication. If this is set then you need to also set KeyFile.
    pub cert_file: Option<String>,

    /// CertPEM is the optional PEM-encoded certificate for Consul
    /// communication. If this is set then you need to also set KeyPEM.
    pub cert_pem: Option<String>,

    /// KeyFile is the optional path to the private key for Consul communication.
    /// If this is set then you need to also set CertFile.
    pub key_file: Option<String>,

    /// KeyPEM is the optional PEM-encoded private key for Consul communication.
    /// If this is set then you need to also set CertPEM.
    pub key_pem: Option<String>,

    /// InsecureSkipVerify if set to true will disable TLS host verification.
    pub insecure_skip_verify: Option<bool>,
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
    pub NodeMeta: Option<HashMap<String, String>>,

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
