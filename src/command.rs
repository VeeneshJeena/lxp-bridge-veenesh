use crate::prelude::*;

#[derive(Debug)]
pub enum Command {
    ReadInputs(config::Inverter, u16),
    ReadInput(config::Inverter, u16, u16),
    ReadHold(config::Inverter, u16, u16),
    ReadParam(config::Inverter, u16),
    ReadAcChargeTime(config::Inverter, u16),
    ReadAcFirstTime(config::Inverter, u16),
    ReadChargePriorityTime(config::Inverter, u16),
    ReadForcedDischargeTime(config::Inverter, u16),
    SetHold(config::Inverter, u16, u16),
    WriteParam(config::Inverter, u16, u16),
    SetAcChargeTime(config::Inverter, u16, [u8; 4]),
    SetAcFirstTime(config::Inverter, u16, [u8; 4]),
    SetChargePriorityTime(config::Inverter, u16, [u8; 4]),
    SetForcedDischargeTime(config::Inverter, u16, [u8; 4]),
    ChargeRate(config::Inverter, u16),
    DischargeRate(config::Inverter, u16),
    AcCharge(config::Inverter, bool),
    ChargePriority(config::Inverter, bool),
    ForcedDischarge(config::Inverter, bool),
    AcChargeRate(config::Inverter, u16),
    AcChargeSocLimit(config::Inverter, u16),
    DischargeCutoffSocLimit(config::Inverter, u16),
}

impl Command {
    pub fn to_result_topic(&self) -> String {
        use Command::*;

        let rest = match self {
            ReadInputs(inverter, c) => format!("{}/read/inputs/{}", inverter.datalog(), c),
            ReadInput(inverter, register, _) => {
                format!("{}/read/input/{}", inverter.datalog(), register)
            }
            ReadHold(inverter, register, _) => {
                format!("{}/read/hold/{}", inverter.datalog(), register)
            }
            ReadParam(inverter, register) => {
                format!("{}/read/param/{}", inverter.datalog(), register)
            }
            ReadAcChargeTime(inverter, num) => {
                format!("{}/read/ac_charge/{}", inverter.datalog(), num)
            }
            ReadAcFirstTime(inverter, num) => {
                format!("{}/read/ac_first/{}", inverter.datalog(), num)
            }
            ReadChargePriorityTime(inverter, num) => {
                format!("{}/read/charge_priority/{}", inverter.datalog(), num)
            }
            ReadForcedDischargeTime(inverter, num) => {
                format!("{}/read/forced_discharge/{}", inverter.datalog(), num)
            }
            SetHold(inverter, register, _) => {
                format!("{}/set/hold/{}", inverter.datalog(), register)
            }
            WriteParam(inverter, register, _) => {
                format!("{}/set/param/{}", inverter.datalog(), register)
            }
            SetAcChargeTime(inverter, num, _) => {
                format!("{}/set/ac_charge/{}", inverter.datalog(), num)
            }
            SetAcFirstTime(inverter, num, _) => {
                format!("{}/set/ac_first/{}", inverter.datalog(), num)
            }
            SetChargePriorityTime(inverter, num, _) => {
                format!("{}/set/charge_priority/{}", inverter.datalog(), num)
            }
            SetForcedDischargeTime(inverter, num, _) => {
                format!("{}/set/forced_discharge/{}", inverter.datalog(), num)
            }
            AcCharge(inverter, _) => format!("{}/set/ac_charge", inverter.datalog()),
            ChargePriority(inverter, _) => format!("{}/set/charge_priority", inverter.datalog()),
            ForcedDischarge(inverter, _) => format!("{}/set/forced_discharge", inverter.datalog()),
            ChargeRate(inverter, _) => format!("{}/set/charge_rate_pct", inverter.datalog()),
            DischargeRate(inverter, _) => format!("{}/set/discharge_rate_pct", inverter.datalog()),
            AcChargeRate(inverter, _) => format!("{}/set/ac_charge_rate_pct", inverter.datalog()),
            AcChargeSocLimit(inverter, _) => {
                format!("{}/set/ac_charge_soc_limit_pct", inverter.datalog())
            }
            DischargeCutoffSocLimit(inverter, _) => {
                format!("{}/set/discharge_cutoff_soc_limit_pct", inverter.datalog())
            }
        };

        format!("result/{}", rest)
    }

