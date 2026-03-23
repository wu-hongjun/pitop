use super::{BoardProfile, BoardType, VoltageSource};

#[derive(Debug)]
pub struct Zero2WProfile;

impl BoardProfile for Zero2WProfile {
    fn board_type(&self) -> BoardType {
        BoardType::Zero2W
    }

    fn name(&self) -> &str {
        "Raspberry Pi Zero 2 W"
    }

    fn soc_name(&self) -> &str {
        "BCM2710A1"
    }

    fn has_pmic(&self) -> bool {
        false
    }

    fn has_fan(&self) -> bool {
        false
    }

    fn has_pcie(&self) -> bool {
        false
    }

    fn has_poe(&self) -> bool {
        false
    }

    fn thermal_zones(&self) -> &[&str] {
        &["soc"]
    }

    fn voltage_source(&self) -> VoltageSource {
        VoltageSource::MeasureVolts
    }
}
