
use rayon::Scope;


pub struct Runtime {
    count: u8
}

impl Runtime {
    pub fn run(&mut self) {
        rayon::scope(move |scope| {
            scope.spawn(|scope| self.run_1(scope));
        });
    }

    fn run_1<'s>(&'s mut self, scope: &Scope<'s>) {
        self.count -= 1;
        self.run_1_1(scope);
    }

    fn run_1_1<'s>(&'s mut self, scope: &Scope<'s>) {
        if self.count > 0 {
            scope.spawn(|scope| self.run_1(scope));
        }
    }
}
