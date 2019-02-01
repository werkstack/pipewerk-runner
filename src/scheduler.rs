use crate::config::Job;
use crate::logger::ConsoleLogger;
use crate::runner;
use crate::runner::Runner;
use actix::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Scheduler {
    runner_addrs: HashMap<String, Addr<Runner>>,
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
        println!("{:?}", ctx);
        match msg {
            Message::RunJobs => self.run(),
        }
    }
}

impl Scheduler {
    pub fn new(job: &Job) -> Addr<Self> {
        let job = job.clone();
        Scheduler::create(|_ctx| {
            let logger = ConsoleLogger::new();
            let job_name = job.name.clone();
            let mut runner_addrs = HashMap::new();
            let runner_addr = Runner::new(job, logger.clone());
            runner_addrs.insert(job_name, runner_addr);
            Self {
                runner_addrs: runner_addrs,
            }
        })
    }

    pub fn run(&self) {
        for (name, addr) in &self.runner_addrs {
            addr.try_send(runner::Message::Start).unwrap();
            println!("Job has been sent to `{}`", name);
        }
    }
}
