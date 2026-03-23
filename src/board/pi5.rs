use super::{BoardProfile, BoardType, VoltageSource};

#[derive(Debug)]
pub struct Pi5Profile;

impl BoardProfile for Pi5Profile {
    fn board_type(&self) -> BoardType {
        BoardType::Pi5
    }

    fn name(&self) -> &str {
        "Raspberry Pi 5"
    }

    fn soc_name(&self) -> &str {
        "BCM2712"
    }

    fn has_pmic(&self) -> bool {
        true
    }

    fn has_fan(&self) -> bool {
        true
    }

    fn has_pcie(&self) -> bool {
        true
    }

    fn has_poe(&self) -> bool {
        true
    }

    fn thermal_zones(&self) -> &[&str] {
        &["soc", "pmic", "rp1"]
    }

    fn voltage_source(&self) -> VoltageSource {
        VoltageSource::Pmic
    }
}
