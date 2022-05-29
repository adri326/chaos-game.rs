use super::rules::*;
use super::shape::{Shape, Point};

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use rust_lisp::{parse, eval_block, default_env, model::{Value, Env, RuntimeError}};

thread_local! {
    static RULES: RefCell<HashMap<String, BoxedRule>> = RefCell::new(HashMap::new());

    static CHOICES: RefCell<HashMap<String, BoxedChoice>> = RefCell::new(HashMap::new());

    static NONCE: RefCell<usize> = RefCell::new(0);
}

fn as_number(value: Option<&Value>, default_value: f64) -> Result<f64, RuntimeError> {
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

fn as_int(value: Option<&Value>) -> Result<i32, RuntimeError> {
    value.map(|x| x.as_int()).flatten().ok_or(
        RuntimeError::new(format!("Expected integer, got {:?}", value))
    )
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

/// Returns a clone of the choice for the symbol `name`, if it exists
fn get_choice<S: AsRef<str>>(name: S) -> Result<BoxedChoice, RuntimeError> {
    CHOICES.with(|c| {
        c.borrow().get(name.as_ref()).cloned().ok_or(RuntimeError::new(format!("No choice named '{}'!", name.as_ref())))
    })
}

mod crate_macro {
    macro_rules! lisp_choice {
        ($name:ty) => {{
            let choice = <$name>::new();
            let name = format!("{} {}", stringify!($name), next_index());

            CHOICES.with(|c| c.borrow_mut().insert(
                name.clone(),
                BoxedChoice::new(choice)
            ));

            Ok(Value::Symbol(name))
        }};

        ($name:ty, $( $params:ident ),*) => {{
            let choice = <$name>::new( $( $params ),* );
            let name = format!("{} {}", stringify!($name), next_index());

            CHOICES.with(|c| c.borrow_mut().insert(
                name.clone(),
                BoxedChoice::new(choice)
            ));

            Ok(Value::Symbol(name))
        }};
    }

    macro_rules! lisp_mathfun {
        ($env:tt, $name:tt) => {
            fn $name(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
                let x = args.get(0).map(|x| x.as_float()).flatten().ok_or(
                    RuntimeError::new(format!("Expected float, got {:?}", args.get(0)))
                )?;

                Ok(Value::Float(x.$name()))
            }

            $env.entries.insert(String::from(stringify!($name)), Value::NativeFunc($name));
        }
    }

    pub(crate) use lisp_choice;
    pub(crate) use lisp_mathfun;
}

fn choice(_env: Rc<RefCell<Env>>, _args: &Vec<Value>) -> Result<Value, RuntimeError> {
    crate_macro::lisp_choice!(DefaultChoice)
}

fn avoid_choice(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let diff = as_int(args.get(0))? as isize;

    crate_macro::lisp_choice!(AvoidChoice, diff)
}

fn avoid2_choice(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let diff1 = as_int(args.get(0))? as isize;
    let diff2 = as_int(args.get(1))? as isize;

    crate_macro::lisp_choice!(AvoidTwoChoice, diff1, diff2)
}

fn neighbor_choice(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let dist = as_int(args.get(0))?;
    let dist: usize = dist.try_into().ok().ok_or(RuntimeError::new(format!("Expected positive integer, got {}", dist)))?;

    crate_macro::lisp_choice!(NeighborChoice, dist)
}

fn neighborhood_choice(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let dist = as_int(args.get(0))?;
    let dist: usize = dist.try_into().ok().ok_or(RuntimeError::new(format!("Expected positive integer, got {}", dist)))?;

    crate_macro::lisp_choice!(NeighborhoodChoice, dist)
}

fn advance_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let move_ratio = as_number(args.get(0), 0.5)?;
    let color_ratio = as_number(args.get(1), 0.5)?;
    let choice = match args.get(2) {
        Some(x) => get_choice(as_symbol(Some(x))?)?,
        None => BoxedChoice::new(DefaultChoice::default())
    };

    let rule = DefaultRule::new(choice, move_ratio, color_ratio);

    let name = format!("DefaultRule {}", next_index());

    RULES.with(|r| r.borrow_mut().insert(
        name.clone(),
        BoxedRule::new(rule)
    ));

    Ok(Value::Symbol(name))
}

fn spiral_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let rule = get_rule(as_symbol(args.get(0))?)?;

    let delta_low = as_number(args.get(1), 0.0)?;
    let delta_high = as_number(args.get(2), 0.0)?;

    let epsilon_low = as_number(args.get(3), 1.0)?;
    let epsilon_high = as_number(args.get(4), 1.0)?;

    let rule = SpiralRule::new(rule, (delta_low, delta_high), (epsilon_low, epsilon_high));

    let name = format!("SpiralRule {}", next_index());

    RULES.with(|r| r.borrow_mut().insert(
        name.clone(),
        BoxedRule::new(rule)
    ));

    Ok(Value::Symbol(name))
}

fn discrete_spiral_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let rule = get_rule(as_symbol(args.get(0))?)?;

    let p = as_number(args.get(1), 0.5)?;
    let p_scatter = as_number(args.get(5), p)?;

    let delta = as_number(args.get(2), 1.0)?;
    let epsilon = as_number(args.get(3), 1.0)?;
    let darken = as_number(args.get(4), 1.0)?;

    let rule = DiscreteSpiralRule::new(rule, (p, p_scatter), delta, epsilon, darken)
        .map_err(|_| RuntimeError::new(format!(
            "Invalid value for p or p_scatter: expected a number between 0 and 1, got ({}, {})",
            p,
            p_scatter
        )))?;

    let name = format!("DiscreteSpiralRule {}", next_index());

    RULES.with(|r| r.borrow_mut().insert(
        name.clone(),
        BoxedRule::new(rule)
    ));

    Ok(Value::Symbol(name))
}

