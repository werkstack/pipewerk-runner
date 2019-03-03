use crate::config::Job;
use crate::logger::ConsoleLogger;
use crate::logger::Message as MessageLogger;
use crate::scheduler::{Message as SchedulerMessage, Scheduler};
use dockworker::{
    container::AttachContainer, ContainerCreateOptions, ContainerHostConfig,
    CreateContainerResponse, CreateExecOptions, CreateExecResponse, Docker, StartExecOptions,
};
use std::env;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use actix::prelude::*;

#[derive(Debug)]
pub struct Runner {
    job: Job,
    docker: Docker,
    logger: Addr<ConsoleLogger>,
    scheduler: Option<Addr<Scheduler>>,
}

impl Actor for Runner {
    type Context = Context<Self>;
}

#[derive(Debug)]
pub enum Message {
    Start(Addr<crate::scheduler::Scheduler>),
    NoOp,
}

impl actix::Message for Message {
    type Result = ();
}

impl Handler<Message> for Runner {
    type Result = ();

    fn handle(&mut self, msg: Message, _ctx: &mut Context<Self>) {
        match msg {
            Message::Start(scheduler) => {
                scheduler
                    .try_send(SchedulerMessage::JobStarted(self.job.name.to_owned()))
                    .unwrap();

                self.scheduler = Some(scheduler);

                self.run();
            }
            other => {
                println!("Ignore `{:?}` MSG", other);
            }
        }
    }
}
impl Runner {
    pub fn new(job: Job, logger: Addr<ConsoleLogger>) -> Self {
        let docker = Docker::connect_with_defaults().unwrap();
        Runner {
            job: job,
            docker: docker,
            logger: logger,
            scheduler: None,
        }
    }

    pub fn run(&self) {
        let mut create = ContainerCreateOptions::new(&self.job.image.to_owned());
        create
            .tty(true)
            .stop_timeout(Duration::from_secs(10))
            .working_dir(PathBuf::from("/opt/app"))
            .host_config(self.host_config());

        let container = self.docker.create_container(None, &create).unwrap();
        self.docker.start_container(&container.id).unwrap();

        let exit_code: Option<u32> =
            self.job
                .commands
                .iter()
                .fold(Some(0), |result, command| match result {
                    Some(0) => self.exec(&container, command.to_owned()),
                    other_value => other_value,
                });
        //TODO: use Option.map
        match &self.scheduler {
            Some(scheduler) => {
                scheduler
                    .try_send(SchedulerMessage::JobFinished(
                        self.job.name.to_owned(),
                        exit_code.unwrap(),
                    ))
                    .unwrap();
            }
            _ => (),
        };
        self.docker
            .stop_container(&container.id, Duration::from_secs(1))
            .unwrap();
    }

    fn exec(&self, container: &CreateContainerResponse, command: String) -> Option<u32> {
        self.logger
            .try_send(MessageLogger::stdin(
                self.job.name.clone(),
                command.to_owned(),
            ))
            .unwrap();

        let mut exec_config = CreateExecOptions::new();
        exec_config
            .cmd("sh".to_owned())
            .cmd("-c".to_owned())
            .cmd(command);
        let exec = self
            .docker
            .container_create_exec_instance(&container.id, &exec_config)
            .unwrap();
        let exec_start_config = StartExecOptions::new();
        let res = self
            .docker
            .start_exec(&exec.id, &exec_start_config)
            .unwrap();
        let attached_container: AttachContainer = res.into();
        self.capture_stdio(attached_container, exec)
    }

    fn capture_stdio(
        &self,
        attached_container: AttachContainer,
        exec: CreateExecResponse,
    ) -> Option<u32> {
        let mut stdout_reader = BufReader::new(attached_container.stdout);
        let mut stderr_reader = BufReader::new(attached_container.stderr);
        loop {
            let mut stdout_line = String::new();
            let mut stderr_line = String::new();
            let stdout_size = stdout_reader.read_line(&mut stdout_line).unwrap();
            let stderr_size = stderr_reader.read_line(&mut stderr_line).unwrap();
            if stdout_size > 0 {
                self.logger
                    .try_send(MessageLogger::stdout(
                        self.job.name.clone(),
                        stdout_line.to_owned(),
                    ))
                    .unwrap();
            }

            if stderr_size > 0 {
                self.logger
                    .try_send(MessageLogger::stderr(
                        self.job.name.clone(),
                        stderr_line.to_owned(),
                    ))
                    .unwrap();
            }

            let exec_info = self.docker.exec_info(&exec.id).unwrap();
            if exec_info.Running == false && stderr_size == 0 && stdout_size == 0 {
                //System::current().stop();
                return exec_info.ExitCode;
            }
        }
    }

    fn host_config(&self) -> ContainerHostConfig {
        let mut host_config = ContainerHostConfig::new();
        host_config
            .binds(format!("{}:/opt/app", Self::current_dir()))
            .auto_remove(true);
        //TODO: improve
        for ssh_key in &self.job.ssh_keys {
            if let Some(file_name) = Path::new(ssh_key).file_name() {
                if let Ok(basename) = file_name.to_os_string().into_string() {
                    host_config.binds(format!("{}:/root/.ssh/{}", ssh_key, basename));
                }
            }
        }
        host_config
    }

    fn current_dir() -> String {
        env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
    }
}
