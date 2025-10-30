use super::job_dao::Job;
use crate::{config::loader::Config, models::payload_dao::Payload};

#[derive(Debug)]
pub struct Queue<'a> {
    pub jobs: Vec<Job>,
    pub config: &'a Config,
}

impl Queue<'_> {
    pub fn new(config: &Config) -> Queue<'_> {
        Queue {
            jobs: Vec::new(),
            config,
        }
    }
}

#[derive(Debug)]
pub struct PayloadQueue<'a> {
    pub jobs: Vec<Payload>,
    pub config: &'a Config,
}

impl PayloadQueue<'_> {
    pub fn new(config: &Config) -> PayloadQueue<'_> {
        PayloadQueue {
            jobs: Vec::new(),
            config,
        }
    }
}
