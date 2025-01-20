use super::{Dynamic, list::DynamicList, object::DynamicObject};

impl From<f64> for Dynamic {
    fn from(value: f64) -> Self {
        Dynamic::Number(value)
    }
}

impl From<&str> for Dynamic {
    fn from(value: &str) -> Self {
        Dynamic::String(value.to_string())
    }
}

impl From<String> for Dynamic {
    fn from(value: String) -> Self {
        Dynamic::String(value)
    }
}

impl From<bool> for Dynamic {
    fn from(value: bool) -> Self {
        Dynamic::Boolean(value)
    }
}

impl<T: Into<Dynamic>> From<Vec<T>> for Dynamic {
    fn from(value: Vec<T>) -> Self {
        Dynamic::List(value.into())
    }
}

impl From<DynamicList> for Dynamic {
    fn from(value: DynamicList) -> Self {
        Dynamic::List(value)
    }
}

impl From<DynamicObject> for Dynamic {
    fn from(value: DynamicObject) -> Self {
        Dynamic::Object(value)
    }
}
