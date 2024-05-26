mod command;
mod servlet;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use std::any::{Any, TypeId};
use std::ops::Add;
use std::collections::HashMap;
use quote::{quote, ToTokens};
use ::syn::{*,
            parse::{Parse, ParseStream, Parser},
            punctuated::Punctuated,
            spanned::Spanned,
            Result, // explicitly shadow Result
};
use log::info;
use crate::command::CommandType;


#[proc_macro_attribute]
pub fn command(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut impl_item = parse_macro_input!(item as ItemImpl);
    let args = parse_macro_input!(attrs as command::CommandsArgs);
    let impl_items = &impl_item.items;

    let function_map: HashMap<String, &ImplItemFn> = impl_items.iter().filter_map(|item| {
        match item {
            ImplItem::Fn(f) => {
                let fn_name = f.sig.ident.to_string();
                Some((fn_name, f))
            }
            _ => None
        }
    }).collect();
    let service = Ident::new(args.service.as_str(), Span::call_site());

    let output = function_map["parse"].sig.output
        .clone()
        .to_token_stream()
        .to_string();
    let output = output.split("->")
        .last()
        .unwrap()
        .trim();
    let output: Type = parse_str(output).unwrap();

    let input = function_map["generate"].sig.inputs.clone();
    let input_args = input.iter().map(|arg| {
        match arg {
            FnArg::Typed(pat) => {
                let pat = pat.pat.to_token_stream();
                quote! { #pat, }
            }
            _ => quote! {}
        }
    }).collect::<TokenStream2>();

    let cmd = args.cmd.as_str();
    let cmd_type = if args.flags.contains(CommandType::REGISTER) {
        quote! { CommandType::Register }
    } else if args.flags.contains(CommandType::SERVICE) {
        quote! { CommandType::Service }
    } else if args.flags.contains(CommandType::HEARTBEAT) {
        quote! { CommandType::Heartbeat }
    } else if args.flags.contains(CommandType::MSF) {
        quote! { CommandType::Msf }
    } else if args.flags.contains(CommandType::CMD_OPEN) {
        quote! { CommandType::CmdOpen }
    } else if args.flags.contains(CommandType::WT_LOGIN_ST) {
        quote! { CommandType::WtLoginSt }
    } else if args.flags.contains(CommandType::WT_LOGIN_SIG) {
        quote! { CommandType::WtLoginSig }
    } else {
        panic!("Invalid CommandType: {:?}", args.flags)
    };

    let impl_name = impl_item.self_ty.to_token_stream().to_string();
    let impl_name = impl_name.split("::").last().unwrap();
    let impl_name = Ident::new(impl_name, Span::call_site());

    let f: TokenStream2 = quote! {
        pub async fn #service(#input) -> Option<Receiver<#output>> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let data = match #impl_name::generate(#input_args).await {
                None => return None,
                Some(data) => data
            };
            let (seq, recv) = bot.client.send_uni_packet(UniPacket::new(
                #cmd_type,
                #cmd.to_string(),
                data
            )).await;
            let recv = match recv {
                Some(result) => result,
                None => return None
            };
            let bot = Arc::clone(bot);
            tokio::spawn(async move {
                let data = match timeout(tokio::time::Duration::from_secs(5), recv).await {
                    Ok(result) => match result {
                        Ok(result) => Some(result),
                        Err(e) => {
                            error!("Failed to receive response for Service(#service): {:?}", e);
                            None
                        }
                    },
                    Err(_) => {
                        warn!("Service(#service) timed out");
                        None
                    }
                };
                if data.is_none() {
                    bot.client.unregister_oneshot(seq).await;
                }
                let data = match data {
                    None => None,
                    Some(data) => match #impl_name::parse(&bot, data.wup_buffer.to_vec()).await {
                        None => None,
                        Some(data) => Some(data)
                    }
                };
                if tx.is_closed() { return }
                if let Err(e) = tx.send(data) {
                    error!("Failed to push response for Service(#service): {:?}", e);
                }
            });
            return Some(rx)
        }
    };
    info!("Generated command: {}", f.to_string());
    // 将f添加进item_impl
    //impl_item.items.push(ImplItem::Verbatim(f.to_string().parse().unwrap()));

    return TokenStream::from(quote! {
        use tokio::sync::oneshot::Receiver;
        use tokio::sync::oneshot::error::RecvError;
        use tokio::time::error::Elapsed;
        use tokio::time::timeout;
        use crate::client::packet::packet::{CommandType, UniPacket};
        use crate::client::packet::from_service_msg::FromServiceMsg;
        use std::sync::Arc;
        use log::{debug, error, warn};
        use crate::bot::Bot;

        #impl_item

        impl crate::bot::Bot {
            #f
        }
    });
}

#[proc_macro_attribute]
pub fn servlet(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut impl_item = parse_macro_input!(item as ItemImpl);
    let args = parse_macro_input!(attrs as servlet::ServletArgs);
    let impl_items = &impl_item.items;
    let fun_map: HashMap<String, &ImplItemFn> = impl_items.iter().filter_map(|item| {
        match item {
            ImplItem::Fn(f) => {
                let fn_name = f.sig.ident.to_string();
                Some((fn_name, f))
            }
            _ => None
        }
    }).collect();
    if fun_map.is_empty() {
        panic!("No function found in servlet");
    }
    let fun_map = fun_map.iter().filter(|(k, f)| {
        let input_args = f.sig.inputs.clone().iter().map(|arg| {
            match arg {
                FnArg::Typed(pat) => {
                    let pat = pat.ty.to_token_stream();
                    quote! { #pat, }
                }
                _ => quote! {}
            }
        }).collect::<TokenStream2>().to_string();
        let input_args = input_args.split(",").map(|s| s.to_string()).collect::<Vec<String>>();
        input_args.len() >= 2 && input_args[1].trim().ends_with("FromServiceMsg")
    }).map(|(n, f)| f.clone()).collect::<Vec<&ImplItemFn>>();

    if fun_map.is_empty() {
        panic!("No function with FromServiceMsg input found in servlet");
    }

    let impl_name = impl_item.self_ty.to_token_stream().to_string();
    let impl_name = impl_name.split("::").last().unwrap();
    let impl_name = Ident::new(impl_name, Span::call_site());

    let commands = args.cmds; // 转换为vec
    let commands = commands.iter().map(|cmd| {
        let cmd = cmd.to_string();
        quote! { #cmd.to_string(), }
    }).collect::<TokenStream2>();

    let f: TokenStream2 = quote! {
        pub async fn initialize(bot: &Arc<Bot>) {
            let servlet = Arc::new(Self(bot.clone()));
            let cmds = vec![
                #commands
            ];
            let (tx, mut rx) = tokio::sync::mpsc::channel(cmds.len());
            bot.client.register_multiple_persistent(cmds, tx).await;
            tokio::spawn(async move {
                loop {
                    if rx.is_closed() { break }
                    if let Some(from) = rx.recv().await {
                        Self::dispatch(&servlet, from).await;
                    }
                }
            });
        }
    };
    info!("Generated command: {}", f.to_string());
    impl_item.items.push(ImplItem::Verbatim(f.to_string().parse().unwrap()));

    return TokenStream::from(quote! {
        #impl_item
    });
}