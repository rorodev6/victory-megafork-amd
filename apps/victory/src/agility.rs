//! App-local D3D12 Agility SDK exports read by the system D3D12 loader.
#![allow(non_upper_case_globals)]

/// Matches the minor version of the Microsoft.Direct3D.D3D12 package
/// installed by the agility xtask.
pub const SDK_VERSION: u32 = 620;

#[cfg(victory_agility_sdk)]
#[unsafe(no_mangle)]
pub static D3D12SDKVersion: u32 = SDK_VERSION;

#[cfg(victory_agility_sdk)]
#[repr(transparent)]
pub struct SdkPath(*const std::ffi::c_char);

// SAFETY: The exported pointer addresses an immutable static C string.
#[cfg(victory_agility_sdk)]
unsafe impl Sync for SdkPath {}

#[cfg(victory_agility_sdk)]
#[unsafe(no_mangle)]
pub static D3D12SDKPath: SdkPath = SdkPath(c".\\D3D12\\".as_ptr());

pub fn show_status() {
    if !cfg!(victory_agility_sdk) {
        println!("d3d12 sdk   : system runtime (Windows 10: run cargo xtask agility and rebuild)");
        return;
    }
    let dll = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("D3D12").join("D3D12Core.dll")));
    match dll {
        Some(dll) if dll.is_file() => {
            println!(
                "d3d12 sdk   : app-local SDK {SDK_VERSION} at {}",
                dll.display()
            )
        }
        _ => println!("d3d12 sdk   : app-local SDK {SDK_VERSION}, D3D12\\D3D12Core.dll missing"),
    }
}
