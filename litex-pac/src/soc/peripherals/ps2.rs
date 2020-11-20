#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! PS2

use crate::{RWRegister};
use core::marker::PhantomData;

/// Receive register
pub mod RX {
    /// Received data
    pub mod data {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
    
        /// Mask (8 bit: 0xff << 0)
        pub const mask: u32 = 0xff << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }
    /// Data valid
    pub mod valid {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
    
        /// Mask (1 bit: 0x1 << 8)
        pub const mask: u32 = 0x1 << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }}

/// This register contains the current raw level of the data event trigger.  Writes
/// to this register have no effect.
pub mod EV_STATUS {
    /// Level of the ``data`` event
    pub mod data {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
    
        /// Mask (1 bit: 0x1 << 0)
        pub const mask: u32 = 0x1 << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }}

/// When a  data event occurs, the corresponding bit will be set in this register.
/// To clear the Event, set the corresponding bit in this register.
pub mod EV_PENDING {
    /// `1` if a `data` event occurred. This Event is **level triggered** when the
    /// signal is **high**.
    pub mod data {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
    
        /// Mask (1 bit: 0x1 << 0)
        pub const mask: u32 = 0x1 << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }}

/// This register enables the corresponding data events.  Write a ``0`` to this
/// register to disable individual events.
pub mod EV_ENABLE {
    /// Write a ``1`` to enable the ``data`` Event
    pub mod data {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
    
        /// Mask (1 bit: 0x1 << 0)
        pub const mask: u32 = 0x1 << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }}

pub struct RegisterBlock {
    /// Receive register
    pub RX: RWRegister<u32>,

    /// This register contains the current raw level of the data event trigger.  Writes
    /// to this register have no effect.
    pub EV_STATUS: RWRegister<u32>,

    /// When a  data event occurs, the corresponding bit will be set in this register.
    /// To clear the Event, set the corresponding bit in this register.
    pub EV_PENDING: RWRegister<u32>,

    /// This register enables the corresponding data events.  Write a ``0`` to this
    /// register to disable individual events.
    pub EV_ENABLE: RWRegister<u32>,
}

pub struct ResetValues {
    pub RX: u32,
    pub EV_STATUS: u32,
    pub EV_PENDING: u32,
    pub EV_ENABLE: u32,
}

pub struct Instance {
    pub(crate) addr: u32,
    pub(crate) _marker: PhantomData<*const RegisterBlock>,
}

impl ::core::ops::Deref for Instance {
    type Target = RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*(self.addr as *const _) }
    }
}
