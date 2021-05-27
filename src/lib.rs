#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod agent;
pub mod health;
pub mod api;
pub mod catalog;
pub mod config_entry;