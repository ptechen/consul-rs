#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod agent;
pub mod api;
pub mod catalog;
pub mod config_entry;
pub mod health;
pub mod watch;

use agent::{AgentServiceRegistration, ServiceRegisterOpts};
use async_trait::async_trait;
use health::{ServiceAddress, ServiceEntry};
use surf::{http::Method, StatusCode};
use watch::WatchService;

#[async_trait]
pub trait ConsulTrait {
    async fn new_request(&self, method: Method, path: String) -> surf::Result<surf::Request>;

    /// service_register is used to register a new service with
    /// the local agent
    async fn service_register(&self, service: &AgentServiceRegistration)
        -> surf::Result<StatusCode>;
    async fn service_register_opts(
        &self,
        service: &AgentServiceRegistration,
        opts: &ServiceRegisterOpts,
    ) -> surf::Result<StatusCode>;
    async fn service_register_self(
        &self,
        service: &AgentServiceRegistration,
        opts: &ServiceRegisterOpts,
    ) -> surf::Result<StatusCode>;

    /// service_deregister is used to deregister a service with
    /// the local agent
    async fn service_deregister(&self, service_id: String) -> surf::Result<StatusCode>;

    /// watch_services
    async fn watch_services(&self) -> surf::Result<StatusCode>;

    async fn health_service(&self, watch_service: &WatchService) -> surf::Result<Vec<ServiceEntry>>;

    async fn get_address(&self, params: &WatchService) -> surf::Result<(String, ServiceAddress)>;

    async fn random_policy(&self, service_name: &str, tag: &str) -> surf::Result<String>;
}
