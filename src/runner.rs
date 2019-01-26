use crate::config::Job;
use dockworker::{
    container::AttachContainer, ContainerCreateOptions, ContainerHostConfig, ContainerLogOptions,
    CreateExecOptions, Docker, StartExecOptions,
};
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct Runner {
    job: Job,
    docker: Docker,
    scheduler: Sender<RunnerMessageOut>,
    receiver: Receiver<RunnerMessageIn>,
}

#[derive(Debug)]
pub enum OutputType {
    Stdin,
    Stdout,
}

#[derive(Debug)]
pub enum RunnerMessageOut {
    Noop,
    HelloFrom(String),
    Log {
        name: String,
        outputType: OutputType,
        output: String,
    },
}

#[derive(Debug)]
pub enum RunnerMessageIn {
    Noop,
}

impl Runner {
    pub fn new(job: &Job, scheduler: Sender<RunnerMessageOut>) -> (Self, Sender<RunnerMessageIn>) {
        let (tx, rx): (Sender<RunnerMessageIn>, Receiver<RunnerMessageIn>) = mpsc::channel();
        scheduler.send(RunnerMessageOut::Noop);
        let job = job.clone();
        let docker = Docker::connect_with_defaults().unwrap();
        let r = Runner {
            job: job,
            docker: docker,
            scheduler: scheduler,
            receiver: rx,
        };
        (r, tx.clone())
    }

    /*
     * Q: Why?
    pub fn init(&self) {
        thread::spawn(|| {
            self.start(1);
        });
    }
    */
    pub fn start(&self, howlong: u64) {
        self.scheduler
            .send(RunnerMessageOut::HelloFrom(self.job.name.clone()))
            .unwrap();

        match self.receiver.recv() {
            Ok(something) => println!("got something? {:?}", something),
            Err(e) => {
                println!("error? {:?}", e);
            }
        }
        thread::sleep(Duration::from_secs(howlong));
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
