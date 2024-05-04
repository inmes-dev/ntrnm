use syn::LitStr;
use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
pub struct ServletArgs {
    pub(crate) cmds: Vec<String>,
}

impl Parse for ServletArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut cmds = Vec::new();
        loop {
            let command = input.parse::<LitStr>();
            let command = command.unwrap().value();
            cmds.push(command);
            if input.parse::<syn::token::Comma>().is_err() {
                break
            }
        }
        Ok(ServletArgs {
            cmds
        })
    }
}