use chrono::{offset, DateTime, Duration, Utc};
use cron::Schedule;

pub struct Job<F> {
    schedule: Schedule,
    run: F,
    last_tick: Option<DateTime<Utc>>,
    limit_missed_runs: usize,
}

impl<F> Job<F> {
    pub fn new(schedule: Schedule, run: F) -> Job<F> {
        Job {
            schedule,
            run,
            last_tick: None,
            limit_missed_runs: 1,
        }
    }
}

impl<F, C> Job<F>
where
    F: Fn() -> C,
    C: std::future::Future,
{
    async fn async_tick(&mut self) {
        let now = Utc::now();
        if self.last_tick.is_none() {
            self.last_tick = Some(now);
            return;
        }

        if self.limit_missed_runs > 0 {
            for event in self
                .schedule
                .after(&self.last_tick.unwrap())
                .take(self.limit_missed_runs)
            {
                if event > now {
                    break;
                }
                (self.run)().await;
            }
        } else {
            for event in self.schedule.after(&self.last_tick.unwrap()) {
                if event > now {
                    break;
                }
                (self.run)().await;
            }
        }
        self.last_tick = Some(now);
    }
}

impl<F> Job<F>
where
    F: FnMut(),
{
    fn tick(&mut self) {
        let now = Utc::now();
        if self.last_tick.is_none() {
            self.last_tick = Some(now);
            return;
        }

        if self.limit_missed_runs > 0 {
            for event in self
                .schedule
                .after(&self.last_tick.unwrap())
                .take(self.limit_missed_runs)
            {
                if event > now {
                    break;
                }
                (self.run)();
            }
        } else {
            for event in self.schedule.after(&self.last_tick.unwrap()) {
                if event > now {
                    break;
                }
                (self.run)();
            }
        }
        self.last_tick = Some(now);
    }
}

pub struct JobScheduler<F> {
    jobs: Vec<Job<F>>,
}

impl<F> JobScheduler<F> {
    pub fn new() -> JobScheduler<F> {
        JobScheduler { jobs: Vec::new() }
    }

    pub fn add(&mut self, job: Job<F>) {
        self.jobs.push(job);
    }

    pub fn time_till_next_job(&self) -> std::time::Duration {
        if self.jobs.is_empty() {
            return std::time::Duration::from_millis(500);
        }
        let mut duration = Duration::zero();
        let now = Utc::now();
        for job in self.jobs.iter() {
            for event in job.schedule.upcoming(offset::Utc).take(1) {
                let d = event - now;
                if duration.is_zero() || d < duration {
                    duration = d;
                }
            }
        }
        duration.to_std().unwrap()
    }
}

impl<F> Default for JobScheduler<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F, C> JobScheduler<F>
where
    F: Fn() -> C,
    C: std::future::Future,
{
    pub async fn async_tick(&mut self) {
        for job in &mut self.jobs {
            job.async_tick().await;
        }
    }
}

impl<F> JobScheduler<F>
where
    F: FnMut(),
{
    pub fn tick(&mut self) {
        for job in &mut self.jobs {
            job.tick();
        }
    }
}
