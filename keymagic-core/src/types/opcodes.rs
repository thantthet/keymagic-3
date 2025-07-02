// Binary opcodes for KM2 format
pub const OP_STRING: u16 = 0x00F0;
pub const OP_VARIABLE: u16 = 0x00F1;
pub const OP_REFERENCE: u16 = 0x00F2;
pub const OP_PREDEFINED: u16 = 0x00F3;
pub const OP_MODIFIER: u16 = 0x00F4;
pub const OP_AND: u16 = 0x00F6;
pub const OP_ANY: u16 = 0x00F8;
pub const OP_SWITCH: u16 = 0x00F9;

// Modifier flags (used with OP_MODIFIER)
pub const FLAG_ANYOF: u16 = 0x00F5;   // Match any character from variable
pub const FLAG_NANYOF: u16 = 0x00F7;  // Match any character NOT in variable