pub trait Job: Send {
    fn run(&self);
}

/*------------------------------------------*/

pub struct SingleJob {
    func: Box<dyn Fn()>,
}

unsafe impl Send for SingleJob {}

impl SingleJob {
    pub fn new<FuncType>(func: FuncType) -> Self
    where
        FuncType: Fn() + 'static,
    {
        Self {
            func: Box::new(func),
        }
    }
}

impl Job for SingleJob {
    fn run(&self) {
        (self.func)();
    }
}
