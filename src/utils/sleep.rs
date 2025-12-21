#[cfg(target_os = "macos")]
use once_cell::sync::Lazy;
#[cfg(target_os = "windows")]
use winapi::um::winbase::{
    ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, SetThreadExecutionState,
};

#[cfg(target_os = "macos")]
use std::process::Child;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::Command;
#[cfg(target_os = "macos")]
use std::sync::Mutex;

#[cfg(target_os = "macos")]
static CAFFEINATE_CHILD: Lazy<Mutex<Option<Child>>> = Lazy::new(|| Mutex::new(None));

pub fn prevent_sleep() {
    #[cfg(target_os = "windows")]
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
    }

    #[cfg(target_os = "macos")]
    {
        let mut guard = CAFFEINATE_CHILD.lock().unwrap();
        if guard.is_none() {
            if let Ok(child) = Command::new("caffeinate")
                .arg("-d") // prevent display sleep
                .spawn()
            {
                *guard = Some(child);
            }
        }
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

    #[cfg(target_os = "macos")]
    {
        let mut guard = CAFFEINATE_CHILD.lock().unwrap();
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
