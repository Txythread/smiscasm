pub struct MicroOperation {
    /// The offset of the "device" in the control word (left shift, in bits).
    pub pos_in_control_word: u8,
    
    /// The name expected in a file from instructions
    pub name: &'static str,
    
    /// The default state
    pub is_active_low: bool,
}


pub const MICRO_OPERATIONS: [MicroOperation; 30] = [
    MicroOperation { pos_in_control_word: 0, name: "PC_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 1, name: "PC_IN", is_active_low: false },
    MicroOperation { pos_in_control_word: 2, name: "PC_INC", is_active_low: false },
    MicroOperation { pos_in_control_word: 3, name: "MEM_ADDR_PTR_IN", is_active_low: false },
    MicroOperation { pos_in_control_word: 4, name: "ALU_IN_A", is_active_low: false },
    MicroOperation { pos_in_control_word: 5, name: "ALU_IN_B", is_active_low: false },
    MicroOperation { pos_in_control_word: 6, name: "CAL_REG_A_IN", is_active_low: false },
    MicroOperation { pos_in_control_word: 7, name: "CAL_REG_B_IN", is_active_low: false },
    MicroOperation { pos_in_control_word: 8, name: "CAL_REG_A_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 9, name: "CAL_REG_B_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 10, name: "IMMEDIATE_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 11, name: "INSTR_IN", is_active_low: false },
    MicroOperation { pos_in_control_word: 12, name: "MEM_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 13, name: "PLUS_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 14, name: "RESET_MICRO", is_active_low: false },
    MicroOperation { pos_in_control_word: 15, name: "STDTRANS_IN", is_active_low: false },
    MicroOperation { pos_in_control_word: 16, name: "STDTRANS_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 17, name: "STDTRANS_SEND", is_active_low: false },
    MicroOperation { pos_in_control_word: 18, name: "ZF_IN", is_active_low: false },
    MicroOperation { pos_in_control_word: 19, name: "ZF_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 20, name: "SUB_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 21, name: "MEM_BYTE_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 22, name: "LSH_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 23, name: "SP_OUT", is_active_low: false },
    MicroOperation { pos_in_control_word: 24, name: "SP_IN", is_active_low: false },
    MicroOperation { pos_in_control_word: 25, name: "MEM_IN", is_active_low: false },
    MicroOperation { pos_in_control_word: 26, name: "SP_DEC_DW", is_active_low: false },
    MicroOperation { pos_in_control_word: 27, name: "SP_INC_DW", is_active_low: false },
    MicroOperation { pos_in_control_word: 28, name: "MEM_B_IN", is_active_low: false },
    MicroOperation { pos_in_control_word: 29, name: "NAND_OUT", is_active_low: false },
];

/// Generates a default control "word" for when nothing is happening.
pub fn generate_empty_control_word() -> u64 {
    let mut control_word: u64 = 0;
    
    for micro_operation in MICRO_OPERATIONS.iter() {
        let offset = micro_operation.pos_in_control_word;
        let bit = micro_operation.is_active_low;
        
        control_word |= u64::from(bit) << offset;
    }
    
    control_word
}