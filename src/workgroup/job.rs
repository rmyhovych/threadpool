pub trait Job: Send {
    fn run(&self);
}

/*------------------------------------------*/

pub struct SingleJob<TFuncType: Fn()> {
    func: TFuncType,
}

unsafe impl<TFuncType: Fn()> Send for SingleJob<TFuncType> {}

impl<TFuncType: Fn() + 'static> SingleJob<TFuncType> {
    pub fn new(func: TFuncType) -> Self {
        Self { func }
    }
}

impl<TFuncType: Fn()> Job for SingleJob<TFuncType> {
    fn run(&self) {
        (self.func)();
    }
}
