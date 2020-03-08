use crate::arch::Arch;
use crate::task::PeriodicTask;
use crate::task_control::TCB;

pub struct RMS<'a, A: Arch> {
    tasks: [TCB<PeriodicTask>; 4],
    arch: &'a A,
}

impl<'a, A: Arch> RMS<'a, A> {
    pub fn new(t: [PeriodicTask; 4], arch: &'a mut A) -> Self {
        Self {
            tasks: [t[0].into(), t[1].into(), t[2].into(), t[3].into()],
            arch,
        }
    }

    pub fn start(mut self) -> ! {
        for t in self.tasks.iter_mut() {
            t.init();
        }

        self.tasks.as_mut().sort_unstable();

        loop {}
    }

    fn handle_systick(&self) {
        //find next task
        //if pri higher do that
        //if not, mark as ready to run
    }
}
