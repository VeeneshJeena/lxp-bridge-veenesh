mod common;
use common::*;

// Added imports
use crate::command::Command;
use crate::config; // Assuming config::Inverter is used directly
use crate::coordinator::Coordinator;
use crate::channels::Channels;
use tokio::sync::broadcast::error::TryRecvError; // Already used, but good to note
use lxp::packet::Packet; // Already used, but good to note

#[tokio::test]
async fn publishes_read_hold_mqtt() {
    common_setup();

    // setup config with only mqtt enabled
    let config = Factory::example_config_wrapped();
    config.influx_mut().enabled = false;
    config.databases_mut()[0].enabled = false;

    let inverter = &config.inverters()[0].clone();

    let channels = Channels::new();

    let coordinator = Coordinator::new(config, channels.clone());

    let tf = async {
        let mut to_influx = channels.to_influx.subscribe();
        let mut to_mqtt = channels.to_mqtt.subscribe();
        let mut to_db = channels.to_database.subscribe();
        let mut to_register_cache = channels.to_register_cache.subscribe();

        // simulate ReadHold in from inverter
        let packet = Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: inverter.datalog(),
            device_function: lxp::packet::DeviceFunction::ReadHold,
            inverter: inverter.serial(),
            register: 12,
            values: vec![22, 6],
        });
        channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))?;

        // verify register_cache is set
        let register_cache::ChannelData::RegisterData(a, b) = to_register_cache.recv().await?
        else {
            unreachable!()
        };
        assert_eq!(a, 12);
        assert_eq!(b, 1558);

        // verify MQTT output
        assert_eq!(
            to_mqtt.recv().await?,
            mqtt::ChannelData::Message(mqtt::Message {
                topic: format!("{}/hold/12", inverter.datalog()),
                retain: true,
                payload: "1558".to_owned()
            })
        );
        // verify nothing sent to influx or database
        assert_eq!(to_influx.try_recv(), Err(TryRecvError::Empty));
        assert_eq!(to_db.try_recv(), Err(TryRecvError::Empty));

        coordinator.stop();

        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(coordinator.start(), tf).unwrap();
}

// Helper function to create a Coordinator instance and a basic Inverter config for tests
// The Coordinator's internal config can be default, as process_command now
// uses the Inverter config directly from the Command enum for read-only checks.
fn setup_coordinator_and_inverter(is_read_only: bool) -> (Coordinator, config::Inverter) {
    common::common_setup(); // Ensure common setup is called
    let config_wrapper = common::Factory::example_config_wrapped(); // Get a default config
    let mut inverter_template = config_wrapper.inverters()[0].clone(); // Use as a template

    inverter_template.read_only = Some(is_read_only);
    // Ensure other fields are valid enough not to cause unrelated panics,
    // like ensuring serial and datalog are not empty if Command constructors need them.
    // The Factory::example_config_wrapped() should provide valid serials.

    let channels = Channels::new();
    let coordinator = Coordinator::new(config_wrapper, channels);

    (coordinator, inverter_template)
}

#[tokio::test]
async fn test_process_command_readonly_sethold_blocked() {
    let (coordinator, read_only_inverter_config) = setup_coordinator_and_inverter(true);

    // Ensure the command uses the inverter config we just set up
    let cmd = Command::SetHold(read_only_inverter_config.clone(), 10, 100); 

    let result = coordinator.process_command(cmd).await;
    assert!(result.is_err(), "Command should have been blocked by read-only mode");
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("read-only mode"), "Error message should mention read-only: {}", error_msg);
        assert!(error_msg.contains("SetHold"), "Error message should mention command name: {}", error_msg);
    }
}

#[tokio::test]
async fn test_process_command_writable_sethold_allowed() {
    let (coordinator, writable_inverter_config) = setup_coordinator_and_inverter(false);

    let cmd = Command::SetHold(writable_inverter_config.clone(), 10, 100);

    let result = coordinator.process_command(cmd).await;
    // In a real scenario, this might be Ok(()) or a different error if the command execution fails
    // for reasons other than read-only. For this test, we mainly care it's NOT a read-only error.
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(!error_msg.contains("read-only mode"), "Error message should NOT mention read-only: {}", error_msg);
    }
    // If it's Ok(()), it passes implicitly, which is fine.
}

