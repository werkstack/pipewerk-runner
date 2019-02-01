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

impl Scheduler {
    pub fn new(job: &Job) -> Self {
        let mut runnerAddrs = HashMap::new();
        let logger = ConsoleLogger::new();
        let runnerAddr = Runner::new(job.clone(), logger.clone());
        runnerAddrs.insert(job.name.clone(), runnerAddr);
        Self {
            runnerAddrs: runnerAddrs,
        }
    }

    pub fn run(&self) {
        let sys = System::new("test");
        for (name, addr) in &self.runnerAddrs {
            addr.try_send(runner::Message::NoOp);
            println!("Job has been sent to `{}`", name);
        }
        println!("waiting ...");
        sys.run();
        println!("Done");
    }
}
