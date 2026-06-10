//! Backward compatibility shim for platform.
//! Re-exports from the new backend module.

pub mod native {
    #[cfg(feature = "reg")]
    pub use crate::toolkit::registry as reg;
    pub use crate::toolkit::monitors;
    #[cfg(feature = "gpu")]
    pub use crate::toolkit::gpu;
    #[cfg(feature = "gpu")]
    pub use crate::toolkit::wgpu_renderer;
    pub use crate::toolkit::config;
    
    #[cfg(feature = "sys-info")]
    pub mod sys_info {
        pub use crate::toolkit::sys_info::*;
        
        #[cfg(target_os = "windows")]
        pub use crate::toolkit::sys_info::providers::WindowsPlatform;
        
        #[cfg(target_os = "linux")]
        pub use crate::toolkit::sys_info::providers::LinuxPlatform;
        
        #[cfg(all(
            not(any(target_os = "windows", target_os = "linux")),
            not(target_arch = "wasm32"),
            not(any(target_os = "android", target_os = "ios")),
            not(any(target_os = "none", target_os = "uefi"))
        ))]
        pub use crate::toolkit::sys_info::providers::FallbackPlatform;
    }
}

pub use crate::toolkit::platform_web as web;
pub use crate::toolkit::platform_mobile as mobile;
pub use crate::toolkit::platform_embedded as embedded;

pub use crate::toolkit::platform::PowerStatus;
pub use crate::toolkit::platform::SystemBiosInfo;
pub use crate::toolkit::platform::DiskDriveInfo;
pub use crate::toolkit::platform::NetworkAdapterInfo;
pub use crate::toolkit::platform::PlatformProvider;

#[cfg(all(target_os = "windows", feature = "sys-info"))]
pub use crate::toolkit::sys_info::providers::WindowsPlatform as CurrentPlatform;

#[cfg(all(target_os = "linux", feature = "sys-info"))]
pub use crate::toolkit::sys_info::providers::LinuxPlatform as CurrentPlatform;

#[cfg(all(not(any(target_os = "windows", target_os = "linux")), target_arch = "wasm32"))]
pub use crate::toolkit::platform_web::WebPlatform as CurrentPlatform;

#[cfg(all(not(any(target_os = "windows", target_os = "linux")), any(target_os = "android", target_os = "ios")))]
pub use crate::toolkit::platform_mobile::MobilePlatform as CurrentPlatform;

#[cfg(all(not(any(target_os = "windows", target_os = "linux")), any(target_os = "none", target_os = "uefi")))]
pub use crate::toolkit::platform_embedded::EmbeddedPlatform as CurrentPlatform;

#[cfg(all(
    not(any(target_os = "windows", target_os = "linux")),
    not(target_arch = "wasm32"),
    not(any(target_os = "android", target_os = "ios")),
    not(any(target_os = "none", target_os = "uefi"))
))]
#[cfg(feature = "sys-info")]
pub use crate::toolkit::sys_info::providers::FallbackPlatform as CurrentPlatform;
