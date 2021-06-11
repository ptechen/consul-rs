#[cfg(test)]
mod tests {
    use async_std::task::block_on;
    use crate::set_client_address;

    #[test]
    fn it_works() {
        block_on(set_client_address("123"));
        assert_eq!(2 + 2, 4);
    }
}

pub mod agent;
pub mod health;
pub mod api;
pub mod catalog;
pub mod config_entry;

pub async fn set_client_address(address: &'static str) {
    api::Client::set_config_address(address).await;
    health::Health::reload_client().await;
    agent::Agent::reload_client().await;
}