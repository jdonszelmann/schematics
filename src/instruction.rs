#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReducedRegister {
    Rra = 0,
    Rrb = 1,
    Rrc = 2,
    Rrd = 3,
    Rre = 4,
    Rrf = 5,
    Rrg = 6,
    Rrh = 7,
}

impl ReducedRegister {
    pub fn encode(&self) -> u8 {
        *self as u8
    }

    pub fn from_num(n: u8) -> Option<Self> {
        match n {
            0 => Some(Self::Rra),
            1 => Some(Self::Rrb),
            2 => Some(Self::Rrc),
            3 => Some(Self::Rrd),
            4 => Some(Self::Rre),
            5 => Some(Self::Rrf),
            6 => Some(Self::Rrg),
            7 => Some(Self::Rrh),
            _ => None,
        }
    }
}

impl Into<Register> for ReducedRegister {
    fn into(self) -> Register {
        Register::from_num(self.encode()).unwrap()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Register {
    Ra = 0,
    Rb = 1,
    Rc = 2,
    Rd = 3,
    Re = 4,
    Rf = 5,
    Rg = 6,
    Rh = 7,
    Rnull = 8,
    Rone = 9,
    Rout = 10,
    Rin = 11,
    Rreserved1 = 12,
    Rreserved2 = 13,
    Rflags = 14,
    Rpc = 15,
}

impl Register {
    pub fn encode(&self) -> u8 {
        *self as u8
    }

    pub fn reduce(&self) -> Option<ReducedRegister> {
        ReducedRegister::from_num(self.encode())
    }

    pub fn from_num(n: u8) -> Option<Self> {
        match n {
            0 => Some(Self::Ra),
            1 => Some(Self::Rb),
            2 => Some(Self::Rc),
            3 => Some(Self::Rd),
            4 => Some(Self::Re),
            5 => Some(Self::Rf),
            6 => Some(Self::Rg),
            7 => Some(Self::Rh),
            8 => Some(Self::Rnull),
            9 => Some(Self::Rone),
            10 => Some(Self::Rout),
            11 => Some(Self::Rin),
            12 => Some(Self::Rreserved1),
            13 => Some(Self::Rreserved2),
            14 => Some(Self::Rflags),
            15 => Some(Self::Rpc),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Condition {
    Unconditional = 0,
    Greater = 1,
    Less = 2,
    Equal = 3,
    NotEqual = 4,
    Overflow = 5,
    Even = 6,
    Carry = 7,
}

impl Condition {
    fn encode(&self) -> u8 {
        *self as u8
    }

    fn from_num(n: u8) -> Option<Self> {
        match n {
            0 => Some(Self::Unconditional),
            1 => Some(Self::Greater),
            2 => Some(Self::Less),
            3 => Some(Self::Equal),
            4 => Some(Self::NotEqual),
            5 => Some(Self::Overflow),
            6 => Some(Self::Even),
            7 => Some(Self::Carry),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum MemoryOperation {
    Load,
    Store,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum CarryOperation {
    WithCarry,
    WithoutCarry,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ArithmeticOperation {
    Add,
    Sub,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum BranchType {
    Relative,
    Absolute,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Instruction {
    Arithmetic {
        op: ArithmeticOperation,
        carry: CarryOperation,
        src1: ReducedRegister,
        src2: Register,
        dst: Register,
    },
    Move {
        condition: Condition,
        set_flags: bool,
        src: Register,
        dst: Register,
    },
    Branch {
        address: u8,
        branch_type: BranchType,
        condition: Condition,
    },
}

macro_rules! shorthand {
    ($name: ident ($($param: ident: $ty: ty),*) -> $variant:ident { $($fieldname: ident: $value: expr),* }) => {
        pub const fn $name($($param: $ty),*) -> Instruction {
            Instruction::$variant {
                $(
                    $fieldname: $value,
                )*
                $(
                    $param
                ),*
            }
        }
    };
    ($name: ident ($($param: ident: $ty: ty),*) no default -> $variant:ident { $($fieldname: ident: $value: expr),* }) => {
        pub const fn $name($($param: $ty),*) -> Instruction {
            Instruction::$variant {
                $(
                    $fieldname: $value,
                )*
            }
        }
    };
}

pub mod shorthands {
    use super::*;

    shorthand!(nop() -> Move {set_flags: false, condition: Condition::Unconditional, src: Register::Rnull, dst: Register::Rnull});

    shorthand!(add(src1: ReducedRegister, src2: Register, dst: Register) -> Arithmetic {op: ArithmeticOperation::Add, carry: CarryOperation::WithoutCarry});
    shorthand!(add_carry(src1: ReducedRegister, src2: Register, dst: Register) -> Arithmetic {op: ArithmeticOperation::Add, carry: CarryOperation::WithCarry});
    shorthand!(inc(src1: ReducedRegister, dst: Register) -> Arithmetic {op: ArithmeticOperation::Add, carry: CarryOperation::WithoutCarry, src2: Register::Rone});

    shorthand!(sub(src1: ReducedRegister, src2: Register, dst: Register) -> Arithmetic {op: ArithmeticOperation::Sub, carry: CarryOperation::WithoutCarry});
    shorthand!(sub_carry(src1: ReducedRegister, src2: Register, dst: Register) -> Arithmetic {op: ArithmeticOperation::Sub, carry: CarryOperation::WithCarry});
    shorthand!(dec(src1: ReducedRegister, dst: Register) -> Arithmetic {op: ArithmeticOperation::Sub, carry: CarryOperation::WithoutCarry, src2: Register::Rone});

    shorthand!(cmp(src1: ReducedRegister, src2: Register) -> Arithmetic {op: ArithmeticOperation::Sub, carry: CarryOperation::WithoutCarry, dst: Register::Rnull});
    shorthand!(cmp_carry(src1: ReducedRegister, src2: Register) -> Arithmetic {op: ArithmeticOperation::Sub, carry: CarryOperation::WithCarry, dst: Register::Rnull});
    shorthand!(cmp_0(src1: ReducedRegister) -> Arithmetic {op: ArithmeticOperation::Sub, carry: CarryOperation::WithoutCarry, src2: Register::Rnull, dst: Register::Rnull});
    shorthand!(cmp_1(src1: ReducedRegister) -> Arithmetic {op: ArithmeticOperation::Sub, carry: CarryOperation::WithoutCarry, src2: Register::Rone, dst: Register::Rnull});

    shorthand!(mov(src: Register, dst: Register) -> Move {set_flags: false, condition: Condition::Unconditional});
    shorthand!(cmoveq(src: Register, dst: Register) -> Move {set_flags: false, condition: Condition::Equal});
    shorthand!(cmovneq(src: Register, dst: Register) -> Move {set_flags: false, condition: Condition::NotEqual});

    shorthand!(jmp(address: u8) -> Branch {branch_type: BranchType::Absolute, condition: Condition::Unconditional});
    shorthand!(jmp_rel(address: i8) no default -> Branch {branch_type: BranchType::Relative, address: address as u8, condition: Condition::Unconditional});

    shorthand!(jeq(address: u8) -> Branch {branch_type: BranchType::Absolute, condition: Condition::Equal});
    shorthand!(jeq_rel(address: i8) no default -> Branch {branch_type: BranchType::Relative, address: address as u8, condition: Condition::Equal});
}

impl Instruction {
    pub fn encode(&self) -> u16 {
        match *self {
            Instruction::Arithmetic { op, carry, src1, src2, dst } => {
                let opcode: u16 = 0b001_0_0_000_0000_0000;
                let op: u16 = if op == ArithmeticOperation::Add {0b000_1_0_000_0000_0000} else {0b000_0_0_000_0000_0000};
                let carry: u16 = if carry == CarryOperation::WithCarry {0b000_0_1_000_0000_0000} else {0b000_0_0_000_0000_0000};

                let src1 = (src1.encode() as u16) << 8;
                let src2 = (src2.encode() as u16) << 4;
                let dst = dst.encode() as u16;

                opcode | op | carry | src1 | src2 | dst
            }
            Instruction::Move { condition, set_flags, src, dst } => {
                let opcode: u16 = 0b100_0000_0_0000_0000;
                let set_flags: u16 = if set_flags {0b000_0000_1_0000_0000} else {0b000_0000_0_0000_0000};

                let condition = (condition.encode() as u16) << 9;
                let src = (src.encode() as u16) << 4;
                let dst = dst.encode() as u16;

                opcode | condition | set_flags | src | dst
            },
            Instruction::Branch { address, branch_type, condition } => {
                let opcode: u16 = 0b101_0000_0_0000_0000;
                let branch_type: u16 = if branch_type == BranchType::Relative {0b000_0000_1_0000_0000} else {0b000_0000_0_0000_0000};

                let condition = (condition.encode() as u16) << 9;

                opcode | condition | branch_type | address as u16
            },
        }
    }
}

macro_rules! program {
    ($($instruction: ident $($param: expr),*);* $(;)?) => {
        {
            use $crate::instruction::shorthands::*;
            use $crate::instruction::BranchType::*;
            use $crate::instruction::Condition::*;
            use $crate::instruction::Register::*;
            use $crate::instruction::ReducedRegister::*;

            let mut program = Vec::new();
            $(
                program.push($instruction ($($param),*) .encode() );
            )*
            program
        }
    };
}