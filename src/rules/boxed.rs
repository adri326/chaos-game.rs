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

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.0.reseed(seed);
    }
}


pub struct BoxedChoice(Box<dyn Choice>);

impl BoxedChoice {
    pub fn new<C: Choice + 'static>(choice: C) -> BoxedChoice {
        BoxedChoice(Box::new(choice))
    }

    pub fn as_any(&self) -> &dyn Any {
        &self.0
    }

    pub fn downgrade<C: Choice + 'static>(&self) -> Option<&C> {
        self.as_any()
            .downcast_ref::<C>()
    }
}

impl Clone for BoxedChoice {
    fn clone(&self) -> Self {
        Self(clone_box(self.0.as_ref()))
    }
}

impl Choice for BoxedChoice {
    fn choose_point(
        &mut self,
        history: &[usize],
        shape: &Shape
    ) -> usize {
        self.0.choose_point(history, shape)
    }

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.0.reseed(seed);
    }
}
