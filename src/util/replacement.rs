use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
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

    pub fn get_name(&self) -> String { self.initial_value.clone() }
    pub fn get_value(&self) -> String { self.new_value.clone() }
    pub fn get_is_function(&self) -> bool { self.is_function }
    pub fn get_is_global(&self) -> bool { self.is_global }
    pub fn set_is_global(&mut self, is_global: bool) { self.is_global = is_global; }
}