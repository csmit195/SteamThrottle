use crate::policy::{BandwidthAction, bytes_per_second_to_steam_kbps};
use std::{
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use windows_sys::Win32::{
    System::Threading::{AttachThreadInput, GetCurrentThreadId},
    UI::WindowsAndMessaging::{
        BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, LSFW_LOCK, LSFW_UNLOCK,
        LockSetForegroundWindow, SetForegroundWindow,
    },
};

fn should_restore_foreground(original: usize, current: usize) -> bool {
    original != 0 && original != current
}

struct ForegroundGuard {
    original: windows_sys::Win32::Foundation::HWND,
    lock_acquired: bool,
}

impl ForegroundGuard {
    fn new() -> Self {
        // SAFETY: These calls only read and temporarily lock the desktop's foreground-window state.
        let original = unsafe { GetForegroundWindow() };
        let lock_acquired = unsafe { LockSetForegroundWindow(LSFW_LOCK) != 0 };
        Self {
            original,
            lock_acquired,
        }
    }
}

impl Drop for ForegroundGuard {
    fn drop(&mut self) {
        // SAFETY: Window handles and thread IDs come directly from Win32. Failed focus operations
        // are harmless and deliberately ignored because throttling must not fail with UI focus.
        unsafe {
            if self.lock_acquired {
                LockSetForegroundWindow(LSFW_UNLOCK);
            }

            let current = GetForegroundWindow();
            if !should_restore_foreground(self.original as usize, current as usize) {
                return;
            }

            let caller_thread = GetCurrentThreadId();
            let foreground_thread = GetWindowThreadProcessId(current, std::ptr::null_mut());
            let attached = foreground_thread != 0
                && foreground_thread != caller_thread
                && AttachThreadInput(caller_thread, foreground_thread, 1) != 0;

            BringWindowToTop(self.original);
            SetForegroundWindow(self.original);

            if attached {
                AttachThreadInput(caller_thread, foreground_thread, 0);
            }
        }
    }
}

pub fn command_for(action: &BandwidthAction) -> Vec<String> {
    match action {
        BandwidthAction::Unlimited => {
            vec!["+set_download_throttle".into(), "0".into(), "false".into()]
        }
        BandwidthAction::Limit { bytes_per_second } => vec![
            "+set_download_throttle".into(),
            bytes_per_second_to_steam_kbps(*bytes_per_second).to_string(),
            "false".into(),
        ],
        BandwidthAction::Pause => vec!["+app_download_enable".into(), "0".into()],
    }
}

pub fn commands_for_transition(
    previous: Option<&BandwidthAction>,
    next: &BandwidthAction,
) -> Vec<Vec<String>> {
    let mut commands = vec![command_for(next)];
    if matches!(previous, Some(BandwidthAction::Pause)) && !matches!(next, BandwidthAction::Pause) {
        commands.push(vec!["+app_download_enable".into(), "1".into()]);
    }
    commands
}

pub fn default_steam_path() -> Option<PathBuf> {
    [
        PathBuf::from(r"C:\Program Files (x86)\Steam\steam.exe"),
        PathBuf::from(r"C:\Program Files\Steam\steam.exe"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

fn select_steam_path(
    running_path: Option<PathBuf>,
    installed_fallback: Option<PathBuf>,
) -> Option<PathBuf> {
    running_path.or(installed_fallback)
}

pub fn running_steam_path(system: &System) -> Option<PathBuf> {
    system.processes().values().find_map(|process| {
        process
            .name()
            .eq_ignore_ascii_case("steam.exe")
            .then(|| process.exe())
            .flatten()
            .filter(|path| path.is_file())
            .map(Path::to_path_buf)
    })
}

pub fn discover_steam_path() -> Option<PathBuf> {
    let mut system = System::new();
    let _ = system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
    select_steam_path(running_steam_path(&system), default_steam_path())
}

pub fn parse_latest_throttle(log: &str) -> Option<BandwidthAction> {
    let marker = "Current download throttle rate: ";
    let kbps = log
        .rsplit(marker)
        .next()?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    if kbps == 0 {
        Some(BandwidthAction::Unlimited)
    } else {
        Some(BandwidthAction::Limit {
            bytes_per_second: kbps.saturating_mul(1_000) / 8,
        })
    }
}

pub fn read_current_throttle(steam_path: &Path) -> io::Result<Option<BandwidthAction>> {
    let _foreground = ForegroundGuard::new();
    let status = Command::new(steam_path)
        .args(["-ifrunning", "+get_download_throttle"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "Steam query exited with {status}"
        )));
    }
    std::thread::sleep(std::time::Duration::from_millis(180));
    let log_path = steam_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("logs")
        .join("console_log.txt");
    std::fs::read_to_string(log_path).map(|log| parse_latest_throttle(&log))
}

pub fn invoke_steam_transition(
    steam_path: &Path,
    previous: Option<&BandwidthAction>,
    next: &BandwidthAction,
) -> io::Result<()> {
    if steam_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case("steam.exe"))
        != Some(true)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Steam path must point to steam.exe",
        ));
    }
    let _foreground = ForegroundGuard::new();
    for args in commands_for_transition(previous, next) {
        let status = Command::new(steam_path)
            .arg("-ifrunning")
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !status.success() {
            return Err(io::Error::other(format!(
                "Steam command exited with {status}"
            )));
        }
    }
    // steam.exe forwards commands to the existing client and can exit before the client
    // processes them. Keep the focus guard alive through that delayed activation window.
    std::thread::sleep(std::time::Duration::from_millis(300));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::BandwidthAction;

    #[test]
    fn builds_documented_runtime_console_commands() {
        assert_eq!(
            command_for(&BandwidthAction::Unlimited),
            vec!["+set_download_throttle", "0", "false"]
        );
        assert_eq!(
            command_for(&BandwidthAction::Pause),
            vec!["+app_download_enable", "0"]
        );
        assert_eq!(
            command_for(&BandwidthAction::Limit {
                bytes_per_second: 10_000_000
            }),
            vec!["+set_download_throttle", "80000", "false"]
        );
    }

    #[test]
    fn resuming_sets_the_new_rate_before_enabling_downloads() {
        let commands = commands_for_transition(
            Some(&BandwidthAction::Pause),
            &BandwidthAction::Limit {
                bytes_per_second: 10_000_000,
            },
        );
        assert_eq!(
            commands,
            vec![
                vec!["+set_download_throttle", "80000", "false"],
                vec!["+app_download_enable", "1"],
            ]
        );
    }

    #[test]
    fn parses_latest_reported_throttle_from_console_log() {
        let log = "Current download throttle rate: 160000 Kbps\nnoise\nCurrent download throttle rate: 80000 Kbps";
        assert_eq!(
            parse_latest_throttle(log),
            Some(BandwidthAction::Limit {
                bytes_per_second: 10_000_000
            })
        );
        assert_eq!(
            parse_latest_throttle("Current download throttle rate: 0 Kbps"),
            Some(BandwidthAction::Unlimited)
        );
    }

    #[test]
    fn foreground_is_restored_only_when_another_window_took_focus() {
        assert!(!should_restore_foreground(0, 44));
        assert!(!should_restore_foreground(44, 44));
        assert!(should_restore_foreground(44, 91));
    }

    #[test]
    fn running_steam_path_takes_priority_over_an_installed_fallback() {
        let running = PathBuf::from(r"D:\Portable\Steam\steam.exe");
        let installed = PathBuf::from(r"C:\Program Files (x86)\Steam\steam.exe");
        assert_eq!(
            select_steam_path(Some(running.clone()), Some(installed)),
            Some(running)
        );
    }
}
