use std::sync::Arc;
use bytes::{BufMut, BytesMut};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CommandType {
    /// Msf Packet
    Msf,

    /// Cmd Open
    CmdOpen,

    /// Wtlogin packet
    /// build for login
    /// build for refresh st or sig
    WtLoginSt,
    /// Sig == Web Cookie
    WtLoginSig,

    /// Cmd Register
    /// request to go live!!!
    Register,

    /// Service packet
    /// eg: get friend list
    Service,

    /// Heartbeat packet
    Heartbeat,
}

#[derive(Debug)]
pub struct UniPacket {
    pub command_type: CommandType,
    pub command: String,
    /// not with data length
    pub wup_buffer: Arc<Vec<u8>>,
}

impl UniPacket {
    pub fn new_service(
        command: String,
        wup_buffer: Vec<u8>,
    ) -> Self {
        Self {
            command_type: CommandType::Service,
            command,
            wup_buffer: Arc::new(wup_buffer)
        }
    }

    pub fn new(
        command_type: CommandType,
        command: String,
        wup_buffer: Vec<u8>,
    ) -> Self {
        Self {
            command_type, command,
            wup_buffer: Arc::new(wup_buffer)
        }
    }

    /// 0x0 no encrypt
    /// 0x1 encrypt by d2key
    /// 0x2 encrypt by default key
    pub fn get_encrypted_flag(&self) -> u8 {
        match self.command_type {
            CommandType::Msf => 0x0,
            CommandType::CmdOpen => 0x0,
            CommandType::WtLoginSt => 0x2,
            CommandType::WtLoginSig => 0x2,
            CommandType::Register => 0x1,
            CommandType::Service => 0x1,
            CommandType::Heartbeat => 0x0
        }
    }

    pub fn get_head_flag(&self) -> u32 {
        match self.command_type {
            CommandType::Msf => 0x1335239,
            CommandType::CmdOpen => 0x1335239,
            CommandType::WtLoginSt => 0xA,
            CommandType::WtLoginSig => 0xA,
            CommandType::Register => 0xA,
            CommandType::Service => 0xB,
            CommandType::Heartbeat => 0xB
        }
    }

    /// Generate a wup buffer with data length
    pub fn to_wup_buffer(&self) -> Arc<Vec<u8>> {
        Arc::clone(&self.wup_buffer)
    }
}