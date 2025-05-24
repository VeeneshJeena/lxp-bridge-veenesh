mod common;
use common::*;

pub fn example_serial() -> lxp::inverter::Serial {
    lxp::inverter::Serial::from_str("TESTSERIAL").unwrap()
}

#[test]
fn config_returns_err_on_nonexistent_file() {
    let config = Config::new("nonexistent".to_owned());

    assert!(config.is_err());
}

#[test]
fn config_returns_ok() {
    let config = Config::new("config.yaml.example".to_owned());

    assert!(config.is_ok());
}

#[test]
fn inverter_defaults() {
    let input =
        json!({ "host": "host", "port": 8000, "serial": "TESTSERIAL", "datalog": "TESTDATALO" });
    let inverter: config::Inverter = serde_json::from_value(input).unwrap();
    assert!(inverter.enabled());
    assert_eq!(inverter.heartbeats(), false);
    assert_eq!(inverter.publish_holdings_on_connect(), false);
}

#[test]
fn inverter_heartbeats() {
    let input = json!({ "host": "host", "port": 8000, "serial": "TESTSERIAL", "datalog": "TESTDATALO", "heartbeats": false });
    let inverter: config::Inverter = serde_json::from_value(input).unwrap();
    assert_eq!(inverter.heartbeats(), false);
    let input = json!({ "host": "host", "port": 8000, "serial": "TESTSERIAL", "datalog": "TESTDATALO", "heartbeats": true });
    let inverter: config::Inverter = serde_json::from_value(input).unwrap();
    assert_eq!(inverter.heartbeats(), true);
}

#[test]
fn inverter_publish_holdings_on_connect() {
    let input = json!({ "host": "host", "port": 8000, "serial": "TESTSERIAL", "datalog": "TESTDATALO", "publish_holdings_on_connect": false });
    let inverter: config::Inverter = serde_json::from_value(input).unwrap();
    assert_eq!(inverter.publish_holdings_on_connect(), false);
    let input = json!({ "host": "host", "port": 8000, "serial": "TESTSERIAL", "datalog": "TESTDATALO", "publish_holdings_on_connect": true });
    let inverter: config::Inverter = serde_json::from_value(input).unwrap();
    assert_eq!(inverter.publish_holdings_on_connect(), true);
}

#[test]
fn database_defaults() {
    let input = json!({ "url": "url" });
    let database: config::Database = serde_json::from_value(input).unwrap();
    assert!(database.enabled());
}

#[test]
fn mqtt_defaults() {
    let input = json!({ "host": "host" });
    let mqtt: config::Mqtt = serde_json::from_value(input).unwrap();
    assert!(mqtt.enabled());
    assert_eq!(mqtt.port(), 1883);
    assert_eq!(mqtt.namespace(), "lxp");
}

#[test]
fn homeassistant_defaults() {
    let input = json!({});
    let ha: config::HomeAssistant = serde_json::from_value(input).unwrap();
    assert!(ha.enabled());
    assert_eq!(ha.prefix(), "homeassistant");
}

#[test]
fn enabled_inverters() {
    let config = Factory::example_config_wrapped();

    config.set_inverters(vec![
        config::Inverter {
            enabled: false,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
            heartbeats: None,
            publish_holdings_on_connect: None,
            read_timeout: None,
        },
        config::Inverter {
            enabled: true,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
            heartbeats: None,
            publish_holdings_on_connect: None,
            read_timeout: None,
        },
    ]);

    assert_eq!(config.enabled_inverters().len(), 1);
}

#[test]
fn inverters_for_message() {
    let config = Factory::example_config_wrapped();

    config.set_inverters(vec![
        config::Inverter {
            enabled: true,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
            heartbeats: None,
            publish_holdings_on_connect: None,
            read_timeout: None,
        },
        config::Inverter {
            enabled: false,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
            heartbeats: None,
            publish_holdings_on_connect: None,
            read_timeout: None,
        },
    ]);

    let message = mqtt::Message {
        topic: "cmd/all/foo".to_string(),
        retain: false,
        payload: "foo".to_string(),
    };

    let r = config.inverters_for_message(&message).unwrap();
    assert_eq!(r.len(), 1);

    let message = mqtt::Message {
        topic: "cmd/MISMATCHED/foo".to_string(),
        retain: false,
        payload: "foo".to_string(),
    };

    let r = config.inverters_for_message(&message).unwrap();
    assert_eq!(r.len(), 0);

    let message = mqtt::Message {
        topic: "cmd/TESTSERIAL/foo".to_string(),
        retain: false,
        payload: "foo".to_string(),
    };

    let r = config.inverters_for_message(&message).unwrap();
    assert_eq!(r.len(), 1);
}

