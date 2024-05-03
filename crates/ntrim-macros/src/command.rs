use bitflags::bitflags;
use proc_macro2::Ident;
use syn::LitStr;
use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
pub struct CommandsArgs {
    pub(crate) cmd: String,
    pub(crate) service: String,
    pub(crate) flags: CommandType,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq)]
    pub struct CommandType: u32 {
        const PROTOBUF =     0b00000001;
        const REGISTER =     0b00000010;
        const SERVICE =      0b00000100;
        const HEARTBEAT =    0b00001000;
        const MSF =          0b00010000;
        const CMD_OPEN =     0b00100000;
        const WT_LOGIN_ST =  0b01000000;
        const WT_LOGIN_SIG = 0b10000000;
    }
}

impl Parse for CommandsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let command = input.parse::<LitStr>()?.value();
        input.parse::<syn::token::Comma>().expect("Expected comma");
        let service = input.parse::<LitStr>()?.value();
        let mut flags = vec![];
        while input.parse::<syn::token::Comma>().is_ok() {
            if let Ok(ident) = input.parse::<Ident>() {
                flags.push(ident.to_string());
            } else {
                break;
            }
        }
        let mut command_type = CommandType::empty();
        for flag in flags {
            match flag.to_lowercase().as_str() {
                "protobuf" => command_type.set(CommandType::PROTOBUF, true),
                "register" => command_type.set(CommandType::REGISTER, true),
                "service" => command_type.set(CommandType::SERVICE, true),
                "heartbeat" => command_type.set(CommandType::HEARTBEAT, true),
                "msf" => command_type.set(CommandType::MSF, true),
                "cmd_open" => command_type.set(CommandType::CMD_OPEN, true),
                "wt_login_st" => command_type.set(CommandType::WT_LOGIN_ST, true),
                "wt_login_sig" => command_type.set(CommandType::WT_LOGIN_SIG, true),
                _ => panic!("Unknown flag: {}", flag)
            }
        }
        Ok(CommandsArgs {
            cmd: command,
            service,
            flags: command_type,
        })
    }
}