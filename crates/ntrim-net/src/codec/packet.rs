use bytes::{BufMut, BytesMut};

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
    Register,

    /// Service packet
    /// eg: get friend list
    Service,

    /// Heartbeat packet
    Heartbeat,
}

pub struct UniPacket {
    command_type: CommandType,
    command: String,
    /// not with data length
    wup_buffer: Vec<u8>,
    uin: String,
    seq: u32,
    is_login: bool,
}


impl UniPacket {
    pub fn new(command_type: CommandType, command: String, wup_buffer: Vec<u8>, uin: String, seq: u32) -> Self {
        Self {
            command_type,
            command,
            wup_buffer,
            uin: uin,
            seq: seq,
            is_login: false
        }
    }

    pub fn get_login_flag(&self) -> u32 {
        if self.is_login { 0xB } else { 0xA }
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

    /// Generate a wup buffer with data length
    pub fn to_wup_buffer(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(self.wup_buffer.len() + 4);
        buf.put_u32((self.wup_buffer.len() as u32) + 4);
        buf.put_slice(&self.wup_buffer);
        buf.to_vec()
    }
}