#[cfg(target_os = "windows")]
use winapi::um::winbase::{
    ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, SetThreadExecutionState,
};

#[cfg(target_os = "macos")]
use core_foundation::runloop::CFRunLoop;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::Command;

pub fn prevent_sleep() {
    #[cfg(target_os = "windows")]
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
    }

    #[cfg(target_os = "macos")]
    {
        // Use caffeinate command or IOKit framework
        std::thread::spawn(|| {
            let _ = Command::new("caffeinate")
                .arg("-d") // prevent display sleep
                .spawn();
        });
    }

    #[cfg(target_os = "linux")]
    {
        // Use systemd-inhibit or xdg-screensaver
        let _ = Command::new("xdg-screensaver").arg("reset").spawn();
    }
}

pub fn allow_sleep() {
    #[cfg(target_os = "windows")]
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS);
    }
}
