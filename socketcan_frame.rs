
// Implements frames for CANbus 2.0 and FD for SocketCAN on Linux.
//
// This file is part of the Rust 'socketcan-rs' library.
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.

//! CAN bus frames.
//!
//! At the lowest level, [libc](https://crates.io/crates/libc) defines the
//! CAN frames as low-level structs that are binary compatible with the C
//! data types sent to and from the kernel:
//! - [can_frame](https://docs.rs/libc/latest/libc/struct.can_frame.html)
//! The Classic CAN 2.0 frame with up to 8 bytes of data.
//! - [canfd_frame](https://docs.rs/libc/latest/libc/struct.canfd_frame.html)
//! The CAN Flexible Data Rate frame with up to 64 bytes of data.
//!
//! The classic frame represents three possibilities:
//! - `CanDataFrame` - A standard CAN frame that can contain up to 8 bytes of
//! data.
//! - `CanRemoteFrame` - A CAN Remote frame which is meant to request a
//! transmission by another node on the bus. It contain no data.
//! - `CanErrorFrame` - This is an incoming (only) frame that contains
//! information about a problem on the bus or in the driver. Error frames
//! can not be sent to the bus, but can be converted to standard Rust
//! [Error](https://doc.rust-lang.org/core/error/trait.Error.html) types.
//!




/// Different imports : 

use crate::socketcan_id::*; 
use crate::socketcan_embedded::*; 

/// Constants To be rechecked with libc
pub const _CANFD_BRS: u32 = 1;
pub const _CANFD_ESI: u32 = 2;
pub const _CANFD_MAX_DLEN: u32 = 64;
pub const _CAN_EFF_FLAG: u32 = 2147483648;
pub const _CAN_RTR_FLAG: u32 = 1073741824;
pub const _CAN_ERR_FLAG: u32 = 536870912;
pub const _CAN_MAX_DLEN: u32 = 8;
pub const _CAN_SFF_MASK: u32 = 2047;
pub const _CAN_ERR_MASK: u32 = 536870911;
pub const _CAN_EFF_MASK: u32 = 536870911;




///Equivalent for bitflags! macro : 
///IdFlags struct !

pub struct IdFlags {
    pub can_id: u32,
}

impl IdFlags {
    pub fn new(can_id: u32) -> Self {
        Self { can_id }
    }
    pub fn is_extended(&self) -> bool {
        (self.can_id & _CAN_EFF_FLAG) != 0
    }
    pub fn is_remote(&self) -> bool {
        (self.can_id & _CAN_RTR_FLAG) != 0
    }
    pub fn is_error(&self) -> bool {
        (self.can_id & _CAN_ERR_FLAG) != 0
    }
}



///Equivalent for bitflags! macro : 
///Fdlags struct !
pub struct FdFlags {
    pub flags: u32,
}
impl FdFlags {
    pub fn new(flags: u32) -> Self {
        Self { flags }
    }
    pub fn is_brs(&self) -> bool {
        (self.flags & _CANFD_BRS) != 0
    }
    pub fn is_esi(&self) -> bool {
        (self.flags & _CANFD_ESI) != 0
    }
}


/// This struct defines some of the fields of the can_frame
/// Could look at a potential bindgen use 
#[repr(C, align(8))]
pub struct can_frame {
    pub can_id: canid_t,
    pub can_dlc: u8,
    pub data: [u8; 8],
    /* private fields */
}

pub type canid_t = u32;

/// This struct defines some of the fields of the canfd_frame
/// Could look at a potential bindgen use 
pub struct canfd_frame {
    pub can_id: canid_t,
    pub len: u8,
    pub flags: u8,
    pub data: [u8; 64],
    /* private fields */
}

/// An error mask that will cause SocketCAN to report all errors
pub const _ERR_MASK_ALL: u32 = _CAN_ERR_MASK;
/// An error mask that will cause SocketCAN to silently drop all errors
pub const _ERR_MASK_NONE: u32 = 0;


/// Gets the canid_t value from an Id
/// If it's an extended ID, the CAN_EFF_FLAG bit is also set.
pub fn id_to_canid_t(id: impl Into<Id>) -> canid_t {
    let id = id.into();
    match id {
        Id::Standard(id) => id.as_raw() as canid_t,
        Id::Extended(id) => id.as_raw() | _CAN_EFF_FLAG,
    }
}

/// Determines if the ID is a 29-bit extended ID.
pub fn id_is_extended(id: &Id) -> bool {
    matches!(id, Id::Extended(_))
}

