use actix::prelude::System;
use dockworker::Docker;
use pipewerk::{config, scheduler};

use std::fs;

fn main() {
    let sys = System::new("test");
    let f = fs::read_to_string("first-pipewerk.yml").unwrap();
    let config = config::Config::from_str(&f).unwrap();

    let _docker = Docker::connect_with_defaults().unwrap();
    let scheduler = scheduler::Scheduler::new(&config.jobs);
    scheduler
        .try_send(scheduler::Message::RunJobs)
        .expect("scheduler failed to start");
    println!("waiting ...");
    sys.run();
    println!("Done");
}
