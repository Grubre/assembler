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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Mnemonic {
    name: String,
}

impl Mnemonic {
    pub fn new(name: String) -> Self {
        Self { name }
    }
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
