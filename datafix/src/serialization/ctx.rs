use core::fmt::{Debug, Display};

use alloc::{string::String, vec::Vec};

#[derive(Clone)]
pub struct Context {
    stack_trace: Vec<TracePoint>,
    cache: Vec<Context>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            stack_trace: [TracePoint::Root].into(),
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

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Stack trace: ")?;
        for element in &self.stack_trace {
            element.fmt(f)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub enum TracePoint {
    Root,
    Field { name: String },
    Array { index: usize },
    Codec { name: String },
}

impl Debug for TracePoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt(f)
    }
}

impl TracePoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TracePoint::Root => write!(f, "$"),
            TracePoint::Field { name } => write!(f, ".{name}"),
            TracePoint::Array { index } => write!(f, "[{index}]"),
            TracePoint::Codec { name } => write!(f, "@ Codec: {name}"),
        }
    }
}
