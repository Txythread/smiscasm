pub(crate) struct Replacement {
    initial_value: String,
    new_value: String,
    is_function: bool,
}

impl Replacement {
    pub(crate) fn new(_name: String, replacement: String, is_function: bool) -> Replacement {
        Replacement {initial_value: _name, new_value: replacement, is_function}
    }

    pub fn get_name(&self) -> String { self.initial_value.clone() }
    pub fn get_value(&self) -> String { self.new_value.clone() }
    pub fn get_is_function(&self) -> bool { self.is_function }
    pub fn set_value(&mut self, new_value: String, is_function: bool) { self.new_value = new_value; self.is_function = is_function; }

    #[allow(dead_code)]
    pub fn make_description(&self) -> String { format!("Replacing {} with {} while being a function: {}", self.initial_value, self.new_value, self.is_function)}
}

impl Clone for Replacement{
    fn clone(&self) -> Self { Replacement{ initial_value: self.initial_value.clone(), new_value: self.new_value.clone(), is_function: self.is_function.clone() } }
}