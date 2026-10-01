#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use librustdesk::*;

#[cfg(any(target_os = "android", target_os = "ios", feature = "flutter"))]
fn main() {
    if !common::global_init() {
        eprintln!("Global initialization failed.");
        return;
    }
    common::test_rendezvous_server();
    common::test_nat_type();
    common::global_clean();
}

#[cfg(not(any(target_os = "android", target_os = "ios", feature = "flutter")))]
fn main() -> std::process::ExitCode {
    #[cfg(windows)]
    if let Some(exit_code) = platform::handle_service_control() {
        return exit_code;
    }
    #[cfg(all(windows, not(feature = "inline")))]
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(2);
    }
    // Preserve command failures, notably verified macOS offline update failures,
    // as a non-zero process exit status for CLI callers.
    let exit_code = match crate::core_main::core_main() {
        Some(crate::core_main::CoreMainAction::StartUi(mut args)) => {
            // Linux is compiled headless. The host service started above keeps running.
            #[cfg(target_os = "linux")]
            run_linux_service(&args);
            #[cfg(not(target_os = "linux"))]
            ui::start(&mut args);
            std::process::ExitCode::SUCCESS
        }
        Some(crate::core_main::CoreMainAction::ExitFailure(err)) => {
            eprintln!("Command failed: {err}");
            hbb_common::log::error!("Command failed: {err}");
            std::process::ExitCode::FAILURE
        }
        None => std::process::ExitCode::SUCCESS,
    };
    common::global_clean();
    exit_code
}

#[cfg(all(target_os = "linux", not(feature = "flutter")))]
fn run_linux_service(args: &[String]) {
    let headless = args.is_empty()
        || args
            .first()
            .is_some_and(|arg| common::is_empty_uni_link(arg));
    if !headless {
        eprintln!(
            "`{}` needs a UI, but rustdesk has been compiled headless.",
            args[0]
        );
        std::process::exit(1);
    }
    std::thread::spawn(ipc::start_pa);
    let id = hbb_common::config::Config::get_id();
    eprintln!("RustDesk host service is running. ID: {id}");
    loop {
        std::thread::park();
    }
}
