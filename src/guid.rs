//! Having different GUIDs for debug and release builds is required because Windows treats the
//! resulting binaries as different.

use windows::core::GUID;

#[cfg(debug_assertions)]
pub const ICON_GUID: GUID = GUID::from_u128(0x128b0e05_5bce_4f87_8ea4_2f79b2797015);

#[cfg(not(debug_assertions))]
pub const ICON_GUID: GUID = GUID::from_u128(0x098588ea_adc5_4b17_a2c0_a5a2bb92fa4f);
