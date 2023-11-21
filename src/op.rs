use std::fmt::{*, self};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FunctionalUnit {
    ALU,
    EffectAddr,
    FPUMul,
    FPUAdd,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Register {
    GP(u64),
    FP(u64),
}

impl Register {
    fn parse_gp(reg: &str) -> Option<Self> {
        let reg = reg.trim();

        if reg.starts_with("x") {
            let reg = reg.trim_start_matches("x");
            if let Ok(reg) = reg.parse::<u64>() {
                return Some(Register::GP(reg));
            }
        }

        None
    }

    fn parse_fp(reg: &str) -> Option<Self> {
        let reg = reg.trim();

        if reg.starts_with("f") {
            let reg = reg.trim_start_matches("f");
            if let Ok(reg) = reg.parse::<u64>() {
                return Some(Register::FP(reg));
            }
        }

        None
    }

    fn parse(reg: &str) -> Option<Self> {
        Self::parse_gp(reg).or_else(|| Self::parse_fp(reg))
    }
}

impl From<Operand> for Register {
    fn from(op: Operand) -> Self {
        match op {
            Operand::Register(reg) => reg,
            _ => panic!("Operand \"{op}\" is not a register"),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Register::GP(r) => write!(f, "x{}", r),
            Register::FP(r) => write!(f, "f{}", r),
        }
    }
}

impl Debug for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Register::GP(r) => write!(f, "x{}", r),
            Register::FP(r) => write!(f, "f{}", r),
        }
    }
}

const GLOBAL_NAME_SIZE: usize = 64;

#[derive(Clone, Copy)]
pub enum Operand {
    Immediate(u64),
    Register(Register),
    Indirect(Register, u64),
    Global([char; GLOBAL_NAME_SIZE]),
    None,
}

impl Operand {
    fn global(name: &str) -> Self {
        let mut chars = name.chars().collect::<Vec<char>>();
        // Convert into array
        let mut result = ['\0'; GLOBAL_NAME_SIZE];
        for i in 0..chars.len() {
            result[i] = chars[i];
        }
        Self::Global(result)
    }

    pub const fn is_reg(&self) -> bool {
        matches!(self, Operand::Register(_))
    }

    pub fn as_reg(&self) -> Register {
        match self {
            Operand::Register(reg) => *reg,
            _ => panic!("Operand \"{self}\" is not a register"),
        }
    }

    /// Get the registers this operation depends on
    pub fn dep_reg(&self) -> Option<Register> {
        match self {
            Operand::Register(reg) => Some(*reg),
            Operand::Indirect(reg, _) => Some(*reg),
            _ => None,
        }
    }

    fn parse(reg: &str) -> Self {
        let reg = reg.trim();
        if let Some(reg) = Register::parse(reg) {
            return Operand::Register(reg);
        }

        if reg.starts_with("0x") {
            let reg = reg.trim_start_matches("0x").trim();
            if let Ok(reg) = u64::from_str_radix(reg, 16) {
                return Operand::Immediate(reg);
            }
        }

        if let Ok(reg) = reg.parse::<u64>() {
            return Operand::Immediate(reg);
        }

        // Parse an indirect address
        // Get the string until the `(` character
        let mut operand = reg.split('(');
        if operand.clone().count() == 1 {
            return Self::global(operand.next().unwrap());
        }


        let offset = operand.next().unwrap().trim();
        let offset = if let Ok(offset) = offset.parse::<u64>() {
            offset
        } else {
            panic!("Could not parse offset \"{offset}\"");
        };

        // Get the string until the `)` character
        let mut operand = operand.next().unwrap().split(')');
        let reg = operand.next().unwrap().trim();
        let reg = if let Some(reg) = Register::parse(reg) {
            reg
        } else {
            panic!("Could not parse register \"{reg}\"");
        };

        return Operand::Indirect(reg, offset);
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Immediate(imm) => write!(f, "{}", imm),
            Operand::Register(reg) => write!(f, "{}", reg),
            Operand::Indirect(reg, imm) => write!(f, "{imm}({reg})"),
            Operand::Global(name) => {
                for c in name.iter() {
                    if *c == '\0' {
                        break;
                    }
                    write!(f, "{}", c)?;
                }
                Ok(())
            },
            Operand::None => write!(f, ""),
        }
    }
}

impl Debug for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Immediate(imm) => write!(f, "{}", imm),
            Operand::Register(reg) => write!(f, "{}", reg),
            Operand::Indirect(reg, imm) => write!(f, "{imm}({reg})"),
            Operand::Global(name) => write!(f, "{}", name.iter().collect::<String>()),
            Operand::None => write!(f, ""),
        }
    }
}

