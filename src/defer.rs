pub struct Defer<F>
where
    F: FnOnce(),
{
    pub f: Option<F>,
}

impl<F> Drop for Defer<F>
where
    F: FnOnce(),
{
    fn drop(&mut self) {
        if let Some(f) = self.f.take() {
            f();
        }
    }
}

#[macro_export]
macro_rules! defer {
    ($code:block) => {
        let _defer = crate::defer::Defer { f: Some(|| $code) };
    };
}
