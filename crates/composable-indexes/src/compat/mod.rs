#[cfg(not(feature = "nostd"))]
mod std;
#[cfg(not(feature = "nostd"))]
pub use std::*;

#[cfg(feature = "nostd")]
mod nostd;
#[cfg(feature = "nostd")]
pub use nostd::*;