#[derive(Clone, Copy)]
pub enum RiscVOp {
    LoadWord(Operand, Operand, u64),
    StoreWord(Operand, Operand, u64),
    LoadFloat(Operand, Operand, u64),
    StoreFloat(Operand, Operand, u64),
    Add(Operand, Operand, Operand),
    Sub(Operand, Operand, Operand),
    BranchEqual(Operand, Operand, Operand),
    BranchNotEqual(Operand, Operand, Operand),
    FloatAdd(Operand, Operand, Operand),
    FloatSub(Operand, Operand, Operand),
    FloatMul(Operand, Operand, Operand),
    FloatDiv(Operand, Operand, Operand),
}

impl RiscVOp {
    pub fn functional_unit(&self) -> FunctionalUnit {
        if self.is_branch() {
            FunctionalUnit::EffectAddr
        } else if self.is_alu() {
            FunctionalUnit::ALU
        } else if self.is_fp_add() {
            FunctionalUnit::FPUAdd
        } else if self.is_fp_mul() {
            FunctionalUnit::FPUMul
        } else if self.is_data_transfer() {
            FunctionalUnit::EffectAddr
        } else {
            panic!("Unknown functional unit for operation \"{self}\"");
        }
    }

    pub fn accesses_memory(&self) -> bool {
        self.is_load()
    }

    pub fn writes_back(&self) -> bool {
        self.is_alu() || self.is_fp() || self.is_load() && !self.is_branch()
    }

