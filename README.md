# consul api

## consul config, consul.yaml
    config:
      address: http://127.0.0.1:8500
      datacenter: dc1
      wait_time: 5s

    watch_services:
      - service_name: hyat_rust
        passing_only: true
        tag: ''

## load config
    ConsulConfig::load_config("consul.yaml").await?;

## consul watch 
    let consul_config = CONSUL_CONFIG.clone();
    let consul_config = consul_config.read().await;
    consul_config.watch_services().await?;