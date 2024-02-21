use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Operand {
    Register(Register),
    Mem8,
    Mem16,
    Const,
    Stc,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Register {
    A,
    B,
    F,
    T,
    TL,
    TH,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Mnemonic {
    Mov,
    Halt,
    Movat,
    And,
    Or,
    Skip1,
    Inv,
    Xor,
    Cmp,
    Shl,
    Nop,
    Add,
    Skip,
    Jmpimm,
    Jmprel,
    Pop,
    Push,
    Neg,
    Sub,
    Shr,
    Inc,
    Skip2,
    Clr,
    Dec,
    Div2,
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
        if s == "MEM8" || s == "MEMZP" {
            return Ok(Self::Mem8);
        }
        if s == "MEM" || s == "MEM16" {
            return Ok(Self::Mem16);
        }

        if s == "STC" {
            return Ok(Self::Stc);
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
            "T" => Ok(Register::T),
            "TL" => Ok(Register::TL),
            "TH" => Ok(Register::TH),
            _ => Err(()),
        }
    }
}

impl FromStr for Mnemonic {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MOV" => Ok(Mnemonic::Mov),
            "HALT" => Ok(Mnemonic::Halt),
            "MOVAT" => Ok(Mnemonic::Movat),
            "AND" => Ok(Mnemonic::And),
            "OR" => Ok(Mnemonic::Or),
            "SKIP1" => Ok(Mnemonic::Skip1),
            "INV" => Ok(Mnemonic::Inv),
            "XOR" => Ok(Mnemonic::Xor),
            "CMP" => Ok(Mnemonic::Cmp),
            "SHL" => Ok(Mnemonic::Shl),
            "NOP" => Ok(Mnemonic::Nop),
            "ADD" => Ok(Mnemonic::Add),
            "SKIP" => Ok(Mnemonic::Skip),
            "JMPIMM" => Ok(Mnemonic::Jmpimm),
            "JMPREL" => Ok(Mnemonic::Jmprel),
            "POP" => Ok(Mnemonic::Pop),
            "PUSH" => Ok(Mnemonic::Push),
            "NEG" => Ok(Mnemonic::Neg),
            "SUB" => Ok(Mnemonic::Sub),
            "SHR" => Ok(Mnemonic::Shr),
            "INC" => Ok(Mnemonic::Inc),
            "SKIP2" => Ok(Mnemonic::Skip2),
            "CLR" => Ok(Mnemonic::Clr),
            "DEC" => Ok(Mnemonic::Dec),
            "DIV2" => Ok(Mnemonic::Div2),
            _ => Err(()),
        }
    }
}