    pub fn parse(line: &str) -> Self {
        // Get the string until the ` ` character
        let mut split_by_space = line.split_whitespace();
        let op = split_by_space.next().unwrap();
        // Now get all the arguments separated by `,`
        let dst = split_by_space.next().unwrap().split(',').next().unwrap();


        let mut args = line.split(',');
        args.next();
        // Get the destination
        // Get the first argument
        let arg1 = args.next().unwrap();
        // Get the second argument
        let arg2 = args.next();

        // Get optional address denoted with colon
        let mut addr = line.split(':');
        addr.next();
        let addr = addr.next().unwrap_or("0");
        let addr = if let Ok(addr) = addr.parse::<u64>() {
            addr
        } else {
            panic!("Could not parse address \"{addr}\"");
        };


        match op {
            "add" => RiscVOp::Add(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "sub" => RiscVOp::Sub(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "lw" => RiscVOp::LoadWord(Operand::parse(dst), Operand::parse(arg1), addr),
            "sw" => RiscVOp::StoreWord(Operand::parse(dst), Operand::parse(arg1), addr),
            "beq" => RiscVOp::BranchEqual(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "bne" => RiscVOp::BranchNotEqual(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "flw" => RiscVOp::LoadFloat(Operand::parse(dst), Operand::parse(arg1), addr),
            "fsw" => RiscVOp::StoreFloat(Operand::parse(dst), Operand::parse(arg1), addr),
            "fadd" => RiscVOp::FloatAdd(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "fsub" => RiscVOp::FloatSub(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "fmul" => RiscVOp::FloatMul(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "fdiv" => RiscVOp::FloatDiv(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "fadd.s" => RiscVOp::FloatAdd(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "fsub.s" => RiscVOp::FloatSub(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "fmul.s" => RiscVOp::FloatMul(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            "fdiv.s" => RiscVOp::FloatDiv(Operand::parse(dst), Operand::parse(arg1), Operand::parse(arg2.unwrap())),
            _ => unimplemented!()
        }
    }

    pub const fn is_load(&self) -> bool {
        matches!(self, RiscVOp::LoadWord(_, _, _) | RiscVOp::LoadFloat(_, _, _))
    }

    pub const fn is_store(&self) -> bool {
        matches!(self, RiscVOp::StoreWord(_, _, _) | RiscVOp::StoreFloat(_, _, _))
    }

    pub const fn is_data_transfer(&self) -> bool {
        matches!(self, RiscVOp::LoadWord(_, _, _) | RiscVOp::StoreWord(_, _, _)
            | RiscVOp::LoadFloat(_, _, _) | RiscVOp::StoreFloat(_, _, _))
    }

    pub const fn is_branch(&self) -> bool {
        matches!(self, RiscVOp::BranchEqual(_, _, _) | RiscVOp::BranchNotEqual(_, _, _))
    }

    pub const fn is_alu(&self) -> bool {
        matches!(self, RiscVOp::Add(_, _, _) | RiscVOp::Sub(_, _, _))
    }

    pub const fn is_fp(&self) -> bool {
        matches!(self, RiscVOp::FloatAdd(_, _, _) | RiscVOp::FloatSub(_, _, _)
            | RiscVOp::FloatMul(_, _, _) | RiscVOp::FloatDiv(_, _, _))
    }

    pub const fn is_fp_add(&self) -> bool {
        matches!(self, RiscVOp::FloatAdd(_, _, _) | RiscVOp::FloatSub(_, _, _))
    }

    pub const fn is_fp_mul(&self) -> bool {
        matches!(self, RiscVOp::FloatMul(_, _, _) | RiscVOp::FloatDiv(_, _, _))
    }
    pub const fn is_fp_div(&self) -> bool {
        matches!(self, RiscVOp::FloatDiv(_, _, _))
    }

    pub fn addr(&self) -> Option<u64> {
        match self {
            RiscVOp::LoadWord(_, _, addr) => Some(*addr),
            RiscVOp::StoreWord(_, _, addr) => Some(*addr),
            RiscVOp::LoadFloat(_, _, addr) => Some(*addr),
            RiscVOp::StoreFloat(_, _, addr) => Some(*addr),
            _ => None,
        }
    }

    pub fn dst(&self) -> Option<Operand> {
        Some(match self {
            RiscVOp::LoadWord(dst, _, _) => *dst,
            RiscVOp::StoreWord(_, _, _) => None?,
            RiscVOp::LoadFloat(dst, _, _) => *dst,
            RiscVOp::StoreFloat(_, _, _) => None?,
            RiscVOp::Add(dst, _, _) => *dst,
            RiscVOp::Sub(dst, _, _) => *dst,
            RiscVOp::BranchEqual(dst, _, _) => *dst,
            RiscVOp::BranchNotEqual(dst, _, _) => *dst,
            RiscVOp::FloatAdd(dst, _, _) => *dst,
            RiscVOp::FloatSub(dst, _, _) => *dst,
            RiscVOp::FloatMul(dst, _, _) => *dst,
            RiscVOp::FloatDiv(dst, _, _) => *dst,
        })
    }

    pub fn src1(&self) -> Operand {
        match self {
            RiscVOp::LoadWord(_, src, _) => *src,
            RiscVOp::StoreWord(_, src, _) => *src,
            RiscVOp::LoadFloat(_, src, _) => *src,
            RiscVOp::StoreFloat(_, src, _) => *src,
            RiscVOp::Add(_, src1, _) => *src1,
            RiscVOp::Sub(_, src1, _) => *src1,
            RiscVOp::BranchEqual(_, src1, _) => *src1,
            RiscVOp::BranchNotEqual(_, src1, _) => *src1,
            RiscVOp::FloatAdd(_, src1, _) => *src1,
            RiscVOp::FloatSub(_, src1, _) => *src1,
            RiscVOp::FloatMul(_, src1, _) => *src1,
            RiscVOp::FloatDiv(_, src1, _) => *src1,
        }
    }

    pub fn src2(&self) -> Operand {
        use Operand::*;
        match self {
            RiscVOp::LoadWord(_, _, _) => None,
            RiscVOp::StoreWord(_, _, _) => None,
            RiscVOp::LoadFloat(_, _, _) => None,
            RiscVOp::StoreFloat(_, _, _) => None,
            RiscVOp::Add(_, _, src2) => *src2,
            RiscVOp::Sub(_, _, src2) => *src2,
            RiscVOp::BranchEqual(_, _, src2) => *src2,
            RiscVOp::BranchNotEqual(_, _, src2) => *src2,
            RiscVOp::FloatAdd(_, _, src2) => *src2,
            RiscVOp::FloatSub(_, _, src2) => *src2,
            RiscVOp::FloatMul(_, _, src2) => *src2,
            RiscVOp::FloatDiv(_, _, src2) => *src2,
        }
    }
}

impl Display for RiscVOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RiscVOp::LoadWord(dst, src, addr)            => write!(f, "lw     {},{}:{addr}", dst, src),
            RiscVOp::StoreWord(dst, src, addr)           => write!(f, "sw     {},{}:{addr}", dst, src),
            RiscVOp::Add(dst, src1, src2)            => write!(f, "add    {},{},{}", dst, src1, src2),
            RiscVOp::Sub(dst, src1, src2)            => write!(f, "sub    {},{},{}", dst, src1, src2),
            RiscVOp::BranchEqual(dst, src1, src2)    => write!(f, "beq    {},{},{}", dst, src1, src2),
            RiscVOp::BranchNotEqual(dst, src1, src2) => write!(f, "bne    {},{},{}", dst, src1, src2),
            RiscVOp::LoadFloat(dst, src, addr)           => write!(f, "flw    {},{}:{addr}", dst, src),
            RiscVOp::StoreFloat(dst, src, addr)          => write!(f, "fsw    {},{}:{addr}", dst, src),
            RiscVOp::FloatAdd(dst, src1, src2)       => write!(f, "fadd.s {},{},{}", dst, src1, src2),
            RiscVOp::FloatSub(dst, src1, src2)       => write!(f, "fsub.s {},{},{}", dst, src1, src2),
            RiscVOp::FloatMul(dst, src1, src2)       => write!(f, "fmul.s {},{},{}", dst, src1, src2),
            RiscVOp::FloatDiv(dst, src1, src2)       => write!(f, "fdiv.s {},{},{}", dst, src1, src2),
        }
    }
}

impl Debug for RiscVOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}