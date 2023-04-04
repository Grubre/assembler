use std::{
    fs::read_to_string,
    io,
    path::Path,
    str::FromStr,
};

#[derive(Debug)]
pub enum ConfigParseKind {
    OpenFile(io::Error),
    ParseArg(String),
    Split,
}

impl ConfigParseKind {
    pub fn complete(self, line_num: usize) -> ConfigParseError {
        ConfigParseError {
            kind: self,
            line_num,
        }
    }
}

#[derive(Debug)]
pub struct ConfigParseError {
    pub kind: ConfigParseKind,
    pub line_num: usize,
}

impl From<io::Error> for ConfigParseError {
    fn from(value: io::Error) -> Self {
        ConfigParseError {
            kind: ConfigParseKind::OpenFile(value),
            line_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArgDef {
    Mem,
    Const,
    A,
    B,
    F,
}

impl FromStr for ArgDef {
    type Err = ConfigParseKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MEM" => Ok(ArgDef::Mem),
            "CONST" => Ok(ArgDef::Const),
            "A" => Ok(ArgDef::A),
            "B" => Ok(ArgDef::B),
            "F" => Ok(ArgDef::F),
            s => Err(ConfigParseKind::ParseArg(s.to_string())),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InstructionDef {
    pub mnem: String,
    pub args_def: Vec<ArgDef>,
    pub mnem_full: String,
    pub binary: String,
}

impl FromStr for InstructionDef {
    type Err = ConfigParseKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<_> = s.split(',').collect();
        let [instr,mnem_full,binary] = &split[..] else { return Err(ConfigParseKind::Split);};

        let instr_split: Vec<_> = instr.split_whitespace().collect();
        let [mnem, args @ ..] = &instr_split[..] else {return Err(ConfigParseKind::Split);};

        let args_def: Vec<_> = args
            .iter()
            .map(|str| str.trim().parse::<ArgDef>())
            .collect::<Result<_, _>>()?;

        let mnem_full = mnem_full.trim().to_string();
        let binary = binary.trim().to_string();
        let mnem = mnem.to_string();

        Ok(InstructionDef {
            mnem,
            args_def,
            mnem_full,
            binary,
        })
    }
}

#[derive(Debug)]
pub struct Config(pub Vec<InstructionDef>);

impl Config {
    pub fn read_from_file(file_path: impl AsRef<Path>) -> Result<Self, ConfigParseError> {
        let content = read_to_string(file_path)?;
        let defs = content
            .lines()
            .enumerate()
            .map(|(line, str)| {
                str.parse::<InstructionDef>()
                    .map_err(|err| err.complete(line + 1))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self(defs))
    }

    pub const fn const_read_from_file() -> &'static str {
        include_str!("../config.cfg")
    }

    pub fn create_mnem_regex(&self) -> String {
        //Example: r"(?i)^(NOP|MOV|PUSH|POP|JMP|ADD|SUB|OR|AND|NEG|INV|SHR|SHL|CMP|HALT)"

        let mnems = self
            .0
            .iter()
            .map(|instr| instr.mnem.as_str())
            .collect::<Vec<_>>()
            .join("|");

        format!(r"(?i)^({})", mnems)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_single_instr() {
        let test = "MOV MEM B,      MOVABSB	    ,c0001000";
        let parsed: InstructionDef = test.parse().unwrap();

        assert_eq!(
            parsed,
            InstructionDef {
                mnem: "MOV".to_string(),
                args_def: vec![ArgDef::Mem, ArgDef::B],
                mnem_full: "MOVABSB".to_string(),
                binary: "c0001000".to_string()
            }
        )
    }

    #[test]
    fn test_regex_instr() {
        let test = Config(vec![
            InstructionDef {
                mnem: "MOV".to_string(),
                args_def: vec![ArgDef::Mem, ArgDef::B],
                mnem_full: "MOVABSB".to_string(),
                binary: "c0001000".to_string(),
            },
            InstructionDef {
                mnem: "NOP".to_string(),
                args_def: Vec::new(),
                mnem_full: "NOP".to_string(),
                binary: "c0000000".to_string(),
            },
        ]);

        assert_eq!(test.create_mnem_regex(), r"(?i)^(MOV|NOP)")
    }
}
