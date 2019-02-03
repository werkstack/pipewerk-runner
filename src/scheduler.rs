use crate::config::Job;
use crate::logger::ConsoleLogger;
use crate::runner;
use crate::runner::Runner;
use actix::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
struct RunnerMeta {
    job_runner: Addr<Runner>,
    exit_code: Option<u32>,
    is_running: bool,
}

#[derive(Debug)]
pub struct Scheduler {
    jobs_meta: HashMap<String, RunnerMeta>,
}

impl Actor for Scheduler {
    type Context = Context<Self>;
}

#[derive(Debug)]
pub enum Message {
    RunJobs(Addr<Scheduler>),
    JobStarted(String),
    JobFinished(String, u32),
}

impl actix::Message for Message {
    type Result = ();
}

impl Handler<Message> for Scheduler {
    type Result = ();

    fn handle(&mut self, msg: Message, _ctx: &mut Context<Self>) {
        match msg {
            Message::RunJobs(scheduler) => self.run(scheduler),
            Message::JobFinished(job_name, exit_code) => self.job_exited(job_name, exit_code),
            Message::JobStarted(job_name) => self.update_running_status(job_name, true),
        }
    }
}

impl Scheduler {
    pub fn new(jobs: &Vec<Job>) -> Addr<Self> {
        let cloned_jobs: Vec<Job> = jobs.iter().map(|j| j.clone()).collect();
        Scheduler::create(|_ctx| {
            let logger = ConsoleLogger::new();
            let mut jobs_meta = HashMap::new();
            for job in cloned_jobs {
                let job_name = job.name.clone();
                let job_runner = Runner::new(job, logger.clone());
                let runner_meta = RunnerMeta {
                    job_runner: job_runner,
                    exit_code: None,
                    is_running: false,
                };
                jobs_meta.insert(job_name, runner_meta);
            }
            Self {
                jobs_meta: jobs_meta,
            }
        })
    }

    fn run(&self, scheduler: Addr<Scheduler>) {
        for (name, meta) in &self.jobs_meta {
            meta.job_runner
                .try_send(runner::Message::Start(scheduler.clone()))
                .unwrap();
            println!("Job has been sent to `{}`", name);
        }
    }

    fn job_exited(&mut self, job_name: String, exit_code: u32) {
        self.update_exit_code(job_name, exit_code);
        if self.is_any_running_job() == false {
            System::current().stop();
        }
    }

    fn update_running_status(&mut self, job_name: String, status: bool) {
        //TODO: use map
        match self.jobs_meta.get_mut(&job_name) {
            Some(job_meta) => {
                job_meta.is_running = status;
            }
            _ => (),
        }
    }
    fn update_exit_code(&mut self, job_name: String, exit_code: u32) {
        //TODO: use map
        match self.jobs_meta.get_mut(&job_name) {
            Some(job_meta) => {
                job_meta.is_running = false;
                job_meta.exit_code = Some(exit_code);
            }
            _ => (),
        }
    }

    fn is_any_running_job(&self) -> bool {
        self.jobs_meta
            .iter()
            .fold(true, |result, (_, meta)| meta.is_running && result)
    }
}
