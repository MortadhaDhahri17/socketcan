
use crate::socketcan_id::*; 


/// A CAN interface that is able to transmit and receive frames.
pub trait NbCan {
    /// Associated frame type.
    type Frame: Frame;

    /// Associated error type.
    type Error: Error;

    /// Puts a frame in the transmit buffer to be sent on the bus.
    ///
    /// If the transmit buffer is full, this function will try to replace a pending
    /// lower priority frame and return the frame that was replaced.
    /// Returns `Err(WouldBlock)` if the transmit buffer is full and no frame can be
    /// replaced.
    ///
    /// # Notes for implementers
    ///
    /// * Frames of equal identifier shall be transmitted in FIFO fashion when more
    ///   than one transmit buffer is available.
    /// * When replacing pending frames make sure the frame is not in the process of
    ///   being send to the bus.
    fn transmit(&mut self, frame: &Self::Frame) -> Result<Option<Self::Frame>, Self::Error>;

    /// Returns a received frame if available.
    fn receive(&mut self) -> Result<Self::Frame, Self::Error>;
}


/// A blocking CAN interface that is able to transmit and receive frames.
pub trait Can {
    /// Associated frame type.
    type Frame: Frame;

    /// Associated error type.
    type Error: Error;

    /// Puts a frame in the transmit buffer. Blocks until space is available in
    /// the transmit buffer.
    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error>;

    /// Blocks until a frame was received or an error occurred.
    fn receive(&mut self) -> Result<Self::Frame, Self::Error>;
}


// A CAN2.0 Frame
pub trait Frame: Sized {
    /// Creates a new frame.
    ///
    /// This will return `None` if the data slice is too long.
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self>;

    /// Creates a new remote frame (RTR bit set).
    ///
    /// This will return `None` if the data length code (DLC) is not valid.
    fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self>;

    /// Returns true if this frame is an extended frame.
    fn is_extended(&self) -> bool;

    /// Returns true if this frame is a standard frame.
    fn is_standard(&self) -> bool {
        !self.is_extended()
    }

    /// Returns true if this frame is a remote frame.
    fn is_remote_frame(&self) -> bool;

    /// Returns true if this frame is a data frame.
    fn is_data_frame(&self) -> bool {
        !self.is_remote_frame()
    }

    /// Returns the frame identifier.
    fn id(&self) -> Id;

    /// Returns the data length code (DLC) which is in the range 0..8.
    ///
    /// For data frames the DLC value always matches the length of the data.
    /// Remote frames do not carry any data, yet the DLC can be greater than 0.
    fn dlc(&self) -> usize;

    /// Returns the frame data (0..8 bytes in length).
    fn data(&self) -> &[u8];
}



/// Serial error.
pub trait Error: core::fmt::Debug {
    /// Convert error to a generic serial error kind
    ///
    /// By using this method, serial errors freely defined by HAL implementations
    /// can be converted to a set of generic serial errors upon which generic
    /// code can act.
    fn kind(&self) -> ErrorKind;
}

impl Error for core::convert::Infallible {
    #[inline]
    fn kind(&self) -> ErrorKind {
        match *self {}
    }
}

/// Serial error kind.
///
/// This represents a common set of serial operation errors. HAL implementations are
/// free to define more specific or additional error types. However, by providing
/// a mapping to these common serial errors, generic code can still react to them.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The peripheral receive buffer was overrun.
    Overrun,
    /// Received data does not conform to the peripheral configuration.
    /// Can be caused by a misconfigured device on either end of the serial line.
    FrameFormat,
    /// Parity check failed.
    Parity,
    /// Serial line is too noisy to read valid data.
    Noise,
    /// A different error occurred. The original error may contain more information.
    Other,
}

impl Error for ErrorKind {
    #[inline]
    fn kind(&self) -> ErrorKind {
        *self
    }
}

impl core::fmt::Display for ErrorKind {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Overrun => write!(f, "The peripheral receive buffer was overrun"),
            Self::Parity => write!(f, "Parity check failed"),
            Self::Noise => write!(f, "Serial line is too noisy to read valid data"),
            Self::FrameFormat => write!(
                f,
                "Received data does not conform to the peripheral configuration"
            ),
            Self::Other => write!(
                f,
                "A different error occurred. The original error may contain more information"
            ),
        }
    }
}
