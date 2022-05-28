use super::*;

pub struct DarkenRule<R: Rule> {
    rule: RuleBox<R>,
    amount: f64,
}

impl<R: Rule> DarkenRule<R> {
    pub fn new(rule: R, amount: f64) -> Self {
        Self { rule: RuleBox::new(rule), amount }
    }
}

impl<R: Rule> Clone for DarkenRule<R> {
    fn clone(&self) -> Self {
        Self {
            rule: self.rule.clone(),
            amount: self.amount
        }
    }
}

impl<R: Rule> Rule for DarkenRule<R> {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        scatter: bool,
    ) -> (Point, usize) {
        let (mut next, index) = self.rule.next(previous, history, shape, scatter);

        next.r *= self.amount;
        next.g *= self.amount;
        next.b *= self.amount;

        (next, index)
    }
}
