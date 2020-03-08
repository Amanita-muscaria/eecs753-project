pub type ThreadID = usize;

pub trait Task {
    fn run(&mut self);
    fn init(&self);

    fn get_pri(&self) -> usize;
    fn set_pri(&mut self, p: usize);
    fn stack_bottom(&self) -> *const usize;
}

#[derive(Copy, Clone)]
pub struct PeriodicTask {
    prd: usize,
    run_fn: fn(),
    init_fn: Option<fn()>,
    stack: &'static [usize],
}

impl PeriodicTask {
    pub fn new(run_fn: fn(), init_fn: Option<fn()>, prd: usize, stack: &'static [usize]) -> Self {
        Self {
            run_fn,
            init_fn,
            prd,
            stack,
        }
    }
}

impl Task for PeriodicTask {
    fn run(&mut self) {
        (self.run_fn)();
    }

    fn init(&self) {
        if let Some(p) = self.init_fn {
            (p)()
        }
    }

    fn get_pri(&self) -> usize {
        self.prd
    }

    fn set_pri(&mut self, _: usize) {}

    fn stack_bottom(&self) -> *const usize {
        self.stack[0] as *const usize
    }
}
