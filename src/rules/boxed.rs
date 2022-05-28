use super::*;
use std::any::Any;
use dyn_clone::clone_box;

pub struct BoxedRule(Box<dyn Rule>);

impl BoxedRule {
    pub fn new<R: Rule + 'static>(rule: R) -> BoxedRule {
        BoxedRule(Box::new(rule))
    }

    pub fn as_any(&self) -> &dyn Any {
        &self.0
    }

    pub fn downgrade<R: Rule + 'static>(&self) -> Option<&R> {
        self.as_any()
            .downcast_ref::<R>()
    }
}

impl Clone for BoxedRule {
    fn clone(&self) -> Self {
        Self(clone_box(self.0.as_ref()))
    }
}

impl Rule for BoxedRule {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        scatter: bool,
    ) -> (Point, usize) {
        self.0.next(previous, history, shape, scatter)
    }
}
