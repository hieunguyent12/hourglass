// inspired by https://github.com/mehcode/schedule-rs/tree/55873eff6c9a678e8e3857085e5dfc3992791799
// and https://github.com/mdsherry/clokwerk

use std::time::{Duration, Instant};

pub enum Time {
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
}

pub trait TimeUnits: Sized {
    fn seconds(self) -> Time;
    fn minutes(self) -> Time;
    fn hours(self) -> Time;
}

impl TimeUnits for u64 {
    fn seconds(self) -> Time {
        Time::Seconds(self)
    }

    fn minutes(self) -> Time {
        Time::Minutes(self)
    }

    fn hours(self) -> Time {
        Time::Hours(self)
    }
}

struct Job {
    interval: Option<Time>,
    cb: Box<dyn FnMut()>,
    last_tick: Instant,
}

impl Job {
    fn new<F: FnMut() + 'static>(cb: F) -> Self {
        Self {
            interval: None,
            cb: Box::new(cb),
            last_tick: Instant::now(),
        }
    }

    fn schedule(&mut self, s: Time) {
        self.interval = Some(s);
    }
}

pub struct JobScheduler<'a> {
    job_index: usize,
    scheduler: &'a mut Scheduler,
}

impl<'a> JobScheduler<'a> {
    pub fn every(&mut self, interval: Time) {
        self.scheduler.jobs[self.job_index].schedule(interval);
    }
}

#[derive(Default)]
pub struct Scheduler {
    jobs: Vec<Job>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler::default()
    }

    pub fn run<F: FnMut() + 'static>(&mut self, cb: F) -> JobScheduler {
        self.jobs.push(Job::new(cb));

        let index = self.jobs.len() - 1;

        JobScheduler {
            scheduler: self,
            job_index: index,
        }
    }

    pub fn start(&mut self) {
        if self.jobs.len() > 0 {
            for job in &mut self.jobs {
                if let Some(interval) = &job.interval {
                    let duration = match interval {
                        Time::Seconds(seconds) => seconds * 1,
                        Time::Minutes(minutes) => minutes * 60,
                        Time::Hours(hours) => hours * 60 * 60,
                    };

                    if job.last_tick.elapsed() >= Duration::from_secs(duration) {
                        (job.cb)();

                        job.last_tick = Instant::now();
                    }
                }
            }
        }
    }
}
