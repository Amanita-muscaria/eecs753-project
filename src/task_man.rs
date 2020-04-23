use crate::tasks::{Task, TaskState};
use core::cmp::Ordering;

type TaskList<'a> = [&'a dyn Task; 3];

pub enum TaskID {
    Current,
    NextRelease,
    ID(usize),
}

pub struct TaskMan<'a> {
    tasks: TaskList<'a>,
    releases: [ReleaseStruct; 3],
    ticks_pr_10ms: u32,
}

impl<'a> TaskMan<'a> {
    pub const fn new(tasks: TaskList<'a>) -> Self {
        Self {
            tasks,
            releases: [ReleaseStruct {
                released: true,
                time: 0,
            }; 3],
            ticks_pr_10ms: 0,
        }
    }

    pub fn set_ticks(&mut self, ticks: u32) {
        self.ticks_pr_10ms = ticks;
    }

    pub fn get_current(&self) -> Option<usize> {
        self.tasks
            .iter()
            .position(|t| t.get_state() == TaskState::Running)
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
            TaskID::NextRelease => {
                if let Some(r) = self.next_to_release() {
                    self.tasks[r].set_state(state)
                }
            }
        }
    }

    pub fn set_next_release(&mut self, id: TaskID, time: u32) {
        let t = match id {
            TaskID::Current => {
                if let Some(c) = self.get_current() {
                    c
                } else {
                    return;
                }
            }
            TaskID::ID(x) => x,
            TaskID::NextRelease => {
                if let Some(r) = self.next_to_release() {
                    r
                } else {
                    return;
                }
            }
        };
        let prd = self.tasks[t].get_prd();
        self.releases[t].next(prd * self.ticks_pr_10ms + time);
    }

    pub fn next_to_release(&self) -> Option<usize> {
        if let Some(r) = self
            .releases
            .iter()
            .filter(|r| !r.released)
            .enumerate()
            .min()
        {
            Some(r.0)
        } else {
            None
        }
    }

    pub fn release_next(&mut self) {
        let n = self.next_to_release().unwrap();
        self.releases[n].released = true;
        self.tasks[n].set_state(TaskState::Ready);
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
            TaskID::NextRelease => {
                if let Some(r) = self.next_to_release() {
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
            TaskID::NextRelease => {
                if let Some(n) = self.next_to_release() {
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

struct ReleaseStruct {
    released: bool,
    time: u32,
}

impl ReleaseStruct {
    pub fn next(&mut self, time: u32) {
        self.time = time;
        self.released = false;
    }
}

impl Ord for ReleaseStruct {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl PartialOrd for ReleaseStruct {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl PartialEq for ReleaseStruct {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for ReleaseStruct {}