    pub fn is_write_operation(&self) -> bool {
        match self {
            Command::SetHold(_, _, _)
            | Command::WriteParam(_, _, _)
            | Command::SetAcChargeTime(_, _, _)
            | Command::SetAcFirstTime(_, _, _)
            | Command::SetChargePriorityTime(_, _, _)
            | Command::SetForcedDischargeTime(_, _, _)
            | Command::AcCharge(_, _)
            | Command::ChargePriority(_, _)
            | Command::ForcedDischarge(_, _)
            | Command::ChargeRate(_, _)
            | Command::DischargeRate(_, _)
            | Command::AcChargeRate(_, _)
            | Command::AcChargeSocLimit(_, _)
            | Command::DischargeCutoffSocLimit(_, _) => true,

            Command::ReadInputs(_, _) // Note: Corrected from ReadInputs(_, _, _) as per actual enum
            | Command::ReadInput(_, _, _)
            | Command::ReadHold(_, _, _)
            | Command::ReadParam(_, _)
            | Command::ReadAcChargeTime(_, _)
            | Command::ReadAcFirstTime(_, _)
            | Command::ReadChargePriorityTime(_, _)
            | Command::ReadForcedDischargeTime(_, _) => false,
        }
    }

    pub fn get_inverter_config(&self) -> &config::Inverter {
        match self {
            Command::ReadInputs(inverter, _) |
            Command::ReadInput(inverter, _, _) |
            Command::ReadHold(inverter, _, _) |
            Command::ReadParam(inverter, _) |
            Command::ReadAcChargeTime(inverter, _) |
            Command::ReadAcFirstTime(inverter, _) |
            Command::ReadChargePriorityTime(inverter, _) |
            Command::ReadForcedDischargeTime(inverter, _) |
            Command::SetHold(inverter, _, _) |
            Command::WriteParam(inverter, _, _) |
            Command::SetAcChargeTime(inverter, _, _) |
            Command::SetAcFirstTime(inverter, _, _) |
            Command::SetChargePriorityTime(inverter, _, _) |
            Command::SetForcedDischargeTime(inverter, _, _) |
            Command::AcCharge(inverter, _) |
            Command::ChargePriority(inverter, _) |
            Command::ForcedDischarge(inverter, _) |
            Command::ChargeRate(inverter, _) |
            Command::DischargeRate(inverter, _) |
            Command::AcChargeRate(inverter, _) |
            Command::AcChargeSocLimit(inverter, _) |
            Command::DischargeCutoffSocLimit(inverter, _) => inverter,
        }
    }

    pub fn variant_name(&self) -> &'static str {
        match self {
            Command::ReadInputs(_, _) => "ReadInputs",
            Command::ReadInput(_, _, _) => "ReadInput",
            Command::ReadHold(_, _, _) => "ReadHold",
            Command::ReadParam(_, _) => "ReadParam",
            Command::ReadAcChargeTime(_, _) => "ReadAcChargeTime",
            Command::ReadAcFirstTime(_, _) => "ReadAcFirstTime",
            Command::ReadChargePriorityTime(_, _) => "ReadChargePriorityTime",
            Command::ReadForcedDischargeTime(_, _) => "ReadForcedDischargeTime",
            Command::SetHold(_, _, _) => "SetHold",
            Command::WriteParam(_, _, _) => "WriteParam",
            Command::SetAcChargeTime(_, _, _) => "SetAcChargeTime",
            Command::SetAcFirstTime(_, _, _) => "SetAcFirstTime",
            Command::SetChargePriorityTime(_, _, _) => "SetChargePriorityTime",
            Command::SetForcedDischargeTime(_, _, _) => "SetForcedDischargeTime",
            Command::AcCharge(_, _) => "AcCharge",
            Command::ChargePriority(_, _) => "ChargePriority",
            Command::ForcedDischarge(_, _) => "ForcedDischarge",
            Command::ChargeRate(_, _) => "ChargeRate",
            Command::DischargeRate(_, _) => "DischargeRate",
            Command::AcChargeRate(_, _) => "AcChargeRate",
            Command::AcChargeSocLimit(_, _) => "AcChargeSocLimit",
            Command::DischargeCutoffSocLimit(_, _) => "DischargeCutoffSocLimit",
        }
    }
}
