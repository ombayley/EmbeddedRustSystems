//! System/Device glue: logging, panic handler, boot metadata, etc.

/// defmt RTT logger (link-time side effects)
use defmt_rtt as _;

/// Panic handler based on target arch
#[cfg(target_arch = "riscv32")]
use panic_halt as _;
#[cfg(target_arch = "arm")]
use panic_probe as _;

use embassy_rp as hal;
use embassy_rp::block::ImageDef;

/// Tell the Boot ROM about our application (RP235x)
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = hal::block::ImageDef::secure_exe();

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Blinky Example"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];

/// Optional: any runtime init hooks you want.
/// (You can also leave this empty and just `use crate::sys as _;` in main.)
pub fn init() {
    // If you ever need to init heap, global alloc, etc., do it here.
}

// End of File
