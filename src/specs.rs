use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Operand {
    Register(Register),
    Mem,
    Const,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Register {
    A,
    B,
    F,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Mnemonic {
    Mov,
    Push,
    Pop,
    Jmp,
    Add,
    Sub,
    Or,
    And,
    Neg,
    Inv,
    Shr,
    Shl,
    Cmp,
    Halt,
    Nop,
}

impl FromStr for Operand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let register = Register::from_str(s);
        if let Ok(register) = register {
            return Ok(Self::Register(register));
        }

        if s == "CONST" {
            return Ok(Self::Const);
        }
        if s == "MEM" {
            return Ok(Self::Mem);
        }

        Err(())
    }
}

impl FromStr for Register {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(Register::A),
            "B" => Ok(Register::B),
            "F" => Ok(Register::F),
            _ => Err(()),
        }
    }
}

impl FromStr for Mnemonic {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MOV" => Ok(Mnemonic::Mov),
            "PUSH" => Ok(Mnemonic::Push),
            "POP" => Ok(Mnemonic::Pop),
            "JMP" => Ok(Mnemonic::Jmp),
            "ADD" => Ok(Mnemonic::Add),
            "SUB" => Ok(Mnemonic::Sub),
            "OR" => Ok(Mnemonic::Or),
            "AND" => Ok(Mnemonic::And),
            "NEG" => Ok(Mnemonic::Neg),
            "INV" => Ok(Mnemonic::Inv),
            "SHR" => Ok(Mnemonic::Shr),
            "SHL" => Ok(Mnemonic::Shl),
            "CMP" => Ok(Mnemonic::Cmp),
            "HALT" => Ok(Mnemonic::Halt),
            "NOP" => Ok(Mnemonic::Nop),
            _ => Err(()),
        }
    }
}
