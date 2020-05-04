use crate::tasks::{Task, TaskState};
use cortex_m::asm::bkpt;

type TaskList<'a> = [&'a dyn Task; 3];

pub enum TaskID {
    Current,
    Next,
    ID(usize),
}

pub struct TaskMan<'a> {
    tasks: TaskList<'a>,
    times: [i32; 3],
    ticks: u32,
}

impl<'a> TaskMan<'a> {
    pub fn new(tasks: TaskList<'a>, ticks: u32) -> Self {
        let times = [
            (tasks[0].get_prd() * ticks) as i32,
            (tasks[1].get_prd() * ticks) as i32,
            (tasks[2].get_prd() * ticks) as i32,
        ];
        Self {
            tasks,
            times,
            ticks,
        }
    }

    pub fn update_release(&mut self, elapsed: u32) {
        for (time, task) in self.times.iter_mut().zip(self.tasks.iter_mut()) {
            *time -= elapsed as i32;
            if *time <= 0 {
                if task.get_state() == TaskState::Done {
                    *time = (task.get_prd() * self.ticks) as i32;
                    task.set_state(TaskState::Ready);
                }
            }
        }
    }

    pub fn next_release_time(&self) -> u32 {
        self.times.iter().copied().min().unwrap() as u32
    }

    pub fn get_current(&self) -> Option<usize> {
        self.find(&[TaskState::Running])
    }

    fn find(&self, state: &[TaskState]) -> Option<usize> {
        self.tasks
            .iter()
            .position(|t| state.contains(&t.get_state()))
    }

    pub fn next_to_run(&self) -> Option<usize> {
        self.find(&[TaskState::Ready, TaskState::Suspended])
    }

    pub fn set_state(&mut self, id: TaskID, state: TaskState) {
        match id {
            TaskID::Current => {
                if let Some(c) = self.get_current() {
                    self.tasks[c].set_state(state);
                }
            }
            TaskID::ID(i) => {
                if i < self.tasks.len() {
                    self.tasks[i].set_state(state);
                }
            }
            TaskID::Next => {
                if let Some(r) = self.next_to_run() {
                    self.tasks[r].set_state(state)
                }
            }
        }
    }

    pub fn get_pri(&self, id: TaskID) -> Option<u32> {
        match id {
            TaskID::Current => {
                if let Some(c) = self.get_current() {
                    Some(self.tasks[c].get_prd())
                } else {
                    None
                }
            }
            TaskID::Next => {
                if let Some(r) = self.next_to_run() {
                    Some(self.tasks[r].get_prd())
                } else {
                    None
                }
            }
            TaskID::ID(i) => Some(self.tasks[i].get_prd()),
        }
    }

    pub fn get(&self, id: TaskID) -> Option<&dyn Task> {
        match id {
            TaskID::Current => {
                if let Some(c) = self.get_current() {
                    Some(self.tasks[c])
                } else {
                    None
                }
            }
            TaskID::Next => {
                if let Some(n) = self.next_to_run() {
                    Some(self.tasks[n])
                } else {
                    None
                }
            }
            TaskID::ID(i) => {
                if i < self.tasks.len() {
                    Some(self.tasks[i])
                } else {
                    None
                }
            }
        }
    }
}
