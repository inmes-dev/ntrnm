/// # NTRIM Network Library
/// Client and Server implementation for NTRIM Network Protocol
/// clietn encapsulates a UniPacket, to send packets please use it.
/// After sending a packet request, your task will be pushed to the core task pool.
/// Please do not expose any then interfaces related to the net module externally.
///
pub mod packet;
pub mod sesson;
pub mod bot;

/// Only current module can access the global module.
pub(crate) mod client;
pub(crate) mod codec;
pub(crate) mod pb;