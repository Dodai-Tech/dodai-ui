//! The Dodai showcase as a WebAssembly module: the page of `topcoat_showcase` (chrome, render and
//! wire are those of the example, included by path) with the model living in the browser instead
//! of behind a server. The host (`web/showcase.js`) calls `view` and `act` where the server build
//! answers `GET /view` and `POST /act`, and feeds the clock with `set_clock`.
//!
//! The ABI is plain C: the host `alloc`s a buffer, writes the action into it, calls `act` and
//! reads the answer from `out_ptr` / `out_len`.

#[path = "../../dodai-topcoat/examples/topcoat_showcase/chrome.rs"]
mod chrome;
#[path = "../../dodai-topcoat/examples/topcoat_showcase/render.rs"]
mod render;
mod shader;
#[path = "../../dodai-topcoat/examples/topcoat_showcase/wire.rs"]
mod wire;

use dodai_showcase_model::{icon_actions::Effect, Showcase};
use std::sync::{Mutex, OnceLock};

struct State {
    model: Showcase,
    out: String,
    copy: String,
}

fn state() -> &'static Mutex<State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(|| {
        Mutex::new(State {
            model: Showcase::from_env(),
            out: String::new(),
            copy: String::new(),
        })
    })
}

fn lock() -> std::sync::MutexGuard<'static, State> {
    state()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A buffer of `len` bytes for the host to write into.
#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len);
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Frees a buffer from [`alloc`].
///
/// # Safety
/// `ptr` and `len` must be what `alloc` returned and was asked for.
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    drop(Vec::from_raw_parts(ptr, 0, len));
}

/// Gives the clock of the page: seconds since the Unix epoch and the UTC offset in minutes.
#[no_mangle]
pub extern "C" fn set_clock(secs: f64, offset_minutes: i32) {
    #[cfg(target_arch = "wasm32")]
    {
        dodai_core::clock_math::set_host_clock(secs);
        dodai_core::clock_math::set_host_utc_offset_minutes(offset_minutes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (secs, offset_minutes);
}

/// The page as it is now. Read it with `out_ptr` / `out_len`.
#[no_mangle]
pub extern "C" fn view() {
    let mut s = lock();
    s.out = chrome::app(&mut s.model);
}

/// Applies the action in `ptr[..len]` (see `wire`). `part` is 1 for the sidebar alone, 0 for the
/// whole page. Read the answer with `out_ptr` / `out_len`, a text to copy with `copy_ptr` / `copy_len`.
///
/// # Safety
/// `ptr[..len]` must be initialised bytes, for instance from `alloc`.
#[no_mangle]
pub unsafe extern "C" fn act(ptr: *const u8, len: usize, part: i32) {
    let body = String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).into_owned();
    let mut s = lock();
    wire::apply(&mut s.model, body.trim_end());
    s.copy = match s.model.take_effect() {
        Some(Effect::Copy(t)) => t,
        None => String::new(),
    };
    s.out = if part == 1 {
        chrome::side(&s.model)
    } else {
        chrome::app(&mut s.model)
    };
}

/// Start of the last answer (UTF-8).
#[no_mangle]
pub extern "C" fn out_ptr() -> *const u8 {
    lock().out.as_ptr()
}

/// Length of the last answer.
#[no_mangle]
pub extern "C" fn out_len() -> usize {
    lock().out.len()
}

/// Start of the text the last action asked to copy.
#[no_mangle]
pub extern "C" fn copy_ptr() -> *const u8 {
    lock().copy.as_ptr()
}

/// Length of that text (0 for none).
#[no_mangle]
pub extern "C" fn copy_len() -> usize {
    lock().copy.len()
}
