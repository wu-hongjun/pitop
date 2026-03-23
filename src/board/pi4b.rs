use super::{BoardProfile, BoardType, VoltageSource};

#[derive(Debug)]
pub struct Pi4BProfile;

impl BoardProfile for Pi4BProfile {
    fn board_type(&self) -> BoardType {
        BoardType::Pi4B
    }

    fn name(&self) -> &str {
        "Raspberry Pi 4 Model B"
    }

    fn soc_name(&self) -> &str {
        "BCM2711"
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
        true
    }

    fn thermal_zones(&self) -> &[&str] {
        &["soc"]
    }

    fn voltage_source(&self) -> VoltageSource {
        VoltageSource::MeasureVolts
    }
}
