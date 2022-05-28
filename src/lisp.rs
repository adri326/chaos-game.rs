use super::rules::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use rust_lisp::{parse, eval_block, default_env, model::{Value, Env, RuntimeError}};

thread_local! {
    static RULES: RefCell<HashMap<String, BoxedRule>> = RefCell::new(HashMap::new());

    static NONCE: RefCell<usize> = RefCell::new(0);
}

fn as_float(value: Option<&Value>, default_value: f64) -> Result<f64, RuntimeError> {
    Ok(match value {
        Some(Value::Float(x)) => *x as f64,
        Some(Value::Int(x)) => *x as f64,
        Some(y) => return Err(RuntimeError::new(format!("Expected int or float, got {:?}", y))),
        _ => default_value
    })
}

fn as_symbol(value: Option<&Value>) -> Result<String, RuntimeError> {
    value
        .map(|x| x.as_symbol())
        .flatten()
        .ok_or(RuntimeError::new(format!("Expected symbol, got {:?}", value)))
}

fn next_index() -> usize {
    NONCE.with(|n| {
        let mut guard = n.borrow_mut();
        *guard += 1;
        *guard
    })
}

/// Returns a clone of the rule for the symbol `name`, if it exists
fn get_rule<S: AsRef<str>>(name: S) -> Result<BoxedRule, RuntimeError> {
    RULES.with(|r| {
        r.borrow().get(name.as_ref()).cloned().ok_or(RuntimeError::new(format!("No rule named '{}'!", name.as_ref())))
    })
}

fn advance_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let choice = DefaultChoice::default();

    let move_ratio = as_float(args.get(0), 0.5)?;
    let color_ratio = as_float(args.get(1), 0.5)?;

    println!("{:?}", args);

    println!("{:?} {:?}", move_ratio, color_ratio);

    let rule = DefaultRule::new(choice, move_ratio, color_ratio);

    let name = format!("DefaultRule {}", next_index());

    RULES.with(|r| r.borrow_mut().insert(
        name.clone(),
        BoxedRule::new(rule)
    ));

    Ok(Value::Symbol(name))
}

fn or_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let p = as_float(args.get(0), 0.5)?;
    let p_scatter = as_float(args.get(3), 0.5)?;

    let left = get_rule(as_symbol(args.get(1))?)?;
    let right = get_rule(as_symbol(args.get(2))?)?;

    let rule = OrRule::new(left, right, p, p_scatter);

    let name = format!("OrRule {}", next_index());

    RULES.with(|r| r.borrow_mut().insert(
        name.clone(),
        BoxedRule::new(rule)
    ));

    Ok(Value::Symbol(name))
}

pub fn eval_rule(raw: &str) -> Result<BoxedRule, RuntimeError> {
    let mut env = default_env();

    env.entries.insert(
        String::from("advance-rule"),
        Value::NativeFunc(advance_rule)
    );

    env.entries.insert(
        String::from("or-rule"),
        Value::NativeFunc(or_rule)
    );

    let env = Rc::new(RefCell::new(env));

    let mut ast = Vec::new();
    for item in parse(raw) {
        ast.push(item.map_err(|e| RuntimeError::new(e.msg))?);
    }

    let evaluation_result = eval_block(env.clone(), ast.into_iter())?;

    let rule = get_rule(as_symbol(Some(&evaluation_result))?)?;

    RULES.with(|r| {
        *r.borrow_mut() = HashMap::new();
    });

    NONCE.with(|n| {
        *n.borrow_mut() = 0;
    });

    Ok(rule)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        assert!(eval_rule("(or-rule 0.5 (advance-rule 0.25) (advance-rule 0.5))").is_ok());
    }
}
