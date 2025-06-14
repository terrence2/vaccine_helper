#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod schedule;

#[cfg(target_arch = "wasm32")]
mod ser_web;
#[cfg(target_arch = "wasm32")]
pub use ser_web::{create_file_picker, download_file};

#[cfg(not(target_arch = "wasm32"))]
mod ser_native;
#[cfg(not(target_arch = "wasm32"))]
pub use ser_native::{create_file_picker, download_file};

pub use app::VaccineHelperApp;
