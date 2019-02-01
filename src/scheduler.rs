use crate::config::Job;
use crate::logger::ConsoleLogger;
use crate::runner;
use crate::runner::Runner;
use actix::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Scheduler {
    runnerAddrs: HashMap<String, Addr<Runner>>,
}

impl Actor for Scheduler {
    type Context = Context<Self>;
}

#[derive(Debug)]
pub enum Message {
    RunJobs,
}

impl actix::Message for Message {
    type Result = ();
}

impl Handler<Message> for Scheduler {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Context<Self>) {
        match msg {
            RunJobs => self.run(),
        }
    }
}

impl Scheduler {
    pub fn new(job: &Job) -> Addr<Self> {
        let job = job.clone();
        Scheduler::create(|ctx| {
            let logger = ConsoleLogger::new();
            let job_name = job.name.clone();
            let mut runnerAddrs = HashMap::new();
            let runnerAddr = Runner::new(job, logger.clone());
            runnerAddrs.insert(job_name, runnerAddr);
            Self {
                runnerAddrs: runnerAddrs,
            }
        })
    }

    pub fn run(&self) {
        for (name, addr) in &self.runnerAddrs {
            addr.try_send(runner::Message::NoOp);
            println!("Job has been sent to `{}`", name);
        }
    }
}
