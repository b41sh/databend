#![forbid(unsafe_code)]
//! Contains modules to interface with other formats such as [`csv`],
//! [`parquet`], [`json`], [`ipc`], [`mod@print`] and [`avro`].


pub mod ipc;
pub mod flight;

pub mod parquet;

pub mod print;

