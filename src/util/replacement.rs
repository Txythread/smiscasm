use std::fmt::{Debug, Formatter};

pub(crate) struct Replacement {
    initial_value: String,
    new_value: String,
    is_function: bool,
    is_global: bool,
}

impl Replacement {
    pub(crate) fn new(_name: String, replacement: String, is_function: bool) -> Replacement {
        Replacement {initial_value: _name, new_value: replacement, is_function, is_global: false}
    }

    #[allow(dead_code)]
    pub fn get_name(&self) -> String { self.initial_value.clone() }
    #[allow(dead_code)]
    pub fn get_value(&self) -> String { self.new_value.clone() }
    #[allow(dead_code)]
    pub fn get_is_function(&self) -> bool { self.is_function }
    #[allow(dead_code)]
    pub fn get_is_global(&self) -> bool { self.is_global }
    #[allow(dead_code)]
    pub fn set_is_global(&mut self, is_global: bool) { self.is_global = is_global; }
    #[allow(dead_code)]
    pub fn set_value(&mut self, new_value: String, is_function: bool) { self.new_value = new_value; self.is_function = is_function; }
    #[allow(dead_code)]
    pub fn set_is_function(&mut self, is_function: bool) { self.is_function = is_function; }

    #[allow(dead_code)]
    pub fn make_description(&self) -> String { format!("Replacing {} with {} while being a function: {}", self.initial_value, self.new_value, self.is_function)}
}

impl Debug for Replacement{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Replacement")
            .field("initial_value", &self.initial_value)
            .field("new_value", &self.new_value)
            .field("is_function", &self.is_function)
            .field("is_global", &self.is_global)
            .finish()
    }
}

impl PartialEq for Replacement{
    fn eq(&self, other: &Self) -> bool {
        if self.initial_value != other.initial_value { return false }
        if self.new_value != other.new_value { return false }
        if self.is_function != other.is_function { return false }
        if self.is_global != other.is_global { return false }

        true
    }
}

impl Clone for Replacement{
    fn clone(&self) -> Self { Replacement{ initial_value: self.initial_value.clone(), new_value: self.new_value.clone(), is_function: self.is_function.clone(), is_global: self.is_global.clone() } }
}
