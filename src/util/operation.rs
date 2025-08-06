pub enum Operation {
    ADDITION,
    SUBTRACTION,
    MULTIPLICATION,
    DIVISION,
    MODULO
}

impl Operation {
    pub fn from_string(s: &str) -> Option<Operation> {
        match s {
            "+" => Some(Operation::ADDITION),
            "-" => Some(Operation::SUBTRACTION),
            "*" => Some(Operation::MULTIPLICATION),
            "/" => Some(Operation::DIVISION),
            "%" => Some(Operation::MODULO),
            _ => None,
        }
    }

    pub fn perform(&self, op1: i64, op2: i64) -> i64 {
        match self{
            Operation::ADDITION => op1 + op2,
            Operation::SUBTRACTION => op1 - op2,
            Operation::MULTIPLICATION => op1 * op2,
            Operation::DIVISION => op1 / op2,
            Operation::MODULO => op1 % op2,
        }
    }
}