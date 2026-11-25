//! Absys, the assertional language of J. M. Foster and E. W. Elcock
//! (University of Aberdeen, 1967-68), reconstructed from what was published
//! about it. A text is a conjunction of assertions; the machine constructs
//! data to satisfy them, elaborating every alternative fairly, and keeps what
//! it cannot yet decide suspended until an assertion added later decides it.

pub mod compile;
pub mod json;
pub mod lsp;
pub mod machine;
pub mod play;
pub mod report;
pub mod research;
pub mod session;
pub mod syntax;
pub mod syntax1969;