fn darken_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let rule = get_rule(as_symbol(args.get(0))?)?;

    let amount = as_number(args.get(1), 1.0)?;

    let rule = DarkenRule::new(rule, amount);

    let name = format!("DarkenRule {}", next_index());

    RULES.with(|r| r.borrow_mut().insert(
        name.clone(),
        BoxedRule::new(rule)
    ));

    Ok(Value::Symbol(name))
}

fn or_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let p = as_number(args.get(0), 0.5)?;
    let p_scatter = as_number(args.get(3), 0.5)?;

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

fn float(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    match args.get(0) {
        Some(Value::Float(x)) => Ok(Value::Float(*x)),
        Some(Value::Int(x)) => Ok(Value::Float(*x as f32)),
        Some(Value::String(x)) => Ok(Value::Float(x.parse::<f32>().map_err(|e| RuntimeError::new(format!("{:?}", e)))?)),
        Some(y) => Err(RuntimeError::new(format!("Invalid argument for 'float': {:?}", y))),
        None => Err(RuntimeError::new("'float' should have 1 parameter!"))
    }
}

fn populate_env(env: &mut Env) {
    env.entries.insert(
        String::from("float"),
        Value::NativeFunc(float)
    );

    crate_macro::lisp_mathfun!(env, sqrt);
    crate_macro::lisp_mathfun!(env, exp);
    crate_macro::lisp_mathfun!(env, sin);
    crate_macro::lisp_mathfun!(env, cos);
    crate_macro::lisp_mathfun!(env, tan);
    crate_macro::lisp_mathfun!(env, asin);
    crate_macro::lisp_mathfun!(env, acos);
    crate_macro::lisp_mathfun!(env, atan);

    env.entries.insert(
        String::from("advance-rule"),
        Value::NativeFunc(advance_rule)
    );

    env.entries.insert(
        String::from("spiral-rule"),
        Value::NativeFunc(spiral_rule)
    );

    env.entries.insert(
        String::from("discrete-spiral-rule"),
        Value::NativeFunc(discrete_spiral_rule)
    );

    env.entries.insert(
        String::from("or-rule"),
        Value::NativeFunc(or_rule)
    );

    env.entries.insert(
        String::from("darken-rule"),
        Value::NativeFunc(darken_rule)
    );

    env.entries.insert(
        String::from("choice"),
        Value::NativeFunc(choice)
    );

    env.entries.insert(
        String::from("avoid-choice"),
        Value::NativeFunc(avoid_choice)
    );

    env.entries.insert(
        String::from("avoid2-choice"),
        Value::NativeFunc(avoid2_choice)
    );

    env.entries.insert(
        String::from("neighbor-choice"),
        Value::NativeFunc(neighbor_choice)
    );

    env.entries.insert(
        String::from("neighborhood-choice"),
        Value::NativeFunc(neighborhood_choice)
    );
}

fn extract_shape(value: &Value) -> Result<Shape, RuntimeError> {
    use rand::Rng;

    let mut res = Vec::new();
    if let Value::List(list) = value {
        for point in list {
            if let Value::List(sublist) = point {
                let mut numbers = Vec::new();
                for number in sublist.into_iter().take(5) {
                    match number {
                        Value::Float(x) => numbers.push(x as f64),
                        Value::Int(x) => numbers.push(x as f64),
                        y => return Err(RuntimeError::new(
                            format!("Invalid point coordinate/color: expected number, got {:?}", y)
                        ))
                    }
                }

                let (x, y, r, g, b) = if numbers.len() == 2 {
                    let mut rng = rand::thread_rng();
                    (numbers[0], numbers[1], rng.gen(), rng.gen(), rng.gen())
                } else if numbers.len() == 5 {
                    (numbers[0], numbers[1], numbers[2], numbers[3], numbers[4])
                } else {
                    return Err(RuntimeError::new(format!("Expected point to have 2 or 5 numbers, got {}", numbers.len())));
                };

                res.push(Point::new(x, y, (r, g, b)));
            } else {
                return Err(RuntimeError::new(format!("Invalid point: expected list, got {:?}", point)));
            }
        }

        Ok(res)
    } else {
        return Err(RuntimeError::new(format!("Expected SHAPE to be a list, got {:?}", value)));
    }
}

pub fn eval_rule(raw: &str) -> Result<(Option<BoxedRule>, Option<Shape>), RuntimeError> {
    let mut env = default_env();
    populate_env(&mut env);

    let env = Rc::new(RefCell::new(env));

    let mut ast = Vec::new();
    for item in parse(raw) {
        ast.push(item.map_err(|e| RuntimeError::new(e.msg))?);
    }

    let evaluation_result = eval_block(env.clone(), ast.into_iter())?;
    let rule = get_rule(as_symbol(Some(&evaluation_result))?)?;

    // Cleanup:
    RULES.with(|r| {
        *r.borrow_mut() = HashMap::new();
    });

    CHOICES.with(|c| {
        *c.borrow_mut() = HashMap::new();
    });

    NONCE.with(|n| {
        *n.borrow_mut() = 0;
    });

    let shape = if let Some(shape) = env.borrow().entries.get("SHAPE") {
        Some(extract_shape(shape)?)
    } else {
        None
    };

    Ok((Some(rule), shape))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        assert!(eval_rule("(or-rule 0.5 (advance-rule 0.25) (advance-rule 0.5))").is_ok());
    }
}
