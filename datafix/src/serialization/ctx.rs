use core::fmt::Display;

use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone)]
pub struct Context {
    stack_trace: Vec<TracePoint>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            stack_trace: Vec::new(),
        }
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Stack trace: ")?;
        for element in &self.stack_trace {
            write!(f, "{element}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TracePoint {
    Field { name: String },
    Array { index: usize },
}

impl Display for TracePoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TracePoint::Field { name } => {
                write!(f, ".{name}")
            }
            TracePoint::Array { index } => {
                write!(f, "[{index}]")
            }
        }
    }
}
