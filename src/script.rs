use super::rules::*;
use super::shape::{Shape, Point};

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
// use rust_lisp::{parse, eval_block, default_env, model::{Value, Env, RuntimeError}};
use passerine::{
    common::source::Source,
    common::data::Data,
    compile_with_ffi,
    run,
    compiler::syntax::Syntax,
    vm::trace::Trace,
    core::ffi_core,
    core::extract::*,
    core::ffi::{FFIFunction},
};

// thread_local! {
//     static RULES: RefCell<HashMap<String, BoxedRule>> = RefCell::new(HashMap::new());

//     static CHOICES: RefCell<HashMap<String, BoxedChoice>> = RefCell::new(HashMap::new());

//     static NONCE: RefCell<usize> = RefCell::new(0);
// }

fn as_number(value: Data) -> Result<f64, String> {
    Ok(match value {
        Data::Real(x) => x,
        Data::Integer(x) => x as f64,
        y => return Err(format!("Expected int or float, got {:?}", y)),
    })
}

// fn as_symbol(value: Option<&Value>) -> Result<String, RuntimeError> {
//     value
//         .map(|x| x.as_symbol())
//         .flatten()
//         .ok_or(RuntimeError::new(format!("Expected symbol, got {:?}", value)))
// }

// fn as_int(value: Option<&Value>) -> Result<i32, RuntimeError> {
//     value.map(|x| x.as_int()).flatten().ok_or(
//         RuntimeError::new(format!("Expected integer, got {:?}", value))
//     )
// }

// fn next_index() -> usize {
//     NONCE.with(|n| {
//         let mut guard = n.borrow_mut();
//         *guard += 1;
//         *guard
//     })
// }

// /// Returns a clone of the rule for the symbol `name`, if it exists
// fn get_rule<S: AsRef<str>>(name: S) -> Result<BoxedRule, RuntimeError> {
//     RULES.with(|r| {
//         r.borrow().get(name.as_ref()).cloned().ok_or(RuntimeError::new(format!("No rule named '{}'!", name.as_ref())))
//     })
// }

// /// Returns a clone of the choice for the symbol `name`, if it exists
// fn get_choice<S: AsRef<str>>(name: S) -> Result<BoxedChoice, RuntimeError> {
//     CHOICES.with(|c| {
//         c.borrow().get(name.as_ref()).cloned().ok_or(RuntimeError::new(format!("No choice named '{}'!", name.as_ref())))
//     })
// }

// mod crate_macro {
//     macro_rules! lisp_choice {
//         ($name:ty) => {{
//             let choice = <$name>::new();
//             let name = format!("{} {}", stringify!($name), next_index());

//             CHOICES.with(|c| c.borrow_mut().insert(
//                 name.clone(),
//                 BoxedChoice::new(choice)
//             ));

//             Ok(Value::Symbol(name))
//         }};

//         ($name:ty, $( $params:ident ),*) => {{
//             let choice = <$name>::new( $( $params ),* );
//             let name = format!("{} {}", stringify!($name), next_index());

//             CHOICES.with(|c| c.borrow_mut().insert(
//                 name.clone(),
//                 BoxedChoice::new(choice)
//             ));

//             Ok(Value::Symbol(name))
//         }};
//     }

//     macro_rules! lisp_mathfun {
//         ($env:tt, $name:tt) => {
//             fn $name(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//                 let x = args.get(0).map(|x| x.as_float()).flatten().ok_or(
//                     RuntimeError::new(format!("Expected float, got {:?}", args.get(0)))
//                 )?;

//                 Ok(Value::Float(x.$name()))
//             }

//             $env.entries.insert(String::from(stringify!($name)), Value::NativeFunc($name));
//         }
//     }

//     pub(crate) use lisp_choice;
//     pub(crate) use lisp_mathfun;
// }

// fn choice(_env: Rc<RefCell<Env>>, _args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     crate_macro::lisp_choice!(DefaultChoice)
// }

// fn avoid_choice(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     let diff = as_int(args.get(0))? as isize;

//     crate_macro::lisp_choice!(AvoidChoice, diff)
// }

// fn avoid2_choice(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     let diff1 = as_int(args.get(0))? as isize;
//     let diff2 = as_int(args.get(1))? as isize;

//     crate_macro::lisp_choice!(AvoidTwoChoice, diff1, diff2)
// }

// fn neighbor_choice(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     let dist = as_int(args.get(0))?;
//     let dist: usize = dist.try_into().ok().ok_or(RuntimeError::new(format!("Expected positive integer, got {}", dist)))?;