#[tokio::test]
async fn test_process_command_readonly_readhold_allowed() {
    let (coordinator, read_only_inverter_config) = setup_coordinator_and_inverter(true);

    let cmd = Command::ReadHold(read_only_inverter_config.clone(), 12, 1); // register 12, count 1

    let result = coordinator.process_command(cmd).await;
    // ReadHold should be allowed even in read-only mode.
    // It might fail for other reasons (e.g., inverter not responding in a real scenario),
    // but it shouldn't be due to a read-only block.
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(!error_msg.contains("read-only mode"), "ReadHold command should not be blocked by read-only: {}", error_msg);
    }
    // If it's Ok(()), it passes.
}

#[tokio::test]
async fn test_process_command_readonly_accharge_blocked() {
    let (coordinator, read_only_inverter_config) = setup_coordinator_and_inverter(true);

    let cmd = Command::AcCharge(read_only_inverter_config.clone(), true); // Enable AcCharge

    let result = coordinator.process_command(cmd).await;
    assert!(result.is_err(), "Command should have been blocked by read-only mode");
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("read-only mode"), "Error message should mention read-only: {}", error_msg);
        assert!(error_msg.contains("AcCharge"), "Error message should mention command name: {}", error_msg);
    }
}

#[tokio::test]
#[cfg_attr(not(feature = "mocks"), ignore)]
async fn handles_read_input_all() {
    common_setup();

    let config = Factory::example_config_wrapped();
    config.influx_mut().enabled = true;
    config.databases_mut()[0].enabled = true;
    let inverter = config.inverters()[0].clone();

    let channels = Channels::new();

    let coordinator = Coordinator::new(config, channels.clone());

    let tf = async {
        let mut to_influx = channels.to_influx.subscribe();
        let mut to_mqtt = channels.to_mqtt.subscribe();
        let mut to_database = channels.to_database.subscribe();

        // simulate ReadHold in from inverter
        let packet = Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: inverter.datalog(),
            device_function: lxp::packet::DeviceFunction::ReadInput,
            inverter: inverter.serial(),
            register: 0,
            values: vec![1; 254],
        });
        channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))?;

        // verify MQTT output. numbers look odd but they're just cos of all the 1's in the input
        assert_eq!(
            to_mqtt.recv().await?,
            mqtt::ChannelData::Message(mqtt::Message {
                topic: format!("{}/inputs/all", inverter.datalog()),
                retain: false,
                payload: "{\"status\":257,\"v_pv_1\":25.7,\"v_pv_2\":25.7,\"v_pv_3\":25.7,\"v_bat\":25.7,\"soc\":1,\"soh\":1,\"internal_fault\":257,\"p_pv\":771,\"p_pv_1\":257,\"p_pv_2\":257,\"p_pv_3\":257,\"p_battery\":0,\"p_charge\":257,\"p_discharge\":257,\"v_ac_r\":25.7,\"v_ac_s\":25.7,\"v_ac_t\":25.7,\"f_ac\":2.57,\"p_inv\":257,\"p_rec\":257,\"pf\":0.257,\"v_eps_r\":25.7,\"v_eps_s\":25.7,\"v_eps_t\":25.7,\"f_eps\":2.57,\"p_eps\":257,\"s_eps\":257,\"p_grid\":0,\"p_to_grid\":257,\"p_to_user\":257,\"e_pv_day\":77.1,\"e_pv_day_1\":25.7,\"e_pv_day_2\":25.7,\"e_pv_day_3\":25.7,\"e_inv_day\":25.7,\"e_rec_day\":25.7,\"e_chg_day\":25.7,\"e_dischg_day\":25.7,\"e_eps_day\":25.7,\"e_to_grid_day\":25.7,\"e_to_user_day\":25.7,\"v_bus_1\":25.7,\"v_bus_2\":25.7,\"e_pv_all\":5052902.7,\"e_pv_all_1\":1684300.9,\"e_pv_all_2\":1684300.9,\"e_pv_all_3\":1684300.9,\"e_inv_all\":1684300.9,\"e_rec_all\":1684300.9,\"e_chg_all\":1684300.9,\"e_dischg_all\":1684300.9,\"e_eps_all\":1684300.9,\"e_to_grid_all\":1684300.9,\"e_to_user_all\":1684300.9,\"fault_code\":16843009,\"warning_code\":16843009,\"t_inner\":257,\"t_rad_1\":257,\"t_rad_2\":257,\"t_bat\":257,\"runtime\":16843009,\"max_chg_curr\":2.57,\"max_dischg_curr\":2.57,\"charge_volt_ref\":25.7,\"dischg_cut_volt\":25.7,\"bat_status_0\":257,\"bat_status_1\":257,\"bat_status_2\":257,\"bat_status_3\":257,\"bat_status_4\":257,\"bat_status_5\":257,\"bat_status_6\":257,\"bat_status_7\":257,\"bat_status_8\":257,\"bat_status_9\":257,\"bat_status_inv\":257,\"bat_count\":257,\"bat_capacity\":257,\"bat_current\":2.57,\"bms_event_1\":257,\"bms_event_2\":257,\"max_cell_voltage\":0.257,\"min_cell_voltage\":0.257,\"max_cell_temp\":25.7,\"min_cell_temp\":25.7,\"bms_fw_update_state\":257,\"cycle_count\":257,\"vbat_inv\":25.7,\"time\":1646370367,\"datalog\":\"2222222222\"}".to_owned()
            })
        );

        // verify influx and database output
        let d = unwrap_influx_channeldata_input_data(to_influx.recv().await?);
        assert_eq!(d["soc"], 1);
        assert_eq!(d["v_pv_1"], 25.7);
        let d = unwrap_database_channeldata_read_input_all(to_database.recv().await?);
        assert_eq!(d.soc, 1);
        assert_eq!(d.v_pv_1, 25.7);

        coordinator.stop();

        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(coordinator.start(), tf).unwrap();
}

