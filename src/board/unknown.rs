use super::{BoardProfile, BoardType, VoltageSource};

#[derive(Debug)]
pub struct UnknownProfile;

impl BoardProfile for UnknownProfile {
    fn board_type(&self) -> BoardType {
        BoardType::Unknown
    }

    fn name(&self) -> &str {
        "Generic Linux"
    }

    fn soc_name(&self) -> &str {
        "Unknown"
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
        VoltageSource::None
    }
}
