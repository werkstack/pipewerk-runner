use crate::config::Job;
use crate::runner::{Runner, RunnerMessageIn, RunnerMessageOut};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

#[derive(Debug)]
pub struct Scheduler {
    runnerSenders: HashMap<String, Sender<RunnerMessageIn>>,
    receiver: Receiver<RunnerMessageOut>,
}

#[derive(Debug)]
struct RunnerInstance {
    runner: Runner,
}

impl Scheduler {
    pub fn new(job: &Job) -> Self {
        let (tx, rx) = mpsc::channel();
        let mut runnerSenders = HashMap::new();
        let (runner, runnerSender) = Runner::new(job, tx.clone());
        runnerSenders.insert(job.name.clone(), runnerSender);
        Self {
            receiver: rx,
            runnerSenders: runnerSenders,
        }
    }
    /*
     * WHY?
     *
    fn doit(runner: &Runner) {
        thread::spawn(move || {
            runner.start(1);
        });
    }
    */

    pub fn run(&self) {
        println!("I'm running");
        loop {
            match self.receiver.recv() {
                Ok(RunnerMessageOut::HelloFrom(runner_name)) => {
                    self.runnerSenders
                        .get(&runner_name)
                        .unwrap()
                        .send(RunnerMessageIn::Noop)
                        .unwrap();
                }
                Ok(something) => println!("{:?}", something),
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
            }
        }
        println!("done!");
    }
}