#[tokio::test]
async fn complete_path_read_hold_command() {
    common_setup();

    let config = Factory::example_config_wrapped();

    let inverter = config.inverters()[0].clone();

    let channels = Channels::new();

    let coordinator = Coordinator::new(config, channels.clone());

    let tf = async {
        let mut to_inverter = channels.to_inverter.subscribe();
        let mut to_mqtt = channels.to_mqtt.subscribe();
        let mut to_register_cache = channels.to_register_cache.subscribe();

        // simulate:
        //   mqtt incoming "read this hold" command
        let message = mqtt::Message {
            topic: "cmd/all/read/hold/12".to_owned(),
            retain: false,
            payload: "".to_owned(),
        };
        channels
            .from_mqtt
            .send(mqtt::ChannelData::Message(message))
            .unwrap();

        //   wait for inverter to receive the right packet
        let packet = Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: inverter.datalog(),
            device_function: lxp::packet::DeviceFunction::ReadHold,
            inverter: inverter.serial(),
            register: 12,
            values: vec![1, 0],
        });
        assert_eq!(
            to_inverter.recv().await?,
            lxp::inverter::ChannelData::Packet(packet),
        );

        //   send the packet back from the inverter
        let reply = Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: inverter.datalog(),
            device_function: lxp::packet::DeviceFunction::ReadHold,
            inverter: inverter.serial(),
            register: 12,
            values: vec![22, 6],
        });
        channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Packet(reply))
            .unwrap();

        //   wait for mqtt to get the right responses
        assert_eq!(
            to_mqtt.recv().await?,
            mqtt::ChannelData::Message(mqtt::Message {
                topic: "2222222222/hold/12".to_owned(),
                retain: true,
                payload: "1558".to_owned()
            })
        );
        assert_eq!(
            to_mqtt.recv().await?,
            mqtt::ChannelData::Message(mqtt::Message {
                topic: "result/2222222222/read/hold/12".to_owned(),
                retain: false,
                payload: "OK".to_owned()
            })
        );

        // verify register_cache is set
        let register_cache::ChannelData::RegisterData(a, b) = to_register_cache.recv().await?
        else {
            unreachable!()
        };
        assert_eq!(a, 12);
        assert_eq!(b, 1558);

        coordinator.stop();

        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(coordinator.start(), tf).unwrap();
}
