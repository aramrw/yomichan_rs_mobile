#![allow(warnings)]

use gpui::{prelude::*, App, WindowOptions, AppContext};
use std::sync::Arc;
use yomichan_rs::Yomichan;
use crate::screens::Router;
use crate::db::PendingNotesDb;

pub mod demos;
pub mod screens;
pub mod db;

// #[cfg(target_os = "macos")]
// #[global_allocator]
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// --- Globals ---
#[derive(Clone)]
pub struct GlobalYomichan(pub Arc<parking_lot::RwLock<Yomichan>>);
impl gpui::Global for GlobalYomichan {}
impl std::ops::Deref for GlobalYomichan {
    type Target = Arc<parking_lot::RwLock<Yomichan>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct GlobalPendingCards(pub Arc<parking_lot::RwLock<Vec<yomichan_rs::TermDictionaryEntry>>>);
impl gpui::Global for GlobalPendingCards {}
impl std::ops::Deref for GlobalPendingCards {
    type Target = Arc<parking_lot::RwLock<Vec<yomichan_rs::TermDictionaryEntry>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct GlobalPendingNotesDb(pub PendingNotesDb);
impl gpui::Global for GlobalPendingNotesDb {}
impl std::ops::Deref for GlobalPendingNotesDb {
    type Target = PendingNotesDb;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// --- iOS Logger ---
#[cfg(target_os = "ios")]
struct NsLogLogger;

#[cfg(target_os = "ios")]
impl log::Log for NsLogLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool { true }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("[{}] {}: {}", record.level(), record.target(), record.args());
            nslog(&msg);
        }
    }
    fn flush(&self) {}
}

#[cfg(target_os = "ios")]
fn nslog(msg: &str) {
    use objc::{class, msg_send, runtime::Object, sel, sel_impl};
    unsafe {
        extern "C" { fn NSLog(fmt: *mut Object, ...); }
        let c_msg = std::ffi::CString::new(msg).unwrap_or_default();
        let ns_msg: *mut Object = msg_send![class!(NSString), alloc];
        let ns_msg: *mut Object = msg_send![ns_msg, initWithUTF8String: c_msg.as_ptr()];
        let c_fmt = std::ffi::CString::new("%@").unwrap_or_default();
        let ns_fmt: *mut Object = msg_send![class!(NSString), alloc];
        let ns_fmt: *mut Object = msg_send![ns_fmt, initWithUTF8String: c_fmt.as_ptr()];
        NSLog(ns_fmt, ns_msg);
    }
}

// --- Main Entry Point ---
#[gpui_mobile::main]
pub fn main(cx: &mut App) {
    // 1. Setup Logging
    #[cfg(target_os = "ios")]
    {
        let _ = log::set_logger(&NsLogLogger).map(|()| log::set_max_level(log::LevelFilter::Info));
        std::panic::set_hook(Box::new(|info| {
            nslog(&format!("GPUI PANIC: {info}"));
        }));
    }

    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Info)
                .with_tag("yomichan-mobile"),
        );
        gpui_mobile::android::jni::install_panic_hook();
    }

    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Application starting...");

    // 2. Initialize Yomichan Data
    let data_dir = gpui_mobile::packages::path_provider::support_directory()
        .or_else(|_| gpui_mobile::packages::path_provider::documents_directory())
        .expect("Failed to get persistent data directory");
    
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).expect("could not create data dir");
    }

    let yomichan_instance = Yomichan::new(&data_dir).expect("Failed to initialize Yomichan");
    let yomichan_lock = Arc::new(parking_lot::RwLock::new(yomichan_instance.into()));

    // Ensure default language
    {
        let mut ycd: parking_lot::RwLockWriteGuard<yomichan_rs::Yomichan> = yomichan_lock.write();
        if ycd.options().read().get_current_profile().unwrap().read().options().general().language.is_empty() {
            ycd.set_language("ja").ok();
            let _ = ycd.update_options();
        }
    }

    cx.set_global(GlobalYomichan(yomichan_lock));
    cx.set_global(GlobalPendingCards(Arc::new(parking_lot::RwLock::new(Vec::new()))));
    cx.set_global(GlobalPendingNotesDb(PendingNotesDb::new(data_dir.join("pending_notes.db"))));

    // 3. Determine Initial Screen
    let initial_screen = match gpui_mobile::packages::deeplink::get_initial_link() {
        Ok(Some(url)) => screens::Screen::from_deeplink_url(&url).unwrap_or_default(),
        _ => screens::Screen::default(),
    };

    // 4. Open Window
    cx.open_window(
        WindowOptions {
            window_bounds: None,
            ..Default::default()
        },
        |window, cx| {
            cx.new(|cx| Router::with_initial_screen(initial_screen, window, cx))
        },
    ).expect("Failed to open main window");

    cx.activate(true);
}