//     crate_macro::lisp_choice!(NeighborChoice, dist)
// }

// fn neighborhood_choice(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     let dist = as_int(args.get(0))?;
//     let dist: usize = dist.try_into().ok().ok_or(RuntimeError::new(format!("Expected positive integer, got {}", dist)))?;

//     crate_macro::lisp_choice!(NeighborhoodChoice, dist)
// }

// fn advance_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     let move_ratio = as_number(args.get(0), 0.5)?;
//     let color_ratio = as_number(args.get(1), 0.5)?;
//     let choice = match args.get(2) {
//         Some(x) => get_choice(as_symbol(Some(x))?)?,
//         None => BoxedChoice::new(DefaultChoice::default())
//     };

//     let rule = DefaultRule::new(choice, move_ratio, color_ratio);

//     let name = format!("DefaultRule {}", next_index());

//     RULES.with(|r| r.borrow_mut().insert(
//         name.clone(),
//         BoxedRule::new(rule)
//     ));

//     Ok(Value::Symbol(name))
// }

// fn spiral_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     let rule = get_rule(as_symbol(args.get(0))?)?;

//     let delta_low = as_number(args.get(1), 0.0)?;
//     let delta_high = as_number(args.get(2), 0.0)?;

//     let epsilon_low = as_number(args.get(3), 1.0)?;
//     let epsilon_high = as_number(args.get(4), 1.0)?;

//     let rule = SpiralRule::new(rule, (delta_low, delta_high), (epsilon_low, epsilon_high));

//     let name = format!("SpiralRule {}", next_index());

//     RULES.with(|r| r.borrow_mut().insert(
//         name.clone(),
//         BoxedRule::new(rule)
//     ));

//     Ok(Value::Symbol(name))
// }

// fn discrete_spiral_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     let rule = get_rule(as_symbol(args.get(0))?)?;

//     let p = as_number(args.get(1), 0.5)?;
//     let p_scatter = as_number(args.get(5), p)?;

//     let delta = as_number(args.get(2), 1.0)?;
//     let epsilon = as_number(args.get(3), 1.0)?;
//     let darken = as_number(args.get(4), 1.0)?;

//     let rule = DiscreteSpiralRule::new(rule, (p, p_scatter), delta, epsilon, darken)
//         .map_err(|_| RuntimeError::new(format!(
//             "Invalid value for p or p_scatter: expected a number between 0 and 1, got ({}, {})",
//             p,
//             p_scatter
//         )))?;

//     let name = format!("DiscreteSpiralRule {}", next_index());

//     RULES.with(|r| r.borrow_mut().insert(
//         name.clone(),
//         BoxedRule::new(rule)
//     ));

//     Ok(Value::Symbol(name))
// }

// fn darken_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     let rule = get_rule(as_symbol(args.get(0))?)?;

//     let amount = as_number(args.get(1), 1.0)?;

//     let rule = DarkenRule::new(rule, amount);

//     let name = format!("DarkenRule {}", next_index());

//     RULES.with(|r| r.borrow_mut().insert(
//         name.clone(),
//         BoxedRule::new(rule)
//     ));

//     Ok(Value::Symbol(name))
// }

// fn or_rule(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     let p = as_number(args.get(0), 0.5)?;
//     let p_scatter = as_number(args.get(3), 0.5)?;

//     let left = get_rule(as_symbol(args.get(1))?)?;
//     let right = get_rule(as_symbol(args.get(2))?)?;

//     let rule = OrRule::new(left, right, p, p_scatter);

//     let name = format!("OrRule {}", next_index());

//     RULES.with(|r| r.borrow_mut().insert(
//         name.clone(),
//         BoxedRule::new(rule)
//     ));

//     Ok(Value::Symbol(name))
// }

// fn float(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
//     match args.get(0) {
//         Some(Value::Float(x)) => Ok(Value::Float(*x)),
//         Some(Value::Int(x)) => Ok(Value::Float(*x as f32)),
//         Some(Value::String(x)) => Ok(Value::Float(x.parse::<f32>().map_err(|e| RuntimeError::new(format!("{:?}", e)))?)),
//         Some(y) => Err(RuntimeError::new(format!("Invalid argument for 'float': {:?}", y))),
//         None => Err(RuntimeError::new("'float' should have 1 parameter!"))
//     }
// }

// fn populate_env(env: &mut Env) {
//     env.entries.insert(
//         String::from("float"),
//         Value::NativeFunc(float)
//     );

