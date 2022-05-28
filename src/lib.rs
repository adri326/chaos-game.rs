pub const BG_R: f64 = 0.001;
pub const BG_G: f64 = 0.001;
pub const BG_B: f64 = 0.001;

pub const GAMMA: f64 = 2.2;

pub mod shape;

pub mod world;

pub mod rules;

#[cfg(feature = "box")]
pub mod lisp;
