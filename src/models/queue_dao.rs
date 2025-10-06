use super::job_dao::Job;
use crate::config::loader::Config;

#[derive(Debug)]
pub struct Queue<'a> {
    pub jobs: Vec<Job>,
    pub config: &'a Config,
}

impl Queue<'_> {
    pub fn new(config: &Config) -> Queue {
        Queue {
            jobs: Vec::new(),
            config,
        }
    }
}