#[test]
fn enabled_databases() {
    let config = Factory::example_config_wrapped();

    config.set_databases(vec![
        config::Database {
            enabled: false,
            url: "sqlite://test.db".to_owned(),
        },
        config::Database {
            enabled: true,
            url: "sqlite://test.db".to_owned(),
        },
    ]);

    assert_eq!(config.enabled_databases().len(), 1);
}

#[test]
fn test_inverter_config_read_only() {
    let yaml_true = r#"
inverters:
- host: "1.2.3.4"
  port: 8000
  serial: "TEST01"
  datalog: "DATA01"
  read_only: true
mqtt:
  host: localhost
influx:
  url: http://localhost:8086
  database: lxp
    "#;
    let config_true: Config = serde_yaml::from_str(yaml_true).unwrap();
    assert_eq!(config_true.inverters[0].read_only, Some(true));
    assert_eq!(config_true.inverters[0].is_read_only(), true);

    let yaml_false = r#"
inverters:
- host: "1.2.3.4"
  port: 8000
  serial: "TEST01"
  datalog: "DATA01"
  read_only: false
mqtt:
  host: localhost
influx:
  url: http://localhost:8086
  database: lxp
    "#;
    let config_false: Config = serde_yaml::from_str(yaml_false).unwrap();
    assert_eq!(config_false.inverters[0].read_only, Some(false));
    assert_eq!(config_false.inverters[0].is_read_only(), false);

    let yaml_default = r#"
inverters:
- host: "1.2.3.4"
  port: 8000
  serial: "TEST01"
  datalog: "DATA01"
  # read_only field omitted
mqtt:
  host: localhost
influx:
  url: http://localhost:8086
  database: lxp
    "#;
    let config_default: Config = serde_yaml::from_str(yaml_default).unwrap();
    assert_eq!(config_default.inverters[0].read_only, None);
    assert_eq!(config_default.inverters[0].is_read_only(), false); // Defaults to false
}

#[test]
fn test_inverter_config_poll_interval_seconds() {
    let yaml_interval_60 = r#"
inverters:
- host: "1.2.3.4"
  port: 8000
  serial: "TEST01"
  datalog: "DATA01"
  poll_interval_seconds: 60
mqtt:
  host: localhost
influx:
  url: http://localhost:8086
  database: lxp
    "#;
    let config_60: Config = serde_yaml::from_str(yaml_interval_60).unwrap();
    assert_eq!(config_60.inverters[0].poll_interval_seconds, Some(60));
    assert_eq!(config_60.inverters[0].poll_interval_seconds(), Some(60));
    assert_eq!(config_60.inverters[0].is_polling_enabled(), true);

    let yaml_interval_0 = r#"
inverters:
- host: "1.2.3.4"
  port: 8000
  serial: "TEST01"
  datalog: "DATA01"
  poll_interval_seconds: 0
mqtt:
  host: localhost
influx:
  url: http://localhost:8086
  database: lxp
    "#;
    let config_0: Config = serde_yaml::from_str(yaml_interval_0).unwrap();
    assert_eq!(config_0.inverters[0].poll_interval_seconds, Some(0));
    assert_eq!(config_0.inverters[0].poll_interval_seconds(), Some(0));
    assert_eq!(config_0.inverters[0].is_polling_enabled(), false);

    let yaml_interval_default = r#"
inverters:
- host: "1.2.3.4"
  port: 8000
  serial: "TEST01"
  datalog: "DATA01"
  # poll_interval_seconds field omitted
mqtt:
  host: localhost
influx:
  url: http://localhost:8086
  database: lxp
    "#;
    let config_default: Config = serde_yaml::from_str(yaml_interval_default).unwrap();
    assert_eq!(config_default.inverters[0].poll_interval_seconds, None);
    assert_eq!(config_default.inverters[0].poll_interval_seconds(), None);
    assert_eq!(config_default.inverters[0].is_polling_enabled(), false);
}
