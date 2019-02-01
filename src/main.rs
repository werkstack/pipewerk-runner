use actix::prelude::System;
use dockworker::{
    container::AttachContainer, ContainerCreateOptions, ContainerLogOptions, CreateExecOptions,
    Docker, StartExecOptions,
};
use pipewerk::{config, runner, scheduler};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use std::time::Duration;

use std::fs;

fn main() {
    let sys = System::new("test");
    let f = fs::read_to_string("first-pipewerk.yml").unwrap();
    let config = config::Config::from_str(&f).unwrap();

    let docker = Docker::connect_with_defaults().unwrap();
    let scheduler = scheduler::Scheduler::new(&config.jobs[0]);
    scheduler
        .try_send(scheduler::Message::RunJobs)
        .expect("scheduler failed to start");
    println!("waiting ...");
    sys.run();
    println!("Done");
}
