
pub struct ScopeTimer<'a> {
    start_time: std::time::Instant,
    name: &'a str
}

impl<'a> ScopeTimer<'a> {
    #[allow(unused_must_use)]
    pub fn new(name: &'a str) -> Self {
        ScopeTimer {
            start_time: std::time::Instant::now(),
            name
        }
    }
}

impl<'a> Drop for ScopeTimer<'a> {
    fn drop(&mut self) {
        let end_time = std::time::Instant::now();
        trace!("ScopeTimer {}: {}", self.name, (end_time - self.start_time).as_micros() as f64 / 1000.0)
    }
}