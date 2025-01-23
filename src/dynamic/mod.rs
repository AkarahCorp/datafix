use alloc::string::String;

pub mod impls;
mod list;
pub use list::DynamicList;
mod object;
pub use object::DynamicObject;

#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Number(f64),
    String(String),
    Boolean(bool),
    List(DynamicList),
    Object(DynamicObject),
    Unit,
}

impl Dynamic {
    pub fn new<T: Into<Dynamic>>(value: T) -> Dynamic {
        value.into()
    }

    pub fn as_number(&self) -> Option<&f64> {
        match self {
            Dynamic::Number(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<&bool> {
        match self {
            Dynamic::Boolean(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&DynamicList> {
        match self {
            Dynamic::List(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&DynamicObject> {
        match self {
            Dynamic::Object(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_string_mut(&mut self) -> Option<&mut str> {
        match self {
            Dynamic::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&DynamicList> {
        match self {
            Dynamic::List(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut DynamicObject> {
        match self {
            Dynamic::Object(v) => Some(v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dynamic::{list::DynamicList, object::DynamicObject};

    use super::Dynamic;

    #[test]
    fn unwrap_dyn_str() {
        let str = Dynamic::new("abc");
        assert_eq!(str.as_string(), Some("abc"));
    }

    #[test]
    fn unwrap_dyn_num() {
        let num = Dynamic::new(14.5);
        assert_eq!(num.as_number(), Some(&14.5));
    }

    #[test]
    fn unwrap_dyn_bool() {
        let bl = Dynamic::new(true);
        assert_eq!(bl.as_bool(), Some(&true));
    }

    #[test]
    fn unwrap_dyn_list() {
        let mut list = DynamicList::new();
        list.push(10.0);
        list.push("str");
        list.push(false);

        let bl = Dynamic::new(list.clone());
        assert_eq!(bl.as_list(), Some(&list));
    }

    #[test]
    fn unwrap_dyn_object() {
        let mut obj = DynamicObject::new();
        obj.insert("num", 10.0);
        obj.insert("str", "string!");
        obj.insert("bool", true);

        let bl = Dynamic::new(obj.clone());
        assert_eq!(bl.as_object(), Some(&obj));
    }
}
