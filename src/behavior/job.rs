// use std::{pin::Pin, future::Future};

// pub enum JobType {
//     Multithread(Pin<Box<dyn Future<Output = ()> + Send>>),
//     MainTHread(Box<dyn FnOnce()>),
// }

// #[derive(Default)]
// pub struct Job {
//     pub (crate) multi_thread_job: Vec<Pin<Box<dyn Future<Output = ()> + Send>>>,
//     pub (crate) main_thread_job: Vec<Box<dyn FnOnce()>>,
// }

// impl std::fmt::Debug for Job {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Job").field("multi_thread_job", &"closure collection").field("main_thread_job", &"closure collection").finish()
//     }
// }

// impl Job {
//     pub fn add_multi_thread_job(&mut self, job: Pin<Box<dyn Future<Output = ()> + Send>>) {
//         self.multi_thread_job.push(job);
//     }

//     pub fn add_main_thread_job(&mut self, job: Box<dyn FnOnce()>) {
//         self.main_thread_job.push(job);
//     }
// }