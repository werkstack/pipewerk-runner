use crate::config::Job;
use crate::logger::ConsoleLogger;
use crate::logger::Message as MessageLogger;
use dockworker::{
    container::AttachContainer, ContainerCreateOptions, ContainerHostConfig, ContainerLogOptions,
    CreateExecOptions, Docker, StartExecOptions,
};
use std::io::{BufRead, BufReader};
use std::time::Duration;

use actix::prelude::*;

#[derive(Debug)]
pub struct Runner {
    job: Job,
    docker: Docker,
    logger: Addr<ConsoleLogger>,
}

impl Actor for Runner {
    type Context = Context<Self>;
}

#[derive(Debug)]
pub enum Message {
    Start,
    NoOp,
}

impl actix::Message for Message {
    type Result = ();
}

impl Handler<Message> for Runner {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Context<Self>) {
        println!("Arrived `{:?}` MSG", msg);
        self.logger
            .try_send(MessageLogger::stdout(
                self.job.name.clone(),
                "hi! ".to_owned(),
            ))
            .unwrap();
        ctx.run_later(Duration::new(5, 100), move |act, _| {
            System::current().stop()
        });
    }
}
impl Runner {
    pub fn new(job: &Job, logger: Addr<ConsoleLogger>) -> Addr<Self> {
        let job = job.clone();
        Runner::create(|ctx| {
            let docker = Docker::connect_with_defaults().unwrap();
            Runner {
                job: job,
                docker: docker,
                logger: logger,
            }
        })
    }

    pub fn run(&self) {
        let mut host_config = ContainerHostConfig::new();
        host_config.binds("/Users/milad/dev/pipewerk-runner:/opt/app".to_owned());

        let mut create = ContainerCreateOptions::new("test-iostream");
        create.tty(true).stop_timeout(Duration::from_secs(10));
        create
            .host_config(host_config)
            .entrypoint(vec!["sleep".into()])
            .cmd("20".to_string());

        let container = self.docker.create_container(None, &create).unwrap();
        self.docker.start_container(&container.id).unwrap();

        let mut exec_config = CreateExecOptions::new();
        exec_config.cmd("ls".to_string()).cmd("/opt/app".to_owned());
        //exec_config
        //    .cmd("./entrypoint.sh".to_string())
        //    .cmd("./sample/apache-2.0.txt".to_string())
        //    .cmd("./sample/bsd4.txt".to_string());
        let exec = self
            .docker
            .container_create_exec_instance(&container.id, &exec_config)
            .unwrap();
        let exec_start_config = StartExecOptions::new();
        let res = self
            .docker
            .start_exec(&exec.id, &exec_start_config)
            .unwrap();
        let cont: AttachContainer = res.into();
        let mut stdout_reader = BufReader::new(cont.stdout);
        let mut stderr_reader = BufReader::new(cont.stderr);
        loop {
            let mut stdout_line = String::new();
            let mut stderr_line = String::new();
            let stdout_size = stdout_reader.read_line(&mut stdout_line).unwrap();
            let stderr_size = stderr_reader.read_line(&mut stderr_line).unwrap();
            println!("stdout {:4}: {}", stdout_size, stdout_line);
            println!("stderr {:4}: {}", stderr_size, stderr_line);
            let exec_info = self.docker.exec_info(&exec.id).unwrap();
            if exec_info.Running == false && stderr_size == 0 && stdout_size == 0 {
                break;
            }
        }
    }
}
