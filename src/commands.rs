use crate::Error;
use indexmap::IndexMap;
use reqwest::blocking::Client as HttpClient;
use serenity::{model::channel::Message, prelude::Context};
use std::collections::HashMap;

pub const PREFIX: &str = "?";
pub type GuardFn = fn(&Args) -> Result<bool, Error>;

struct Command {
    name: String,
    guard: GuardFn,
    handler: Box<dyn Fn(Args<'_>) -> Result<(), Error> + Send + Sync>,
}

pub struct Args<'a> {
    pub http: &'a HttpClient,
    pub cx: &'a Context,
    pub msg: &'a Message,
    pub params: &'a HashMap<&'a str, &'a str>,
    pub body: &'a str,
}

pub struct Commands {
    client: HttpClient,
    menu: Option<IndexMap<&'static str, (&'static str, GuardFn)>>,
    new_commands: Vec<Command>,
}

impl Commands {
    pub fn new() -> Self {
        Self {
            client: HttpClient::new(),
            menu: Some(IndexMap::new()),
            new_commands: Vec::new(),
        }
    }

    pub fn add(
        &mut self,
        command: &'static str,
        handler: impl Fn(Args) -> Result<(), Error> + Send + Sync + 'static,
    ) {
        self.add_protected(command, handler, |_| Ok(true));
    }

    pub fn add_protected(
        &mut self,
        command: &'static str,
        handler: impl Fn(Args) -> Result<(), Error> + Send + Sync + 'static,
        guard: GuardFn,
    ) {
        self.new_commands.push(Command {
            name: command.to_owned(),
            guard,
            handler: Box::new(handler),
        });
    }

    pub fn help(
        &mut self,
        cmd: &'static str,
        desc: &'static str,
        handler: impl Fn(Args) -> Result<(), Error> + Send + Sync + 'static,
    ) {
        self.help_protected(cmd, desc, handler, |_| Ok(true));
    }

    pub fn help_protected(
        &mut self,
        cmd: &'static str,
        desc: &'static str,
        handler: impl Fn(Args) -> Result<(), Error> + Send + Sync + 'static,
        guard: GuardFn,
    ) {
        let base_cmd = &cmd[1..];
        info!("Adding command ?help {}", &base_cmd);

        self.menu.as_mut().map(|menu| {
            menu.insert(cmd, (desc, guard));
            menu
        });

        self.new_commands.push(Command {
            name: format!("?help {}", base_cmd),
            guard,
            handler: Box::new(handler),
        });
    }

    pub fn take_menu(&mut self) -> Option<IndexMap<&'static str, (&'static str, GuardFn)>> {
        self.menu.take()
    }

    pub fn execute(&self, cx: &Context, serenity_msg: &Message) {
        for command in &self.new_commands {
            // Extract "body" from something like "?command_name body"
            let msg = match serenity_msg.content.strip_prefix(&command.name) {
                Some(msg) => msg.trim(),
                None => continue,
            };

            let mut params = HashMap::new();
            let mut body = "";
            for token in msg.split_whitespace() {
                let mut splitn_2 = token.splitn(2, '=');
                if let (Some(param_name), Some(param_val)) = (splitn_2.next(), splitn_2.next()) {
                    params.insert(param_name, param_val);
                } else {
                    // If this whitespace-separated token is not a "key=value" pair, this must
                    // be the beginning of the command body. So, let's find out where we are within
                    // the msg string and set the body accordingly
                    let body_start = token.as_ptr() as usize - msg.as_ptr() as usize;
                    body = &msg[body_start..];
                    break;
                }
            }

            let args = Args {
                body,
                params: &params,
                cx: &cx,
                msg: serenity_msg,
                http: &self.client,
            };
            if let Ok(true) = (command.guard)(&args).map_err(|e| error!("{}", e)) {
                if let Err(e) = (command.handler)(args) {
                    error!("{}", e)
                }
            }
        }
    }
}
