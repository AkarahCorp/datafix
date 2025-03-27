use core::fmt::Display;

use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone)]
pub struct Context {
    stack_trace: Vec<TracePoint>,
    cache: Vec<Context>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            stack_trace: Vec::new(),
            cache: Vec::new(),
        }
    }

    pub fn push_field(&mut self, name: &str) {
        self.stack_trace
            .push(TracePoint::Field { name: name.into() });
    }

    pub fn push_codec(&mut self, name: &str) {
        self.stack_trace
            .push(TracePoint::Codec { name: name.into() });
    }

    pub fn push_array(&mut self, index: usize) {
        self.stack_trace.push(TracePoint::Array { index });
    }

    pub fn pop(&mut self) -> Option<TracePoint> {
        self.stack_trace.pop()
    }

    pub fn save(&mut self) {
        self.cache.push(self.clone());
    }

    pub fn load_save(&mut self) {
        let last = self.cache.pop().unwrap();
        self.stack_trace = last.stack_trace;
    }

    pub fn pop_save(&mut self) -> Option<Context> {
        self.cache.pop()
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
    Codec { name: String },
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
            TracePoint::Codec { name } => {
                write!(f, "| (Codec: {name})")
            }
        }
    }
}