//     crate_macro::lisp_mathfun!(env, sqrt);
//     crate_macro::lisp_mathfun!(env, exp);
//     crate_macro::lisp_mathfun!(env, sin);
//     crate_macro::lisp_mathfun!(env, cos);
//     crate_macro::lisp_mathfun!(env, tan);
//     crate_macro::lisp_mathfun!(env, asin);
//     crate_macro::lisp_mathfun!(env, acos);
//     crate_macro::lisp_mathfun!(env, atan);

//     env.entries.insert(
//         String::from("advance-rule"),
//         Value::NativeFunc(advance_rule)
//     );

//     env.entries.insert(
//         String::from("spiral-rule"),
//         Value::NativeFunc(spiral_rule)
//     );

//     env.entries.insert(
//         String::from("discrete-spiral-rule"),
//         Value::NativeFunc(discrete_spiral_rule)
//     );

//     env.entries.insert(
//         String::from("or-rule"),
//         Value::NativeFunc(or_rule)
//     );

//     env.entries.insert(
//         String::from("darken-rule"),
//         Value::NativeFunc(darken_rule)
//     );

//     env.entries.insert(
//         String::from("choice"),
//         Value::NativeFunc(choice)
//     );

//     env.entries.insert(
//         String::from("avoid-choice"),
//         Value::NativeFunc(avoid_choice)
//     );

//     env.entries.insert(
//         String::from("avoid2-choice"),
//         Value::NativeFunc(avoid2_choice)
//     );

//     env.entries.insert(
//         String::from("neighbor-choice"),
//         Value::NativeFunc(neighbor_choice)
//     );

//     env.entries.insert(
//         String::from("neighborhood-choice"),
//         Value::NativeFunc(neighborhood_choice)
//     );
// }

// fn extract_shape(value: &Value) -> Result<Shape, RuntimeError> {
//     use rand::Rng;

//     let mut res = Vec::new();
//     if let Value::List(list) = value {
//         for point in list {
//             if let Value::List(sublist) = point {
//                 let mut numbers = Vec::new();
//                 for number in sublist.into_iter().take(5) {
//                     match number {
//                         Value::Float(x) => numbers.push(x as f64),
//                         Value::Int(x) => numbers.push(x as f64),
//                         y => return Err(RuntimeError::new(
//                             format!("Invalid point coordinate/color: expected number, got {:?}", y)
//                         ))
//                     }
//                 }

//                 let (x, y, r, g, b) = if numbers.len() == 2 {
//                     let mut rng = rand::thread_rng();
//                     (numbers[0], numbers[1], rng.gen(), rng.gen(), rng.gen())
//                 } else if numbers.len() == 5 {
//                     (numbers[0], numbers[1], numbers[2], numbers[3], numbers[4])
//                 } else {
//                     return Err(RuntimeError::new(format!("Expected point to have 2 or 5 numbers, got {}", numbers.len())));
//                 };

//                 res.push(Point::new(x, y, (r, g, b)));
//             } else {
//                 return Err(RuntimeError::new(format!("Invalid point: expected list, got {:?}", point)));
//             }
//         }

//         Ok(res)
//     } else {
//         return Err(RuntimeError::new(format!("Expected SHAPE to be a list, got {:?}", value)));
//     }
// }

macro_rules! passerine_rule {
    ($ffi:tt, $rules:tt, $choices:tt, $nonce:tt, $name:tt, $closure:expr) => {{
        let choices = $choices.clone();
        let rules = $rules.clone();
        let nonce = $nonce.clone();
        let closure = $closure;
        $ffi.add($name, FFIFunction::new(Box::new(move |data| {
            let rule: Result<_, String> = closure(data, rules.clone(), choices.clone());
            let rule = rule?;
            let name = get_name!($name, nonce);

            rules.borrow_mut().insert(name.clone(), BoxedRule::new(rule));

            Ok(Data::String(name))
        }))).unwrap();
    }}
}

macro_rules! passerine_choice {
    ($ffi:tt, $choices:tt, $nonce:tt, $name:tt, $closure:expr) => {{
        let choices = $choices.clone();
        let nonce = $nonce.clone();
        let closure = $closure;
        $ffi.add($name, FFIFunction::new(Box::new(move |data| {
            let choice: Result<_, String> = closure(data, choices.clone());
            let choice = choice?;
            let name = get_name!($name, nonce);

            choices.borrow_mut().insert(name.clone(), BoxedChoice::new(choice));

            Ok(Data::String(name))
        }))).unwrap();
    }}
}

