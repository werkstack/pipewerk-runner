use actix::prelude::*;

#[derive(Debug)]
pub struct ConsoleLogger {}

impl Actor for ConsoleLogger {
    type Context = Context<Self>;
}

#[derive(Debug)]
pub enum OutputType {
    Stderr,
    Stdout,
    Stdin,
}

#[derive(Debug)]
pub struct Message {
    runner_name: String,
    text: String,
    output: OutputType,
}

impl Message {
    pub fn stdout(runner_name: String, text: String) -> Self {
        Self {
            runner_name: runner_name,
            text: text,
            output: OutputType::Stdout,
        }
    }

    pub fn stderr(runner_name: String, text: String) -> Self {
        Self {
            runner_name: runner_name,
            text: text,
            output: OutputType::Stderr,
        }
    }

    pub fn stdin(runner_name: String, text: String) -> Self {
        Self {
            runner_name: runner_name,
            text: text,
            output: OutputType::Stdin,
        }
    }
}

impl actix::Message for Message {
    type Result = ();
}

impl Handler<Message> for ConsoleLogger {
    type Result = ();

    fn handle(&mut self, msg: Message, _ctx: &mut Context<Self>) {
        println!("received a log {:?}", msg)
    }
}

impl ConsoleLogger {
    pub fn new() -> Addr<Self> {
        Self::create(|ctx: &mut Context<Self>| {
            ctx.set_mailbox_capacity(1000);
            Self {}
        })
    }
}
