use log::warn;
use speedy::{Context, Endianness};

/// This is a speedy::Context for deserializing from a Reader while tracking
/// the number of bytes used so far.
pub trait SizeTrackingContext : Context {
    fn subtract_from_remaining(&mut self, length: usize);
    fn length_remaining(&self) -> usize;
}

impl SizeTrackingContext for Endianness {
    fn subtract_from_remaining(&mut self, _: usize) {
        warn!("this function is not implemented");
    }

    fn length_remaining(&self) -> usize {
        warn!("this function is not implemented");
        0
    }
}
