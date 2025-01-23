use super::job_dao::Job;

#[derive(Debug)]
pub struct Queue {
    pub jobs: Vec<Job>,
}

impl Queue {
    pub fn new() -> Queue {
        Queue { jobs: Vec::new() }
    }
}