macro_rules! get_name {
    ($base:tt, $nonce:tt) => {{
        let id = {
            let mut guard = $nonce.borrow_mut();
            *guard += 1;
            *guard
        };

        format!("{}::{}", $base, id)
    }}
}

// TODO: deduplicate?
fn get_choice(
    choices: &Rc<RefCell<HashMap<String, BoxedChoice>>>,
    value: Data
) -> Result<BoxedChoice, String> {
    match value {
        Data::String(s) => choices.borrow().get(&s).cloned().ok_or(format!("No choice named {}", s)),
        y => Err(format!("Expected choice, got {:?}", y))
    }
}

fn get_rule(
    rules: &Rc<RefCell<HashMap<String, BoxedRule>>>,
    value: Data
) -> Result<BoxedRule, String> {
    match value {
        Data::String(s) => rules.borrow().get(&s).cloned().ok_or(format!("No rule named {}", s)),
        y => Err(format!("Expected rule, got {:?}", y))
    }
}

#[derive(Debug)]
pub enum SyntaxOrTrace {
    Syntax(Syntax),
    Trace(Trace),
}

impl From<Syntax> for SyntaxOrTrace {
    fn from(syntax: Syntax) -> SyntaxOrTrace {
        SyntaxOrTrace::Syntax(syntax)
    }
}

impl From<Trace> for SyntaxOrTrace {
    fn from(trace: Trace) -> SyntaxOrTrace {
        SyntaxOrTrace::Trace(trace)
    }
}

pub fn eval_rule(raw: &str) -> Result<(Option<BoxedRule>, Option<Shape>), SyntaxOrTrace> {
    // let mut env = default_env();
    // populate_env(&mut env);

    // let env = Rc::new(RefCell::new(env));

    // let mut ast = Vec::new();
    // for item in parse(raw) {
    //     ast.push(item.map_err(|e| RuntimeError::new(e.msg))?);
    // }

    // let evaluation_result = eval_block(env.clone(), ast.into_iter())?;
    // let rule = get_rule(as_symbol(Some(&evaluation_result))?)?;

    // // Cleanup:
    // RULES.with(|r| {
    //     *r.borrow_mut() = HashMap::new();
    // });

    // CHOICES.with(|c| {
    //     *c.borrow_mut() = HashMap::new();
    // });

    // NONCE.with(|n| {
    //     *n.borrow_mut() = 0;
    // });

    // let shape = if let Some(shape) = env.borrow().entries.get("SHAPE") {
    //     Some(extract_shape(shape)?)
    // } else {
    //     None
    // };

    // Ok((rule, shape))

    let source = Source::source(&format!("{}\n{}", include_str!("prelude.pn"), raw));

    let mut ffi = ffi_core();
    type Rules = Rc<RefCell<HashMap<String, BoxedRule>>>;
    type Choices = Rc<RefCell<HashMap<String, BoxedChoice>>>;
    type Nonce = Rc<RefCell<usize>>;
    let rules: Rules = Rc::new(RefCell::new(HashMap::new()));
    let choices: Choices = Rc::new(RefCell::new(HashMap::new()));
    let nonce: Nonce = Rc::new(RefCell::new(0));

    passerine_choice!(
        ffi, choices, nonce, "choice",
        |_data, _choices: Choices| -> Result<_, String> {
            Ok(DefaultChoice::default())
        }
    );

    passerine_rule!(
        ffi, rules, choices, nonce, "advance_rule",
        |data, _rules: Rules, choices: Choices| -> Result<_, String> {
            let (amount, color_amount, choice) = triop(data);
            let amount = as_number(amount)?;
            let color_amount = as_number(color_amount)?;
            let choice = get_choice(&choices, choice)?;

            Ok(DefaultRule::new(choice, amount, color_amount))
        }
    );

    {
        let rules = rules.clone();
        ffi.add("set_rule", FFIFunction::new(Box::new(move |data| {
            let rule = get_rule(&rules, data)?;
            rules.borrow_mut().insert(String::from("rule"), rule);
            Ok(Data::Unit)
        }))).unwrap();
    }

    let compiled = compile_with_ffi(source, ffi)?;
    run(compiled)?;

    let rule = rules.borrow().get("rule").cloned();

    Ok((rule, None))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        eval_rule(r#"
            advance_rule (0.5)
            set_rule (advance_rule (0.5, 0.5))
        "#).unwrap();
    }
}
