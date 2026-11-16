pub mod edge;
pub mod face;
// cxx bridge to the OCCT wrapper; the file lives at src/ffi.rs next to
// src/ffi.h / src/ffi.cpp, mounted here to stay crate-internal.
#[path = "../ffi.rs"]
mod ffi;
pub mod io;
pub mod solid;

/// An exception from a bridge call that has no error of its own, as `Error::Kernel`
/// naming the call.
pub(crate) fn raised(call: &str) -> impl FnOnce(cxx::Exception) -> crate::common::error::Error + '_ {
	move |e| crate::common::error::Error::Kernel(format!("{call}: {}", e.what()))
}

/// The same exception where the public call returns no `Result`: a panic naming the call,
/// as `Solid::rotate` and the other transforms already do. It unwinds; nothing aborts.
pub(crate) fn kernel_panic<T>(call: &str, result: Result<T, cxx::Exception>) -> T {
	result.unwrap_or_else(|e| panic!("{call}: {}", e.what()))
}
