#![no_std]

#[cfg(feature = "tcp-client")]
pub mod tcp;
#[cfg(feature = "udp-client")]
pub mod udp;
pub mod encapsulation;
pub mod cpf;
mod common;
extern crate alloc;