mod board;
mod collectors;
mod util;

use anyhow::Result;
use std::path::Path;

fn main() -> Result<()> {
    let root = Path::new("/");

    let board_type = board::detect(root);
    let profile = board::create_profile(board_type);
    let system_info = board::collect_system_info(root);

    println!("pitop v{}", env!("CARGO_PKG_VERSION"));
    println!("Board: {} ({:?})", profile.name(), profile.board_type());
    println!("Model: {}", system_info.model_name);
    println!("Kernel: {}", system_info.kernel_version);
    println!("Host: {}", system_info.hostname);
    println!("OS: {} {}", system_info.os_name, system_info.os_version);
    println!();
    println!("Capabilities:");
    println!("  PMIC:  {}", profile.has_pmic());
    println!("  Fan:   {}", profile.has_fan());
    println!("  PCIe:  {}", profile.has_pcie());
    println!("  PoE:   {}", profile.has_poe());
    println!("  Volts: {:?}", profile.voltage_source());
    println!("  Thermal zones: {:?}", profile.thermal_zones());

    Ok(())
}
