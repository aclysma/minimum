


pub trait InspectTestTrait {
    fn visit(&self, label: &'static str);
    fn visit_mut(&mut self, label: &'static str);
}