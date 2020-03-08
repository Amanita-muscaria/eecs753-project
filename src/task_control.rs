use crate::task::{PeriodicTask, Task};
use core::cmp::Ordering;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskState {
    Waiting,
    Stopped,
    Running,
    Dead,
}

pub struct TCB<T: Task> {
    task: T,
    stack_ptr: *const usize,
    state: TaskState,
}

impl<T: Task> TCB<T> {
    pub fn get_state(&self) -> TaskState {
        self.state
    }

    pub fn init(&mut self) {
        self.task.init();
    }
}

impl<T: Task> From<T> for TCB<T> {
    fn from(p: T) -> Self {
        TCB {
            stack_ptr: p.stack_bottom(),
            task: p,
            state: TaskState::Waiting,
        }
    }
}

impl<T: Task> PartialOrd for TCB<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.task.get_pri().partial_cmp(&other.task.get_pri())
    }
}

impl<T: Task> PartialEq for TCB<T> {
    fn eq(&self, other: &Self) -> bool {
        self.task.get_pri() == other.task.get_pri()
    }
}

impl<T: Task> Eq for TCB<T> {}

impl<T: Task> Ord for TCB<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.task.get_pri().cmp(&other.task.get_pri())
    }
}
