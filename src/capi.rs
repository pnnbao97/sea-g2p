//! C ABI for embedding sea-g2p in a non-Rust host.
//!
//! The Python wheel reaches this crate through PyO3; a C or C++ host (audio.cpp,
//! a game engine, an iOS app) has no such bridge and would otherwise have to
//! re-implement Vietnamese normalisation and G2P — a second set of rules that
//! drifts from this one the first time either side is fixed. The functions below
//! are the same two stages the Python `G2P` class exposes, nothing more.
//!
//! Built only with `--features capi`, which also turns off the PyO3 dependency,
//! so the default wheel build is unchanged:
//!
//! ```bash
//! cargo build --release --no-default-features --features capi
//! # target/release/sea_g2p.dll | libsea_g2p.so | libsea_g2p.dylib
//! ```
//!
//! Contract:
//!
//! - every returned `char *` is owned by the caller and freed with
//!   [`sea_g2p_string_free`];
//! - a handle is not thread-safe on its own; separate handles may be used from
//!   separate threads (the engine itself is read-only after construction);
//! - a failing call returns NULL and leaves a message in
//!   [`sea_g2p_last_error`], which is per-thread;
//! - strings cross the boundary as UTF-8; invalid UTF-8 in means an error, not
//!   a panic.

use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::g2p::G2PEngine;
use crate::lang::vi::Normalizer;

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_error(message: impl Into<Vec<u8>>) {
    let text = CString::new(message).unwrap_or_else(|_| CString::new("sea-g2p error").unwrap());
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(text));
}

/// The two stages, held together because a host almost always wants both and
/// they share the same dictionary file.
pub struct SeaG2p {
    normalizer: Normalizer,
    engine: G2PEngine,
}

/// The detail for the last failing call **on this thread**, or NULL if the last
/// call succeeded. Borrowed: valid until the next failing call on this thread.
#[no_mangle]
pub extern "C" fn sea_g2p_last_error() -> *const c_char {
    LAST_ERROR.with(|slot| match slot.borrow().as_ref() {
        Some(text) => text.as_ptr(),
        None => std::ptr::null(),
    })
}

/// Opens `sea_g2p.bin` and returns a handle, or NULL on failure.
///
/// # Safety
/// `dict_path` must be a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn sea_g2p_open(dict_path: *const c_char) -> *mut SeaG2p {
    if dict_path.is_null() {
        set_error("dict_path must not be null");
        return std::ptr::null_mut();
    }
    let path = match unsafe { CStr::from_ptr(dict_path) }.to_str() {
        Ok(path) => path.to_string(),
        Err(_) => {
            set_error("dict_path is not valid UTF-8");
            return std::ptr::null_mut();
        }
    };
    // A panic must not unwind into C: the host has no way to catch it and the
    // stack it would unwind through is not Rust's.
    let built = catch_unwind(AssertUnwindSafe(|| {
        let engine = G2PEngine::new(&path)?;
        Ok::<_, std::io::Error>(SeaG2p { normalizer: Normalizer::new("vi", Some(&path)), engine })
    }));
    match built {
        Ok(Ok(handle)) => Box::into_raw(Box::new(handle)),
        Ok(Err(error)) => {
            set_error(format!("cannot open {path}: {error}"));
            std::ptr::null_mut()
        }
        Err(_) => {
            set_error(format!("panicked while opening {path}"));
            std::ptr::null_mut()
        }
    }
}

/// Frees a handle. Freeing NULL is a no-op.
///
/// # Safety
/// `handle` must come from [`sea_g2p_open`] and must not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn sea_g2p_close(handle: *mut SeaG2p) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

/// Frees a string returned by this library. Freeing NULL is a no-op.
///
/// # Safety
/// `text` must come from one of the calls below and must not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn sea_g2p_string_free(text: *mut c_char) {
    if !text.is_null() {
        drop(unsafe { CString::from_raw(text) });
    }
}

/// Runs `run` over the UTF-8 string at `text` and returns an owned C string.
unsafe fn with_text(
    handle: *const SeaG2p,
    text: *const c_char,
    what: &str,
    run: impl Fn(&SeaG2p, &str) -> String,
) -> *mut c_char {
    if handle.is_null() || text.is_null() {
        set_error(format!("{what}: handle and text must not be null"));
        return std::ptr::null_mut();
    }
    let input = match unsafe { CStr::from_ptr(text) }.to_str() {
        Ok(input) => input,
        Err(_) => {
            set_error(format!("{what}: text is not valid UTF-8"));
            return std::ptr::null_mut();
        }
    };
    let handle = unsafe { &*handle };
    let output = match catch_unwind(AssertUnwindSafe(|| run(handle, input))) {
        Ok(output) => output,
        Err(_) => {
            set_error(format!("{what}: panicked"));
            return std::ptr::null_mut();
        }
    };
    // The phonemes never contain a NUL, but a caller's text might.
    match CString::new(output) {
        Ok(output) => output.into_raw(),
        Err(_) => {
            set_error(format!("{what}: result contains a NUL byte"));
            std::ptr::null_mut()
        }
    }
}

/// Vietnamese text → phonemes: normalise (numbers, dates, units, abbreviations),
/// then phonemise, resolving Vietnamese and English readings from context.
/// `punc_norm` non-zero applies the trailing-punctuation rule first.
///
/// The caller owns the result and frees it with [`sea_g2p_string_free`].
///
/// # Safety
/// `handle` must be live and `text` a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn sea_g2p_phonemize(
    handle: *const SeaG2p,
    text: *const c_char,
    punc_norm: i32,
) -> *mut c_char {
    unsafe {
        with_text(handle, text, "phonemize", |g2p, input| {
            if input.is_empty() {
                return String::new();
            }
            let normalized = g2p.normalizer.normalize(input, punc_norm != 0);
            g2p.engine.phonemize(&normalized)
        })
    }
}

/// Normalisation alone — what a chunker measures length on, because the length
/// that matters is the one after "3,5 triệu" has become words.
///
/// # Safety
/// `handle` must be live and `text` a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn sea_g2p_normalize(
    handle: *const SeaG2p,
    text: *const c_char,
    punc_norm: i32,
) -> *mut c_char {
    unsafe {
        with_text(handle, text, "normalize", |g2p, input| {
            if input.is_empty() {
                return String::new();
            }
            g2p.normalizer.normalize(input, punc_norm != 0)
        })
    }
}

/// The trailing-punctuation rule as a pure string operation, for a host that
/// settles punctuation at a chunk boundary without normalising again.
///
/// # Safety
/// `text` must be a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn sea_g2p_punc_norm(text: *const c_char) -> *mut c_char {
    if text.is_null() {
        set_error("punc_norm: text must not be null");
        return std::ptr::null_mut();
    }
    let input = match unsafe { CStr::from_ptr(text) }.to_str() {
        Ok(input) => input,
        Err(_) => {
            set_error("punc_norm: text is not valid UTF-8");
            return std::ptr::null_mut();
        }
    };
    match CString::new(crate::punc::apply_punc_norm(input)) {
        Ok(output) => output.into_raw(),
        Err(_) => {
            set_error("punc_norm: result contains a NUL byte");
            std::ptr::null_mut()
        }
    }
}

/// The ABI version, bumped when a signature changes. A host built against 1 can
/// refuse a library that reports something else rather than crash on it.
#[no_mangle]
pub extern "C" fn sea_g2p_abi_version() -> i32 {
    1
}
