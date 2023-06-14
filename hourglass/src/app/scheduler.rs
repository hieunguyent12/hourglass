/*
 let age = 5 seconds;
 let scheduler = Scheduler::new();

 scheduler::add(|| {
   println!("job")
 }, age);
*/

struct Job {
    age: String,
    cb: Box<dyn FnMut()>,
}

#[derive(Default)]
pub struct Scheduler {
    jobs: Vec<Job>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler::default()
    }

    pub fn add(&mut self, cb: Box<dyn FnMut()>, age: String) {
        self.jobs.push(Job { age, cb });
    }
}
