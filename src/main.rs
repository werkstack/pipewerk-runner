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

    /*
    let runners: Vec<runner::Runner> = config
        .jobs
        .into_iter()
        .map(|job| runner::Runner::new(&job))
        .collect();

    for runner in runners {
        runner.run();
    }
    */
    //println!("{:#?}", docker.system_info().unwrap());
    /*
    let mut create = ContainerCreateOptions::new("ubuntu");
    create.tty(true).stop_timeout(Duration::from_secs(10));
    create
        .entrypoint(vec!["sleep".into()])
        .cmd("20".to_string())
        .working_dir(PathBuf::from("/opt"));

    let container = docker.create_container(None, &create).unwrap();
    docker.start_container(&container.id).unwrap();

    let mut exec_config = CreateExecOptions::new();
    exec_config.cmd("pwd".to_string()).cmd("-m".to_string());
    let exec = docker
        .container_create_exec_instance(&container.id, &exec_config)
        .unwrap();
    let exec_start_config = StartExecOptions::new();
    let res = docker.start_exec(&exec.id, &exec_start_config).unwrap();

    let cont: AttachContainer = res.into();
    let mut line_reader = BufReader::new(cont.stderr);

    loop {
        let mut line = String::new();
        let size = line_reader.read_line(&mut line).unwrap();
        print!("{:4}: {}", size, line);
        if size == 0 {
            break;
        }
    }
    println!("");
    */
}