/// Creates a CAN ID from a raw integer value.
///
/// If the `id` is <= 0x7FF, it's assumed to be a standard ID, otherwise
/// it is created as an Extened ID. If you require an Extended ID <= 0x7FF,
/// create it explicitly.
pub fn id_from_raw(id: u32) -> Option<Id> {
    let id = match id {
        n if n <= _CAN_SFF_MASK => StandardId::new(n as u16)?.into(),
        n => ExtendedId::new(n)?.into(),
    };
    Some(id)
}

// ===== can_frame =====

/// Creates a default C `can_frame`.
/// This initializes the entire structure to zeros.
#[inline(always)]
pub fn can_frame_default() -> can_frame {
    unsafe {core::mem::zeroed() }
}

/// Creates a default C `can_frame`.
/// This initializes the entire structure to zeros.
#[inline(always)]
pub fn canfd_frame_default() -> canfd_frame {
    unsafe {core::mem::zeroed() }
}

// ===== AsPtr trait =====

/// Trait to get a pointer to an inner type
pub trait AsPtr {
    /// The inner type to which we resolve as a pointer
    type Inner;

    /// Gets a const pointer to the inner type
    fn as_ptr(&self) -> *const Self::Inner;

    /// Gets a mutable pointer to the inner type
    fn as_mut_ptr(&mut self) -> *mut Self::Inner;

    /// The size of the inner type
    fn size(&self) -> usize {
        core::mem::size_of::<Self::Inner>()
    }

    /// Gets a byte slice to the inner type
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts::<'_, u8>(
                self.as_ptr() as *const _ as *const u8,
                self.size(),
            )
        }
    }

    /// Gets a mutable byte slice to the inner type
    fn as_bytes_mut(&mut self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts::<'_, u8>(
                self.as_mut_ptr() as *mut _ as *mut u8,
                self.size(),
            )
        }
    }
}


// ===== Frame trait =====

/// Shared trait for CAN frames
#[allow(clippy::len_without_is_empty)]
pub trait Frame: crate::socketcan_embedded::Frame {
    /// Creates a frame using a raw, integer CAN ID.
    ///
    /// If the `id` is <= 0x7FF, it's assumed to be a standard ID, otherwise
    /// it is created as an Extened ID. If you require an Etended ID <= 0x7FF,
    /// use `new()`.
    fn from_raw_id(id: u32, data: &[u8]) -> Option<Self>{
        Self::new(id_from_raw(id)?, data)
    }

    /// Creates a remote frame using a raw, integer CAN ID.
    ///
    /// If the `id` is <= 0x7FF, it's assumed to be a standard ID, otherwise
    /// it is created as an Extened ID. If you require an Etended ID <= 0x7FF,
    /// use `new_remote()`.
    fn remote_from_raw_id(id: u32, dlc: usize) -> Option<Self>  {
        Self::new_remote(id_from_raw(id)?, dlc)
    }

    /// Get the composite SocketCAN ID word, with EFF/RTR/ERR flags
    fn id_word(&self) -> canid_t;

    /// Return the actual raw CAN ID (without EFF/RTR/ERR flags)
    fn raw_id(&self) -> canid_t {
        let mask = if self.is_extended() {
            _CAN_EFF_MASK
        } else {
            _CAN_SFF_MASK
        };
        self.id_word() & mask
    }

    /*  Returns the EFF/RTR/ERR flags from the ID word
    ///fn id_flags(&self) -> IdFlags {
       /// IdFlags::from_bits_truncate(self.id_word())
    }*/

    /// Get the data length
    fn len(&self) -> usize {
        self.dlc()
    }

    /*  Check if frame is an error message
    fn is_error_frame(&self) -> bool {
        self.id_flags().contains(IdFlags::ERR)
    }*/

    /// Sets the CAN ID for the frame
    fn set_id(&mut self, id: impl Into<Id>);

}


// ===== CanAnyFrame =====

/// An FD socket can read a raw classic 2.0 or FD frame.
#[allow(missing_debug_implementations)]
pub enum CanRawFrame {
    /// A classic CAN 2.0 frame, with up to 8-bytes of data
    Classic(can_frame),
    /// A flexible data rate frame, with up to 64-bytes of data
    Fd(canfd_frame),
}

impl From<can_frame> for CanRawFrame {
    fn from(frame: can_frame) -> Self {
        Self::Classic(frame)
    }
}

impl From<canfd_frame> for CanRawFrame {
    fn from(frame: canfd_frame) -> Self {
        Self::Fd(frame)
    }
}

