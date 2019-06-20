// This file is based on parity-wasm from parity, MIT & Apache Licensed
use pwasm_std;
use pwasm_std::vec::Vec;
use pwasm_std::Box;

use crate::io;
use crate::{Deserialize,
    Uint8, VarUint32, CountedList,
    Uint32, Uint64,
    VarInt32, VarInt64,
};
use crate::types::{BlockType};
use crate::serialization::{Error};

/// List of instructions (usually inside a block section).
#[derive(Debug, Clone, PartialEq)]
pub struct Instructions(Vec<Instruction>);

impl Instructions {
    /// New list of instructions from vector of instructions.
    pub fn new(elements: Vec<Instruction>) -> Self {
        Instructions(elements)
    }

    /// Empty expression with only `Instruction::End` instruction.
    pub fn empty() -> Self {
        Instructions(vec![Instruction::End])
    }

    /// List of individual instructions.
    pub fn elements(&self) -> &[Instruction] { &self.0 }

    /// Individual instructions, mutable.
    pub fn elements_mut(&mut self) -> &mut Vec<Instruction> { &mut self.0 }
}

impl Deserialize for Instructions {
    type Error = Error;

    fn deserialize<R: io::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut instructions = Vec::new();
        let mut block_count = 1usize;

        loop {
            let instruction = Instruction::deserialize(reader)?;
            if instruction.is_terminal() {
                block_count -= 1;
            } else if instruction.is_block() {
                block_count = block_count.checked_add(1).ok_or(Error::Other("too many instructions"))?;
            }

            instructions.push(instruction);
            if block_count == 0 {
                break;
            }
        }

        Ok(Instructions(instructions))
    }
}

/// Initialization expression.
#[derive(Debug, Clone, PartialEq)]
pub struct InitExpr(Vec<Instruction>);

impl InitExpr {
    /// New initialization expression from instruction list.
    ///
    /// `code` must end with the `Instruction::End` instruction!
    pub fn new(code: Vec<Instruction>) -> Self {
        InitExpr(code)
    }

    /// Empty expression with only `Instruction::End` instruction.
    pub fn empty() -> Self {
        InitExpr(vec![Instruction::End])
    }

    /// List of instructions used in the expression.
    pub fn code(&self) -> &[Instruction] {
        &self.0
    }

    /// List of instructions used in the expression.
    pub fn code_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.0
    }
}

impl Deserialize for InitExpr {
    type Error = Error;

    fn deserialize<R: io::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut instructions = Vec::new();

        loop {
            let instruction = Instruction::deserialize(reader)?;
            let is_terminal = instruction.is_terminal();
            instructions.push(instruction);
            if is_terminal {
                break;
            }
        }

        Ok(InitExpr(instructions))
    }
}

/// Instruction.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Instruction {
    Unreachable,
    Nop,
    Block(BlockType),
    Loop(BlockType),
    If(BlockType),
    Else,
    End,
    Br(u32),
    BrIf(u32),
    BrTable(Box<BrTableData>),
    Return,

    Call(u32),
    CallIndirect(u32, u8),

    Drop,
    Select,

    GetLocal(u32),
    SetLocal(u32),
    TeeLocal(u32),
    GetGlobal(u32),
    SetGlobal(u32),

    // All store/load instructions operate with 'memory immediates'
    // which represented here as (flag, offset) tuple
    I32Load(u32, u32),
    I64Load(u32, u32),
    F32Load(u32, u32),
    F64Load(u32, u32),
    I32Load8S(u32, u32),
    I32Load8U(u32, u32),
    I32Load16S(u32, u32),
    I32Load16U(u32, u32),
    I64Load8S(u32, u32),
    I64Load8U(u32, u32),
    I64Load16S(u32, u32),
    I64Load16U(u32, u32),
    I64Load32S(u32, u32),
    I64Load32U(u32, u32),
    I32Store(u32, u32),
    I64Store(u32, u32),
    F32Store(u32, u32),
    F64Store(u32, u32),
    I32Store8(u32, u32),
    I32Store16(u32, u32),
    I64Store8(u32, u32),
    I64Store16(u32, u32),
    I64Store32(u32, u32),

    CurrentMemory(u8),
    GrowMemory(u8),

    I32Const(i32),
    I64Const(i64),
    F32Const(u32),
    F64Const(u64),

    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,

    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,

    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,

    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,

    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,

    I32WrapI64,
    I32TruncSF32,
    I32TruncUF32,
    I32TruncSF64,
    I32TruncUF64,
    I64ExtendSI32,
    I64ExtendUI32,
    I64TruncSF32,
    I64TruncUF32,
    I64TruncSF64,
    I64TruncUF64,
    F32ConvertSI32,
    F32ConvertUI32,
    F32ConvertSI64,
    F32ConvertUI64,
    F32DemoteF64,
    F64ConvertSI32,
    F64ConvertUI32,
    F64ConvertSI64,
    F64ConvertUI64,
    F64PromoteF32,

    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,

    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,

    AtomicWake(MemArg),
    I32AtomicWait(MemArg),
    I64AtomicWait(MemArg),

    I32AtomicLoad(MemArg),
    I64AtomicLoad(MemArg),
    I32AtomicLoad8u(MemArg),
    I32AtomicLoad16u(MemArg),
    I64AtomicLoad8u(MemArg),
    I64AtomicLoad16u(MemArg),
    I64AtomicLoad32u(MemArg),
    I32AtomicStore(MemArg),
    I64AtomicStore(MemArg),
    I32AtomicStore8u(MemArg),
    I32AtomicStore16u(MemArg),
    I64AtomicStore8u(MemArg),
    I64AtomicStore16u(MemArg),
    I64AtomicStore32u(MemArg),

    I32AtomicRmwAdd(MemArg),
    I64AtomicRmwAdd(MemArg),
    I32AtomicRmwAdd8u(MemArg),
    I32AtomicRmwAdd16u(MemArg),
    I64AtomicRmwAdd8u(MemArg),
    I64AtomicRmwAdd16u(MemArg),
    I64AtomicRmwAdd32u(MemArg),

    I32AtomicRmwSub(MemArg),
    I64AtomicRmwSub(MemArg),
    I32AtomicRmwSub8u(MemArg),
    I32AtomicRmwSub16u(MemArg),
    I64AtomicRmwSub8u(MemArg),
    I64AtomicRmwSub16u(MemArg),
    I64AtomicRmwSub32u(MemArg),

    I32AtomicRmwAnd(MemArg),
    I64AtomicRmwAnd(MemArg),
    I32AtomicRmwAnd8u(MemArg),
    I32AtomicRmwAnd16u(MemArg),
    I64AtomicRmwAnd8u(MemArg),
    I64AtomicRmwAnd16u(MemArg),
    I64AtomicRmwAnd32u(MemArg),

    I32AtomicRmwOr(MemArg),
    I64AtomicRmwOr(MemArg),
    I32AtomicRmwOr8u(MemArg),
    I32AtomicRmwOr16u(MemArg),
    I64AtomicRmwOr8u(MemArg),
    I64AtomicRmwOr16u(MemArg),
    I64AtomicRmwOr32u(MemArg),

    I32AtomicRmwXor(MemArg),
    I64AtomicRmwXor(MemArg),
    I32AtomicRmwXor8u(MemArg),
    I32AtomicRmwXor16u(MemArg),
    I64AtomicRmwXor8u(MemArg),
    I64AtomicRmwXor16u(MemArg),
    I64AtomicRmwXor32u(MemArg),

    I32AtomicRmwXchg(MemArg),
    I64AtomicRmwXchg(MemArg),
    I32AtomicRmwXchg8u(MemArg),
    I32AtomicRmwXchg16u(MemArg),
    I64AtomicRmwXchg8u(MemArg),
    I64AtomicRmwXchg16u(MemArg),
    I64AtomicRmwXchg32u(MemArg),

    I32AtomicRmwCmpxchg(MemArg),
    I64AtomicRmwCmpxchg(MemArg),
    I32AtomicRmwCmpxchg8u(MemArg),
    I32AtomicRmwCmpxchg16u(MemArg),
    I64AtomicRmwCmpxchg8u(MemArg),
    I64AtomicRmwCmpxchg16u(MemArg),
    I64AtomicRmwCmpxchg32u(MemArg),

    V128Const(Box<[u8; 16]>),
    V128Load(MemArg),
    V128Store(MemArg),
    I8x16Splat,
    I16x8Splat,
    I32x4Splat,
    I64x2Splat,
    F32x4Splat,
    F64x2Splat,
    I8x16ExtractLaneS(u8),
    I8x16ExtractLaneU(u8),
    I16x8ExtractLaneS(u8),
    I16x8ExtractLaneU(u8),
    I32x4ExtractLane(u8),
    I64x2ExtractLane(u8),
    F32x4ExtractLane(u8),
    F64x2ExtractLane(u8),
    I8x16ReplaceLane(u8),
    I16x8ReplaceLane(u8),
    I32x4ReplaceLane(u8),
    I64x2ReplaceLane(u8),
    F32x4ReplaceLane(u8),
    F64x2ReplaceLane(u8),
    V8x16Shuffle(Box<[u8; 16]>),
    I8x16Add,
    I16x8Add,
    I32x4Add,
    I64x2Add,
    I8x16Sub,
    I16x8Sub,
    I32x4Sub,
    I64x2Sub,
    I8x16Mul,
    I16x8Mul,
    I32x4Mul,
    // I64x2Mul,
    I8x16Neg,
    I16x8Neg,
    I32x4Neg,
    I64x2Neg,
    I8x16AddSaturateS,
    I8x16AddSaturateU,
    I16x8AddSaturateS,
    I16x8AddSaturateU,
    I8x16SubSaturateS,
    I8x16SubSaturateU,
    I16x8SubSaturateS,
    I16x8SubSaturateU,
    I8x16Shl,
    I16x8Shl,
    I32x4Shl,
    I64x2Shl,
    I8x16ShrS,
    I8x16ShrU,
    I16x8ShrS,
    I16x8ShrU,
    I32x4ShrS,
    I32x4ShrU,
    I64x2ShrS,
    I64x2ShrU,
    V128And,
    V128Or,
    V128Xor,
    V128Not,
    V128Bitselect,
    I8x16AnyTrue,
    I16x8AnyTrue,
    I32x4AnyTrue,
    I64x2AnyTrue,
    I8x16AllTrue,
    I16x8AllTrue,
    I32x4AllTrue,
    I64x2AllTrue,
    I8x16Eq,
    I16x8Eq,
    I32x4Eq,
    // I64x2Eq,
    F32x4Eq,
    F64x2Eq,
    I8x16Ne,
    I16x8Ne,
    I32x4Ne,
    // I64x2Ne,
    F32x4Ne,
    F64x2Ne,
    I8x16LtS,
    I8x16LtU,
    I16x8LtS,
    I16x8LtU,
    I32x4LtS,
    I32x4LtU,
    // I64x2LtS,
    // I64x2LtU,
    F32x4Lt,
    F64x2Lt,
    I8x16LeS,
    I8x16LeU,
    I16x8LeS,
    I16x8LeU,
    I32x4LeS,
    I32x4LeU,
    // I64x2LeS,
    // I64x2LeU,
    F32x4Le,
    F64x2Le,
    I8x16GtS,
    I8x16GtU,
    I16x8GtS,
    I16x8GtU,
    I32x4GtS,
    I32x4GtU,
    // I64x2GtS,
    // I64x2GtU,
    F32x4Gt,
    F64x2Gt,
    I8x16GeS,
    I8x16GeU,
    I16x8GeS,
    I16x8GeU,
    I32x4GeS,
    I32x4GeU,
    // I64x2GeS,
    // I64x2GeU,
    F32x4Ge,
    F64x2Ge,
    F32x4Neg,
    F64x2Neg,
    F32x4Abs,
    F64x2Abs,
    F32x4Min,
    F64x2Min,
    F32x4Max,
    F64x2Max,
    F32x4Add,
    F64x2Add,
    F32x4Sub,
    F64x2Sub,
    F32x4Div,
    F64x2Div,
    F32x4Mul,
    F64x2Mul,
    F32x4Sqrt,
    F64x2Sqrt,
    F32x4ConvertSI32x4,
    F32x4ConvertUI32x4,
    F64x2ConvertSI64x2,
    F64x2ConvertUI64x2,
    I32x4TruncSF32x4Sat,
    I32x4TruncUF32x4Sat,
    I64x2TruncSF64x2Sat,
    I64x2TruncUF64x2Sat,

    // https://github.com/WebAssembly/bulk-memory-operations
    MemoryInit(u32),
    MemoryDrop(u32),
    MemoryCopy,
    MemoryFill,
    TableInit(u32),
    TableDrop(u32),
    TableCopy,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub struct MemArg {
    pub align: u8,
    pub offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub struct BrTableData {
    pub table: Box<[u32]>,
    pub default: u32,
}

impl Instruction {
    /// Is this instruction starts the new block (which should end with terminal instruction).
    pub fn is_block(&self) -> bool {
        match self {
            &Instruction::Block(_) | &Instruction::Loop(_) | &Instruction::If(_) => true,
            _ => false,
        }
    }

    /// Is this instruction determines the termination of instruction sequence?
    ///
    /// `true` for `Instruction::End`
    pub fn is_terminal(&self) -> bool {
        match self {
            &Instruction::End => true,
            _ => false,
        }
    }
}

#[allow(missing_docs)]
#[allow(dead_code)]
pub mod opcodes {
    pub const UNREACHABLE: u8 = 0x00;
    pub const NOP: u8 = 0x01;
    pub const BLOCK: u8 = 0x02;
    pub const LOOP: u8 = 0x03;
    pub const IF: u8 = 0x04;
    pub const ELSE: u8 = 0x05;
    pub const END: u8 = 0x0b;
    pub const BR: u8 = 0x0c;
    pub const BRIF: u8 = 0x0d;
    pub const BRTABLE: u8 = 0x0e;
    pub const RETURN: u8 = 0x0f;
    pub const CALL: u8 = 0x10;
    pub const CALLINDIRECT: u8 = 0x11;
    pub const DROP: u8 = 0x1a;
    pub const SELECT: u8 = 0x1b;
    pub const GETLOCAL: u8 = 0x20;
    pub const SETLOCAL: u8 = 0x21;
    pub const TEELOCAL: u8 = 0x22;
    pub const GETGLOBAL: u8 = 0x23;
    pub const SETGLOBAL: u8 = 0x24;
    pub const I32LOAD: u8 = 0x28;
    pub const I64LOAD: u8 = 0x29;
    pub const F32LOAD: u8 = 0x2a;
    pub const F64LOAD: u8 = 0x2b;
    pub const I32LOAD8S: u8 = 0x2c;
    pub const I32LOAD8U: u8 = 0x2d;
    pub const I32LOAD16S: u8 = 0x2e;
    pub const I32LOAD16U: u8 = 0x2f;
    pub const I64LOAD8S: u8 = 0x30;
    pub const I64LOAD8U: u8 = 0x31;
    pub const I64LOAD16S: u8 = 0x32;
    pub const I64LOAD16U: u8 = 0x33;
    pub const I64LOAD32S: u8 = 0x34;
    pub const I64LOAD32U: u8 = 0x35;
    pub const I32STORE: u8 = 0x36;
    pub const I64STORE: u8 = 0x37;
    pub const F32STORE: u8 = 0x38;
    pub const F64STORE: u8 = 0x39;
    pub const I32STORE8: u8 = 0x3a;
    pub const I32STORE16: u8 = 0x3b;
    pub const I64STORE8: u8 = 0x3c;
    pub const I64STORE16: u8 = 0x3d;
    pub const I64STORE32: u8 = 0x3e;
    pub const CURRENTMEMORY: u8 = 0x3f;
    pub const GROWMEMORY: u8 = 0x40;
    pub const I32CONST: u8 = 0x41;
    pub const I64CONST: u8 = 0x42;
    pub const F32CONST: u8 = 0x43;
    pub const F64CONST: u8 = 0x44;
    pub const I32EQZ: u8 = 0x45;
    pub const I32EQ: u8 = 0x46;
    pub const I32NE: u8 = 0x47;
    pub const I32LTS: u8 = 0x48;
    pub const I32LTU: u8 = 0x49;
    pub const I32GTS: u8 = 0x4a;
    pub const I32GTU: u8 = 0x4b;
    pub const I32LES: u8 = 0x4c;
    pub const I32LEU: u8 = 0x4d;
    pub const I32GES: u8 = 0x4e;
    pub const I32GEU: u8 = 0x4f;
    pub const I64EQZ: u8 = 0x50;
    pub const I64EQ: u8 = 0x51;
    pub const I64NE: u8 = 0x52;
    pub const I64LTS: u8 = 0x53;
    pub const I64LTU: u8 = 0x54;
    pub const I64GTS: u8 = 0x55;
    pub const I64GTU: u8 = 0x56;
    pub const I64LES: u8 = 0x57;
    pub const I64LEU: u8 = 0x58;
    pub const I64GES: u8 = 0x59;
    pub const I64GEU: u8 = 0x5a;

    pub const F32EQ: u8 = 0x5b;
    pub const F32NE: u8 = 0x5c;
    pub const F32LT: u8 = 0x5d;
    pub const F32GT: u8 = 0x5e;
    pub const F32LE: u8 = 0x5f;
    pub const F32GE: u8 = 0x60;

    pub const F64EQ: u8 = 0x61;
    pub const F64NE: u8 = 0x62;
    pub const F64LT: u8 = 0x63;
    pub const F64GT: u8 = 0x64;
    pub const F64LE: u8 = 0x65;
    pub const F64GE: u8 = 0x66;

    pub const I32CLZ: u8 = 0x67;
    pub const I32CTZ: u8 = 0x68;
    pub const I32POPCNT: u8 = 0x69;
    pub const I32ADD: u8 = 0x6a;
    pub const I32SUB: u8 = 0x6b;
    pub const I32MUL: u8 = 0x6c;
    pub const I32DIVS: u8 = 0x6d;
    pub const I32DIVU: u8 = 0x6e;
    pub const I32REMS: u8 = 0x6f;
    pub const I32REMU: u8 = 0x70;
    pub const I32AND: u8 = 0x71;
    pub const I32OR: u8 = 0x72;
    pub const I32XOR: u8 = 0x73;
    pub const I32SHL: u8 = 0x74;
    pub const I32SHRS: u8 = 0x75;
    pub const I32SHRU: u8 = 0x76;
    pub const I32ROTL: u8 = 0x77;
    pub const I32ROTR: u8 = 0x78;

    pub const I64CLZ: u8 = 0x79;
    pub const I64CTZ: u8 = 0x7a;
    pub const I64POPCNT: u8 = 0x7b;
    pub const I64ADD: u8 = 0x7c;
    pub const I64SUB: u8 = 0x7d;
    pub const I64MUL: u8 = 0x7e;
    pub const I64DIVS: u8 = 0x7f;
    pub const I64DIVU: u8 = 0x80;
    pub const I64REMS: u8 = 0x81;
    pub const I64REMU: u8 = 0x82;
    pub const I64AND: u8 = 0x83;
    pub const I64OR: u8 = 0x84;
    pub const I64XOR: u8 = 0x85;
    pub const I64SHL: u8 = 0x86;
    pub const I64SHRS: u8 = 0x87;
    pub const I64SHRU: u8 = 0x88;
    pub const I64ROTL: u8 = 0x89;
    pub const I64ROTR: u8 = 0x8a;
    pub const F32ABS: u8 = 0x8b;
    pub const F32NEG: u8 = 0x8c;
    pub const F32CEIL: u8 = 0x8d;
    pub const F32FLOOR: u8 = 0x8e;
    pub const F32TRUNC: u8 = 0x8f;
    pub const F32NEAREST: u8 = 0x90;
    pub const F32SQRT: u8 = 0x91;
    pub const F32ADD: u8 = 0x92;
    pub const F32SUB: u8 = 0x93;
    pub const F32MUL: u8 = 0x94;
    pub const F32DIV: u8 = 0x95;
    pub const F32MIN: u8 = 0x96;
    pub const F32MAX: u8 = 0x97;
    pub const F32COPYSIGN: u8 = 0x98;
    pub const F64ABS: u8 = 0x99;
    pub const F64NEG: u8 = 0x9a;
    pub const F64CEIL: u8 = 0x9b;
    pub const F64FLOOR: u8 = 0x9c;
    pub const F64TRUNC: u8 = 0x9d;
    pub const F64NEAREST: u8 = 0x9e;
    pub const F64SQRT: u8 = 0x9f;
    pub const F64ADD: u8 = 0xa0;
    pub const F64SUB: u8 = 0xa1;
    pub const F64MUL: u8 = 0xa2;
    pub const F64DIV: u8 = 0xa3;
    pub const F64MIN: u8 = 0xa4;
    pub const F64MAX: u8 = 0xa5;
    pub const F64COPYSIGN: u8 = 0xa6;

    pub const I32WRAPI64: u8 = 0xa7;
    pub const I32TRUNCSF32: u8 = 0xa8;
    pub const I32TRUNCUF32: u8 = 0xa9;
    pub const I32TRUNCSF64: u8 = 0xaa;
    pub const I32TRUNCUF64: u8 = 0xab;
    pub const I64EXTENDSI32: u8 = 0xac;
    pub const I64EXTENDUI32: u8 = 0xad;
    pub const I64TRUNCSF32: u8 = 0xae;
    pub const I64TRUNCUF32: u8 = 0xaf;
    pub const I64TRUNCSF64: u8 = 0xb0;
    pub const I64TRUNCUF64: u8 = 0xb1;
    pub const F32CONVERTSI32: u8 = 0xb2;
    pub const F32CONVERTUI32: u8 = 0xb3;
    pub const F32CONVERTSI64: u8 = 0xb4;
    pub const F32CONVERTUI64: u8 = 0xb5;
    pub const F32DEMOTEF64: u8 = 0xb6;
    pub const F64CONVERTSI32: u8 = 0xb7;
    pub const F64CONVERTUI32: u8 = 0xb8;
    pub const F64CONVERTSI64: u8 = 0xb9;
    pub const F64CONVERTUI64: u8 = 0xba;
    pub const F64PROMOTEF32: u8 = 0xbb;

    pub const I32REINTERPRETF32: u8 = 0xbc;
    pub const I64REINTERPRETF64: u8 = 0xbd;
    pub const F32REINTERPRETI32: u8 = 0xbe;
    pub const F64REINTERPRETI64: u8 = 0xbf;

    pub const I32_EXTEND8_S: u8 = 0xc0;
    pub const I32_EXTEND16_S: u8 = 0xc1;
    pub const I64_EXTEND8_S: u8 = 0xc2;
    pub const I64_EXTEND16_S: u8 = 0xc3;
    pub const I64_EXTEND32_S: u8 = 0xc4;

    pub const ATOMIC_PREFIX: u8 = 0xfe;
    pub const ATOMIC_WAKE: u8 = 0x00;
    pub const I32_ATOMIC_WAIT: u8 = 0x01;
    pub const I64_ATOMIC_WAIT: u8 = 0x02;

    pub const I32_ATOMIC_LOAD: u8 = 0x10;
    pub const I64_ATOMIC_LOAD: u8 = 0x11;
    pub const I32_ATOMIC_LOAD8U: u8 = 0x12;
    pub const I32_ATOMIC_LOAD16U: u8 = 0x13;
    pub const I64_ATOMIC_LOAD8U: u8 = 0x14;
    pub const I64_ATOMIC_LOAD16U: u8 = 0x15;
    pub const I64_ATOMIC_LOAD32U: u8 = 0x16;
    pub const I32_ATOMIC_STORE: u8 = 0x17;
    pub const I64_ATOMIC_STORE: u8 = 0x18;
    pub const I32_ATOMIC_STORE8U: u8 = 0x19;
    pub const I32_ATOMIC_STORE16U: u8 = 0x1a;
    pub const I64_ATOMIC_STORE8U: u8 = 0x1b;
    pub const I64_ATOMIC_STORE16U: u8 = 0x1c;
    pub const I64_ATOMIC_STORE32U: u8 = 0x1d;

    pub const I32_ATOMIC_RMW_ADD: u8 = 0x1e;
    pub const I64_ATOMIC_RMW_ADD: u8 = 0x1f;
    pub const I32_ATOMIC_RMW_ADD8U: u8 = 0x20;
    pub const I32_ATOMIC_RMW_ADD16U: u8 = 0x21;
    pub const I64_ATOMIC_RMW_ADD8U: u8 = 0x22;
    pub const I64_ATOMIC_RMW_ADD16U: u8 = 0x23;
    pub const I64_ATOMIC_RMW_ADD32U: u8 = 0x24;

    pub const I32_ATOMIC_RMW_SUB: u8 = 0x25;
    pub const I64_ATOMIC_RMW_SUB: u8 = 0x26;
    pub const I32_ATOMIC_RMW_SUB8U: u8 = 0x27;
    pub const I32_ATOMIC_RMW_SUB16U: u8 = 0x28;
    pub const I64_ATOMIC_RMW_SUB8U: u8 = 0x29;
    pub const I64_ATOMIC_RMW_SUB16U: u8 = 0x2a;
    pub const I64_ATOMIC_RMW_SUB32U: u8 = 0x2b;

    pub const I32_ATOMIC_RMW_AND: u8 = 0x2c;
    pub const I64_ATOMIC_RMW_AND: u8 = 0x2d;
    pub const I32_ATOMIC_RMW_AND8U: u8 = 0x2e;
    pub const I32_ATOMIC_RMW_AND16U: u8 = 0x2f;
    pub const I64_ATOMIC_RMW_AND8U: u8 = 0x30;
    pub const I64_ATOMIC_RMW_AND16U: u8 = 0x31;
    pub const I64_ATOMIC_RMW_AND32U: u8 = 0x32;

    pub const I32_ATOMIC_RMW_OR: u8 = 0x33;
    pub const I64_ATOMIC_RMW_OR: u8 = 0x34;
    pub const I32_ATOMIC_RMW_OR8U: u8 = 0x35;
    pub const I32_ATOMIC_RMW_OR16U: u8 = 0x36;
    pub const I64_ATOMIC_RMW_OR8U: u8 = 0x37;
    pub const I64_ATOMIC_RMW_OR16U: u8 = 0x38;
    pub const I64_ATOMIC_RMW_OR32U: u8 = 0x39;

    pub const I32_ATOMIC_RMW_XOR: u8 = 0x3a;
    pub const I64_ATOMIC_RMW_XOR: u8 = 0x3b;
    pub const I32_ATOMIC_RMW_XOR8U: u8 = 0x3c;
    pub const I32_ATOMIC_RMW_XOR16U: u8 = 0x3d;
    pub const I64_ATOMIC_RMW_XOR8U: u8 = 0x3e;
    pub const I64_ATOMIC_RMW_XOR16U: u8 = 0x3f;
    pub const I64_ATOMIC_RMW_XOR32U: u8 = 0x40;

    pub const I32_ATOMIC_RMW_XCHG: u8 = 0x41;
    pub const I64_ATOMIC_RMW_XCHG: u8 = 0x42;
    pub const I32_ATOMIC_RMW_XCHG8U: u8 = 0x43;
    pub const I32_ATOMIC_RMW_XCHG16U: u8 = 0x44;
    pub const I64_ATOMIC_RMW_XCHG8U: u8 = 0x45;
    pub const I64_ATOMIC_RMW_XCHG16U: u8 = 0x46;
    pub const I64_ATOMIC_RMW_XCHG32U: u8 = 0x47;

    pub const I32_ATOMIC_RMW_CMPXCHG: u8 = 0x48;
    pub const I64_ATOMIC_RMW_CMPXCHG: u8 = 0x49;
    pub const I32_ATOMIC_RMW_CMPXCHG8U: u8 = 0x4a;
    pub const I32_ATOMIC_RMW_CMPXCHG16U: u8 = 0x4b;
    pub const I64_ATOMIC_RMW_CMPXCHG8U: u8 = 0x4c;
    pub const I64_ATOMIC_RMW_CMPXCHG16U: u8 = 0x4d;
    pub const I64_ATOMIC_RMW_CMPXCHG32U: u8 = 0x4e;

    // https://github.com/WebAssembly/simd/blob/master/proposals/simd/BinarySIMD.md
    pub const SIMD_PREFIX: u8 = 0xfd;

    pub const V128_LOAD: u32 = 0x00;
    pub const V128_STORE: u32 = 0x01;
    pub const V128_CONST: u32 = 0x02;
    pub const V8X16_SHUFFLE: u32 = 0x03;

    pub const I8X16_SPLAT: u32 = 0x04;
    pub const I8X16_EXTRACT_LANE_S: u32 = 0x05;
    pub const I8X16_EXTRACT_LANE_U: u32 = 0x06;
    pub const I8X16_REPLACE_LANE: u32 = 0x07;
    pub const I16X8_SPLAT: u32 = 0x08;
    pub const I16X8_EXTRACT_LANE_S: u32 = 0x09;
    pub const I16X8_EXTRACT_LANE_U: u32 = 0xa;
    pub const I16X8_REPLACE_LANE: u32 = 0x0b;
    pub const I32X4_SPLAT: u32 = 0x0c;
    pub const I32X4_EXTRACT_LANE: u32 = 0x0d;
    pub const I32X4_REPLACE_LANE: u32 = 0x0e;
    pub const I64X2_SPLAT: u32 = 0x0f;
    pub const I64X2_EXTRACT_LANE: u32 = 0x10;
    pub const I64X2_REPLACE_LANE: u32 = 0x11;
    pub const F32X4_SPLAT: u32 = 0x12;
    pub const F32X4_EXTRACT_LANE: u32 = 0x13;
    pub const F32X4_REPLACE_LANE: u32 = 0x14;
    pub const F64X2_SPLAT: u32 = 0x15;
    pub const F64X2_EXTRACT_LANE: u32 = 0x16;
    pub const F64X2_REPLACE_LANE: u32 = 0x17;

    pub const I8X16_EQ: u32 = 0x18;
    pub const I8X16_NE: u32 = 0x19;
    pub const I8X16_LT_S: u32 = 0x1a;
    pub const I8X16_LT_U: u32 = 0x1b;
    pub const I8X16_GT_S: u32 = 0x1c;
    pub const I8X16_GT_U: u32 = 0x1d;
    pub const I8X16_LE_S: u32 = 0x1e;
    pub const I8X16_LE_U: u32 = 0x1f;
    pub const I8X16_GE_S: u32 = 0x20;
    pub const I8X16_GE_U: u32 = 0x21;

    pub const I16X8_EQ: u32 = 0x22;
    pub const I16X8_NE: u32 = 0x23;
    pub const I16X8_LT_S: u32 = 0x24;
    pub const I16X8_LT_U: u32 = 0x25;
    pub const I16X8_GT_S: u32 = 0x26;
    pub const I16X8_GT_U: u32 = 0x27;
    pub const I16X8_LE_S: u32 = 0x28;
    pub const I16X8_LE_U: u32 = 0x29;
    pub const I16X8_GE_S: u32 = 0x2a;
    pub const I16X8_GE_U: u32 = 0x2b;

    pub const I32X4_EQ: u32 = 0x2c;
    pub const I32X4_NE: u32 = 0x2d;
    pub const I32X4_LT_S: u32 = 0x2e;
    pub const I32X4_LT_U: u32 = 0x2f;
    pub const I32X4_GT_S: u32 = 0x30;
    pub const I32X4_GT_U: u32 = 0x31;
    pub const I32X4_LE_S: u32 = 0x32;
    pub const I32X4_LE_U: u32 = 0x33;
    pub const I32X4_GE_S: u32 = 0x34;
    pub const I32X4_GE_U: u32 = 0x35;

    pub const F32X4_EQ: u32 = 0x40;
    pub const F32X4_NE: u32 = 0x41;
    pub const F32X4_LT: u32 = 0x42;
    pub const F32X4_GT: u32 = 0x43;
    pub const F32X4_LE: u32 = 0x44;
    pub const F32X4_GE: u32 = 0x45;

    pub const F64X2_EQ: u32 = 0x46;
    pub const F64X2_NE: u32 = 0x47;
    pub const F64X2_LT: u32 = 0x48;
    pub const F64X2_GT: u32 = 0x49;
    pub const F64X2_LE: u32 = 0x4a;
    pub const F64X2_GE: u32 = 0x4b;

    pub const V128_NOT: u32 = 0x4c;
    pub const V128_AND: u32 = 0x4d;
    pub const V128_OR: u32 = 0x4e;
    pub const V128_XOR: u32 = 0x4f;
    pub const V128_BITSELECT: u32 = 0x50;

    pub const I8X16_NEG: u32 = 0x51;
    pub const I8X16_ANY_TRUE: u32 = 0x52;
    pub const I8X16_ALL_TRUE: u32 = 0x53;
    pub const I8X16_SHL: u32 = 0x54;
    pub const I8X16_SHR_S: u32 = 0x55;
    pub const I8X16_SHR_U: u32 = 0x56;
    pub const I8X16_ADD: u32 = 0x57;
    pub const I8X16_ADD_SATURATE_S: u32 = 0x58;
    pub const I8X16_ADD_SATURATE_U: u32 = 0x59;
    pub const I8X16_SUB: u32 = 0x5a;
    pub const I8X16_SUB_SATURATE_S: u32 = 0x5b;
    pub const I8X16_SUB_SATURATE_U: u32 = 0x5c;
    pub const I8X16_MUL: u32 = 0x5d;

    pub const I16X8_NEG: u32 = 0x62;
    pub const I16X8_ANY_TRUE: u32 = 0x63;
    pub const I16X8_ALL_TRUE: u32 = 0x64;
    pub const I16X8_SHL: u32 = 0x65;
    pub const I16X8_SHR_S: u32 = 0x66;
    pub const I16X8_SHR_U: u32 = 0x67;
    pub const I16X8_ADD: u32 = 0x68;
    pub const I16X8_ADD_SATURATE_S: u32 = 0x69;
    pub const I16X8_ADD_SATURATE_U: u32 = 0x6a;
    pub const I16X8_SUB: u32 = 0x6b;
    pub const I16X8_SUB_SATURATE_S: u32 = 0x6c;
    pub const I16X8_SUB_SATURATE_U: u32 = 0x6d;
    pub const I16X8_MUL: u32 = 0x6e;

    pub const I32X4_NEG: u32 = 0x73;
    pub const I32X4_ANY_TRUE: u32 = 0x74;
    pub const I32X4_ALL_TRUE: u32 = 0x75;
    pub const I32X4_SHL: u32 = 0x76;
    pub const I32X4_SHR_S: u32 = 0x77;
    pub const I32X4_SHR_U: u32 = 0x78;
    pub const I32X4_ADD: u32 = 0x79;
    pub const I32X4_ADD_SATURATE_S: u32 = 0x7a;
    pub const I32X4_ADD_SATURATE_U: u32 = 0x7b;
    pub const I32X4_SUB: u32 = 0x7c;
    pub const I32X4_SUB_SATURATE_S: u32 = 0x7d;
    pub const I32X4_SUB_SATURATE_U: u32 = 0x7e;
    pub const I32X4_MUL: u32 = 0x7f;

    pub const I64X2_NEG: u32 = 0x84;
    pub const I64X2_ANY_TRUE: u32 = 0x85;
    pub const I64X2_ALL_TRUE: u32 = 0x86;
    pub const I64X2_SHL: u32 = 0x87;
    pub const I64X2_SHR_S: u32 = 0x88;
    pub const I64X2_SHR_U: u32 = 0x89;
    pub const I64X2_ADD: u32 = 0x8a;
    pub const I64X2_SUB: u32 = 0x8d;

    pub const F32X4_ABS: u32 = 0x95;
    pub const F32X4_NEG: u32 = 0x96;
    pub const F32X4_SQRT: u32 = 0x97;
    pub const F32X4_ADD: u32 = 0x9a;
    pub const F32X4_SUB: u32 = 0x9b;
    pub const F32X4_MUL: u32 = 0x9c;
    pub const F32X4_DIV: u32 = 0x9d;
    pub const F32X4_MIN: u32 = 0x9e;
    pub const F32X4_MAX: u32 = 0x9f;

    pub const F64X2_ABS: u32 = 0xa0;
    pub const F64X2_NEG: u32 = 0xa1;
    pub const F64X2_SQRT: u32 = 0xa2;
    pub const F64X2_ADD: u32 = 0xa5;
    pub const F64X2_SUB: u32 = 0xa6;
    pub const F64X2_MUL: u32 = 0xa7;
    pub const F64X2_DIV: u32 = 0xa8;
    pub const F64X2_MIN: u32 = 0xa9;
    pub const F64X2_MAX: u32 = 0xaa;

    pub const I32X4_TRUNC_S_F32X4_SAT: u32 = 0xab;
    pub const I32X4_TRUNC_U_F32X4_SAT: u32 = 0xac;
    pub const I64X2_TRUNC_S_F64X2_SAT: u32 = 0xad;
    pub const I64X2_TRUNC_U_F64X2_SAT: u32 = 0xae;

    pub const F32X4_CONVERT_S_I32X4: u32 = 0xaf;
    pub const F32X4_CONVERT_U_I32X4: u32 = 0xb0;
    pub const F64X2_CONVERT_S_I64X2: u32 = 0xb1;
    pub const F64X2_CONVERT_U_I64X2: u32 = 0xb2;

    pub const BULK_PREFIX: u8 = 0xfc;
    pub const MEMORY_INIT: u8 = 0x08;
    pub const MEMORY_DROP: u8 = 0x09;
    pub const MEMORY_COPY: u8 = 0x0a;
    pub const MEMORY_FILL: u8 = 0x0b;
    pub const TABLE_INIT: u8 = 0x0c;
    pub const TABLE_DROP: u8 = 0x0d;
    pub const TABLE_COPY: u8 = 0x0e;
}

impl Deserialize for Instruction {
    type Error = Error;

    fn deserialize<R: io::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        use self::Instruction::*;
        use self::opcodes::*;

        let val: u8 = Uint8::deserialize(reader)?.into();

        Ok(
            match val {
                UNREACHABLE => Unreachable,
                NOP => Nop,
                BLOCK => Block(BlockType::deserialize(reader)?),
                LOOP => Loop(BlockType::deserialize(reader)?),
                IF => If(BlockType::deserialize(reader)?),
                ELSE => Else,
                END => End,

                BR => Br(VarUint32::deserialize(reader)?.into()),
                BRIF => BrIf(VarUint32::deserialize(reader)?.into()),
                BRTABLE => {
                    let t1: Vec<u32> = CountedList::<VarUint32>::deserialize(reader)?
                        .into_inner()
                        .into_iter()
                        .map(Into::into)
                        .collect();

                    BrTable(Box::new(BrTableData {
                        table: t1.into_boxed_slice(),
                        default: VarUint32::deserialize(reader)?.into(),
                    }))
                },
                RETURN => Return,
                CALL => Call(VarUint32::deserialize(reader)?.into()),
                CALLINDIRECT => {
                    let signature: u32 = VarUint32::deserialize(reader)?.into();
                    let table_ref: u8 = Uint8::deserialize(reader)?.into();
                    if table_ref != 0 { return Err(Error::InvalidTableReference(table_ref)); }

                    CallIndirect(
                        signature,
                        table_ref,
                    )
                },
                DROP => Drop,
                SELECT => Select,

                GETLOCAL => GetLocal(VarUint32::deserialize(reader)?.into()),
                SETLOCAL => SetLocal(VarUint32::deserialize(reader)?.into()),
                TEELOCAL => TeeLocal(VarUint32::deserialize(reader)?.into()),
                GETGLOBAL => GetGlobal(VarUint32::deserialize(reader)?.into()),
                SETGLOBAL => SetGlobal(VarUint32::deserialize(reader)?.into()),

                I32LOAD => I32Load(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64LOAD => I64Load(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                F32LOAD => F32Load(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                F64LOAD => F64Load(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I32LOAD8S => I32Load8S(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I32LOAD8U => I32Load8U(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I32LOAD16S => I32Load16S(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I32LOAD16U => I32Load16U(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64LOAD8S => I64Load8S(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64LOAD8U => I64Load8U(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64LOAD16S => I64Load16S(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64LOAD16U => I64Load16U(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64LOAD32S => I64Load32S(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64LOAD32U => I64Load32U(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I32STORE => I32Store(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64STORE => I64Store(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                F32STORE => F32Store(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                F64STORE => F64Store(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I32STORE8 => I32Store8(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I32STORE16 => I32Store16(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64STORE8 => I64Store8(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64STORE16 => I64Store16(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),

                I64STORE32 => I64Store32(
                    VarUint32::deserialize(reader)?.into(),
                    VarUint32::deserialize(reader)?.into()),


                CURRENTMEMORY => {
                    let mem_ref: u8 = Uint8::deserialize(reader)?.into();
                    if mem_ref != 0 { return Err(Error::InvalidMemoryReference(mem_ref)); }
                    CurrentMemory(mem_ref)
                },
                GROWMEMORY => {
                    let mem_ref: u8 = Uint8::deserialize(reader)?.into();
                    if mem_ref != 0 { return Err(Error::InvalidMemoryReference(mem_ref)); }
                    GrowMemory(mem_ref)
                }

                I32CONST => I32Const(VarInt32::deserialize(reader)?.into()),
                I64CONST => I64Const(VarInt64::deserialize(reader)?.into()),
                F32CONST => F32Const(Uint32::deserialize(reader)?.into()),
                F64CONST => F64Const(Uint64::deserialize(reader)?.into()),
                I32EQZ => I32Eqz,
                I32EQ => I32Eq,
                I32NE => I32Ne,
                I32LTS => I32LtS,
                I32LTU => I32LtU,
                I32GTS => I32GtS,
                I32GTU => I32GtU,
                I32LES => I32LeS,
                I32LEU => I32LeU,
                I32GES => I32GeS,
                I32GEU => I32GeU,

                I64EQZ => I64Eqz,
                I64EQ => I64Eq,
                I64NE => I64Ne,
                I64LTS => I64LtS,
                I64LTU => I64LtU,
                I64GTS => I64GtS,
                I64GTU => I64GtU,
                I64LES => I64LeS,
                I64LEU => I64LeU,
                I64GES => I64GeS,
                I64GEU => I64GeU,

                F32EQ => F32Eq,
                F32NE => F32Ne,
                F32LT => F32Lt,
                F32GT => F32Gt,
                F32LE => F32Le,
                F32GE => F32Ge,

                F64EQ => F64Eq,
                F64NE => F64Ne,
                F64LT => F64Lt,
                F64GT => F64Gt,
                F64LE => F64Le,
                F64GE => F64Ge,

                I32CLZ => I32Clz,
                I32CTZ => I32Ctz,
                I32POPCNT => I32Popcnt,
                I32ADD => I32Add,
                I32SUB => I32Sub,
                I32MUL => I32Mul,
                I32DIVS => I32DivS,
                I32DIVU => I32DivU,
                I32REMS => I32RemS,
                I32REMU => I32RemU,
                I32AND => I32And,
                I32OR => I32Or,
                I32XOR => I32Xor,
                I32SHL => I32Shl,
                I32SHRS => I32ShrS,
                I32SHRU => I32ShrU,
                I32ROTL => I32Rotl,
                I32ROTR => I32Rotr,

                I64CLZ => I64Clz,
                I64CTZ => I64Ctz,
                I64POPCNT => I64Popcnt,
                I64ADD => I64Add,
                I64SUB => I64Sub,
                I64MUL => I64Mul,
                I64DIVS => I64DivS,
                I64DIVU => I64DivU,
                I64REMS => I64RemS,
                I64REMU => I64RemU,
                I64AND => I64And,
                I64OR => I64Or,
                I64XOR => I64Xor,
                I64SHL => I64Shl,
                I64SHRS => I64ShrS,
                I64SHRU => I64ShrU,
                I64ROTL => I64Rotl,
                I64ROTR => I64Rotr,
                F32ABS => F32Abs,
                F32NEG => F32Neg,
                F32CEIL => F32Ceil,
                F32FLOOR => F32Floor,
                F32TRUNC => F32Trunc,
                F32NEAREST => F32Nearest,
                F32SQRT => F32Sqrt,
                F32ADD => F32Add,
                F32SUB => F32Sub,
                F32MUL => F32Mul,
                F32DIV => F32Div,
                F32MIN => F32Min,
                F32MAX => F32Max,
                F32COPYSIGN => F32Copysign,
                F64ABS => F64Abs,
                F64NEG => F64Neg,
                F64CEIL => F64Ceil,
                F64FLOOR => F64Floor,
                F64TRUNC => F64Trunc,
                F64NEAREST => F64Nearest,
                F64SQRT => F64Sqrt,
                F64ADD => F64Add,
                F64SUB => F64Sub,
                F64MUL => F64Mul,
                F64DIV => F64Div,
                F64MIN => F64Min,
                F64MAX => F64Max,
                F64COPYSIGN => F64Copysign,

                I32WRAPI64 => I32WrapI64,
                I32TRUNCSF32 => I32TruncSF32,
                I32TRUNCUF32 => I32TruncUF32,
                I32TRUNCSF64 => I32TruncSF64,
                I32TRUNCUF64 => I32TruncUF64,
                I64EXTENDSI32 => I64ExtendSI32,
                I64EXTENDUI32 => I64ExtendUI32,
                I64TRUNCSF32 => I64TruncSF32,
                I64TRUNCUF32 => I64TruncUF32,
                I64TRUNCSF64 => I64TruncSF64,
                I64TRUNCUF64 => I64TruncUF64,
                F32CONVERTSI32 => F32ConvertSI32,
                F32CONVERTUI32 => F32ConvertUI32,
                F32CONVERTSI64 => F32ConvertSI64,
                F32CONVERTUI64 => F32ConvertUI64,
                F32DEMOTEF64 => F32DemoteF64,
                F64CONVERTSI32 => F64ConvertSI32,
                F64CONVERTUI32 => F64ConvertUI32,
                F64CONVERTSI64 => F64ConvertSI64,
                F64CONVERTUI64 => F64ConvertUI64,
                F64PROMOTEF32 => F64PromoteF32,

                I32REINTERPRETF32 => I32ReinterpretF32,
                I64REINTERPRETF64 => I64ReinterpretF64,
                F32REINTERPRETI32 => F32ReinterpretI32,
                F64REINTERPRETI64 => F64ReinterpretI64,
                I32_EXTEND8_S => I32Extend8S,
                I32_EXTEND16_S => I32Extend16S,
                I64_EXTEND8_S => I64Extend8S,
                I64_EXTEND16_S => I64Extend16S,
                I64_EXTEND32_S => I64Extend32S,

                ATOMIC_PREFIX => return deserialize_atomic(reader),
                SIMD_PREFIX => return deserialize_simd(reader),

                BULK_PREFIX => return deserialize_bulk(reader),

                _ => { return Err(Error::UnknownOpcode(val)); }
            }
        )
    }
}

fn deserialize_atomic<R: io::Read<u8>>(reader: &mut R) -> Result<Instruction, Error> {
    use self::Instruction::*;
    use self::opcodes::*;

    let val: u8 = Uint8::deserialize(reader)?.into();
    let mem = MemArg::deserialize(reader)?;
    Ok(match val {
        ATOMIC_WAKE => AtomicWake(mem),
        I32_ATOMIC_WAIT => I32AtomicWait(mem),
        I64_ATOMIC_WAIT => I64AtomicWait(mem),

        I32_ATOMIC_LOAD => I32AtomicLoad(mem),
        I64_ATOMIC_LOAD => I64AtomicLoad(mem),
        I32_ATOMIC_LOAD8U => I32AtomicLoad8u(mem),
        I32_ATOMIC_LOAD16U => I32AtomicLoad16u(mem),
        I64_ATOMIC_LOAD8U => I64AtomicLoad8u(mem),
        I64_ATOMIC_LOAD16U => I64AtomicLoad16u(mem),
        I64_ATOMIC_LOAD32U => I64AtomicLoad32u(mem),
        I32_ATOMIC_STORE => I32AtomicStore(mem),
        I64_ATOMIC_STORE => I64AtomicStore(mem),
        I32_ATOMIC_STORE8U => I32AtomicStore8u(mem),
        I32_ATOMIC_STORE16U => I32AtomicStore16u(mem),
        I64_ATOMIC_STORE8U => I64AtomicStore8u(mem),
        I64_ATOMIC_STORE16U => I64AtomicStore16u(mem),
        I64_ATOMIC_STORE32U => I64AtomicStore32u(mem),

        I32_ATOMIC_RMW_ADD => I32AtomicRmwAdd(mem),
        I64_ATOMIC_RMW_ADD => I64AtomicRmwAdd(mem),
        I32_ATOMIC_RMW_ADD8U => I32AtomicRmwAdd8u(mem),
        I32_ATOMIC_RMW_ADD16U => I32AtomicRmwAdd16u(mem),
        I64_ATOMIC_RMW_ADD8U => I64AtomicRmwAdd8u(mem),
        I64_ATOMIC_RMW_ADD16U => I64AtomicRmwAdd16u(mem),
        I64_ATOMIC_RMW_ADD32U => I64AtomicRmwAdd32u(mem),

        I32_ATOMIC_RMW_SUB => I32AtomicRmwSub(mem),
        I64_ATOMIC_RMW_SUB => I64AtomicRmwSub(mem),
        I32_ATOMIC_RMW_SUB8U => I32AtomicRmwSub8u(mem),
        I32_ATOMIC_RMW_SUB16U => I32AtomicRmwSub16u(mem),
        I64_ATOMIC_RMW_SUB8U => I64AtomicRmwSub8u(mem),
        I64_ATOMIC_RMW_SUB16U => I64AtomicRmwSub16u(mem),
        I64_ATOMIC_RMW_SUB32U => I64AtomicRmwSub32u(mem),

        I32_ATOMIC_RMW_OR => I32AtomicRmwOr(mem),
        I64_ATOMIC_RMW_OR => I64AtomicRmwOr(mem),
        I32_ATOMIC_RMW_OR8U => I32AtomicRmwOr8u(mem),
        I32_ATOMIC_RMW_OR16U => I32AtomicRmwOr16u(mem),
        I64_ATOMIC_RMW_OR8U => I64AtomicRmwOr8u(mem),
        I64_ATOMIC_RMW_OR16U => I64AtomicRmwOr16u(mem),
        I64_ATOMIC_RMW_OR32U => I64AtomicRmwOr32u(mem),

        I32_ATOMIC_RMW_XOR => I32AtomicRmwXor(mem),
        I64_ATOMIC_RMW_XOR => I64AtomicRmwXor(mem),
        I32_ATOMIC_RMW_XOR8U => I32AtomicRmwXor8u(mem),
        I32_ATOMIC_RMW_XOR16U => I32AtomicRmwXor16u(mem),
        I64_ATOMIC_RMW_XOR8U => I64AtomicRmwXor8u(mem),
        I64_ATOMIC_RMW_XOR16U => I64AtomicRmwXor16u(mem),
        I64_ATOMIC_RMW_XOR32U => I64AtomicRmwXor32u(mem),

        I32_ATOMIC_RMW_XCHG => I32AtomicRmwXchg(mem),
        I64_ATOMIC_RMW_XCHG => I64AtomicRmwXchg(mem),
        I32_ATOMIC_RMW_XCHG8U => I32AtomicRmwXchg8u(mem),
        I32_ATOMIC_RMW_XCHG16U => I32AtomicRmwXchg16u(mem),
        I64_ATOMIC_RMW_XCHG8U => I64AtomicRmwXchg8u(mem),
        I64_ATOMIC_RMW_XCHG16U => I64AtomicRmwXchg16u(mem),
        I64_ATOMIC_RMW_XCHG32U => I64AtomicRmwXchg32u(mem),

        I32_ATOMIC_RMW_CMPXCHG => I32AtomicRmwCmpxchg(mem),
        I64_ATOMIC_RMW_CMPXCHG => I64AtomicRmwCmpxchg(mem),
        I32_ATOMIC_RMW_CMPXCHG8U => I32AtomicRmwCmpxchg8u(mem),
        I32_ATOMIC_RMW_CMPXCHG16U => I32AtomicRmwCmpxchg16u(mem),
        I64_ATOMIC_RMW_CMPXCHG8U => I64AtomicRmwCmpxchg8u(mem),
        I64_ATOMIC_RMW_CMPXCHG16U => I64AtomicRmwCmpxchg16u(mem),
        I64_ATOMIC_RMW_CMPXCHG32U => I64AtomicRmwCmpxchg32u(mem),

        _ => return Err(Error::UnknownOpcode(val)),
    })
}

fn deserialize_simd<R: io::Read<u8>>(reader: &mut R) -> Result<Instruction, Error> {
    use self::Instruction::*;
    use self::opcodes::*;

    let val = VarUint32::deserialize(reader)?.into();
    Ok(match val {
        V128_CONST => {
            let mut buf = [0; 16];
            reader.read(&mut buf)?;
            V128Const(Box::new(buf))
        }
        V128_LOAD => V128Load(MemArg::deserialize(reader)?),
        V128_STORE => V128Store(MemArg::deserialize(reader)?),
        I8X16_SPLAT => I8x16Splat,
        I16X8_SPLAT => I16x8Splat,
        I32X4_SPLAT => I32x4Splat,
        I64X2_SPLAT => I64x2Splat,
        F32X4_SPLAT => F32x4Splat,
        F64X2_SPLAT => F64x2Splat,
        I8X16_EXTRACT_LANE_S => I8x16ExtractLaneS(Uint8::deserialize(reader)?.into()),
        I8X16_EXTRACT_LANE_U => I8x16ExtractLaneU(Uint8::deserialize(reader)?.into()),
        I16X8_EXTRACT_LANE_S => I16x8ExtractLaneS(Uint8::deserialize(reader)?.into()),
        I16X8_EXTRACT_LANE_U => I16x8ExtractLaneU(Uint8::deserialize(reader)?.into()),
        I32X4_EXTRACT_LANE => I32x4ExtractLane(Uint8::deserialize(reader)?.into()),
        I64X2_EXTRACT_LANE => I64x2ExtractLane(Uint8::deserialize(reader)?.into()),
        F32X4_EXTRACT_LANE => F32x4ExtractLane(Uint8::deserialize(reader)?.into()),
        F64X2_EXTRACT_LANE => F64x2ExtractLane(Uint8::deserialize(reader)?.into()),
        I8X16_REPLACE_LANE => I8x16ReplaceLane(Uint8::deserialize(reader)?.into()),
        I16X8_REPLACE_LANE => I16x8ReplaceLane(Uint8::deserialize(reader)?.into()),
        I32X4_REPLACE_LANE => I32x4ReplaceLane(Uint8::deserialize(reader)?.into()),
        I64X2_REPLACE_LANE => I64x2ReplaceLane(Uint8::deserialize(reader)?.into()),
        F32X4_REPLACE_LANE => F32x4ReplaceLane(Uint8::deserialize(reader)?.into()),
        F64X2_REPLACE_LANE => F64x2ReplaceLane(Uint8::deserialize(reader)?.into()),
        V8X16_SHUFFLE => {
            let mut buf = [0; 16];
            reader.read(&mut buf)?;
            V8x16Shuffle(Box::new(buf))
        }
        I8X16_ADD => I8x16Add,
        I16X8_ADD => I16x8Add,
        I32X4_ADD => I32x4Add,
        I64X2_ADD => I64x2Add,
        I8X16_SUB => I8x16Sub,
        I16X8_SUB => I16x8Sub,
        I32X4_SUB => I32x4Sub,
        I64X2_SUB => I64x2Sub,
        I8X16_MUL => I8x16Mul,
        I16X8_MUL => I16x8Mul,
        I32X4_MUL => I32x4Mul,
        // I64X2_MUL => I64x2Mul,
        I8X16_NEG => I8x16Neg,
        I16X8_NEG => I16x8Neg,
        I32X4_NEG => I32x4Neg,
        I64X2_NEG => I64x2Neg,

        I8X16_ADD_SATURATE_S => I8x16AddSaturateS,
        I8X16_ADD_SATURATE_U => I8x16AddSaturateU,
        I16X8_ADD_SATURATE_S => I16x8AddSaturateS,
        I16X8_ADD_SATURATE_U => I16x8AddSaturateU,
        I8X16_SUB_SATURATE_S => I8x16SubSaturateS,
        I8X16_SUB_SATURATE_U => I8x16SubSaturateU,
        I16X8_SUB_SATURATE_S => I16x8SubSaturateS,
        I16X8_SUB_SATURATE_U => I16x8SubSaturateU,
        I8X16_SHL => I8x16Shl,
        I16X8_SHL => I16x8Shl,
        I32X4_SHL => I32x4Shl,
        I64X2_SHL => I64x2Shl,
        I8X16_SHR_S => I8x16ShrS,
        I8X16_SHR_U => I8x16ShrU,
        I16X8_SHR_S => I16x8ShrS,
        I16X8_SHR_U => I16x8ShrU,
        I32X4_SHR_S => I32x4ShrS,
        I32X4_SHR_U => I32x4ShrU,
        I64X2_SHR_S => I64x2ShrS,
        I64X2_SHR_U => I64x2ShrU,
        V128_AND => V128And,
        V128_OR => V128Or,
        V128_XOR => V128Xor,
        V128_NOT => V128Not,
        V128_BITSELECT => V128Bitselect,
        I8X16_ANY_TRUE => I8x16AnyTrue,
        I16X8_ANY_TRUE => I16x8AnyTrue,
        I32X4_ANY_TRUE => I32x4AnyTrue,
        I64X2_ANY_TRUE => I64x2AnyTrue,
        I8X16_ALL_TRUE => I8x16AllTrue,
        I16X8_ALL_TRUE => I16x8AllTrue,
        I32X4_ALL_TRUE => I32x4AllTrue,
        I64X2_ALL_TRUE => I64x2AllTrue,
        I8X16_EQ => I8x16Eq,
        I16X8_EQ => I16x8Eq,
        I32X4_EQ => I32x4Eq,
        // I64X2_EQ => I64x2Eq,
        F32X4_EQ => F32x4Eq,
        F64X2_EQ => F64x2Eq,
        I8X16_NE => I8x16Ne,
        I16X8_NE => I16x8Ne,
        I32X4_NE => I32x4Ne,
        // I64X2_NE => I64x2Ne,
        F32X4_NE => F32x4Ne,
        F64X2_NE => F64x2Ne,
        I8X16_LT_S => I8x16LtS,
        I8X16_LT_U => I8x16LtU,
        I16X8_LT_S => I16x8LtS,
        I16X8_LT_U => I16x8LtU,
        I32X4_LT_S => I32x4LtS,
        I32X4_LT_U => I32x4LtU,
        // I64X2_LT_S => I64x2LtS,
        // I64X2_LT_U => I64x2LtU,
        F32X4_LT => F32x4Lt,
        F64X2_LT => F64x2Lt,
        I8X16_LE_S => I8x16LeS,
        I8X16_LE_U => I8x16LeU,
        I16X8_LE_S => I16x8LeS,
        I16X8_LE_U => I16x8LeU,
        I32X4_LE_S => I32x4LeS,
        I32X4_LE_U => I32x4LeU,
        // I64X2_LE_S => I64x2LeS,
        // I64X2_LE_U => I64x2LeU,
        F32X4_LE => F32x4Le,
        F64X2_LE => F64x2Le,
        I8X16_GT_S => I8x16GtS,
        I8X16_GT_U => I8x16GtU,
        I16X8_GT_S => I16x8GtS,
        I16X8_GT_U => I16x8GtU,
        I32X4_GT_S => I32x4GtS,
        I32X4_GT_U => I32x4GtU,
        // I64X2_GT_S => I64x2GtS,
        // I64X2_GT_U => I64x2GtU,
        F32X4_GT => F32x4Gt,
        F64X2_GT => F64x2Gt,
        I8X16_GE_S => I8x16GeS,
        I8X16_GE_U => I8x16GeU,
        I16X8_GE_S => I16x8GeS,
        I16X8_GE_U => I16x8GeU,
        I32X4_GE_S => I32x4GeS,
        I32X4_GE_U => I32x4GeU,
        // I64X2_GE_S => I64x2GeS,
        // I64X2_GE_U => I64x2GeU,
        F32X4_GE => F32x4Ge,
        F64X2_GE => F64x2Ge,
        F32X4_NEG => F32x4Neg,
        F64X2_NEG => F64x2Neg,
        F32X4_ABS => F32x4Abs,
        F64X2_ABS => F64x2Abs,
        F32X4_MIN => F32x4Min,
        F64X2_MIN => F64x2Min,
        F32X4_MAX => F32x4Max,
        F64X2_MAX => F64x2Max,
        F32X4_ADD => F32x4Add,
        F64X2_ADD => F64x2Add,
        F32X4_SUB => F32x4Sub,
        F64X2_SUB => F64x2Sub,
        F32X4_DIV => F32x4Div,
        F64X2_DIV => F64x2Div,
        F32X4_MUL => F32x4Mul,
        F64X2_MUL => F64x2Mul,
        F32X4_SQRT => F32x4Sqrt,
        F64X2_SQRT => F64x2Sqrt,
        F32X4_CONVERT_S_I32X4 => F32x4ConvertSI32x4,
        F32X4_CONVERT_U_I32X4 => F32x4ConvertUI32x4,
        F64X2_CONVERT_S_I64X2 => F64x2ConvertSI64x2,
        F64X2_CONVERT_U_I64X2 => F64x2ConvertUI64x2,
        I32X4_TRUNC_S_F32X4_SAT => I32x4TruncSF32x4Sat,
        I32X4_TRUNC_U_F32X4_SAT => I32x4TruncUF32x4Sat,
        I64X2_TRUNC_S_F64X2_SAT => I64x2TruncSF64x2Sat,
        I64X2_TRUNC_U_F64X2_SAT => I64x2TruncUF64x2Sat,

        _ => return Err(Error::UnknownSimdOpcode(val)),
    })
}

fn deserialize_bulk<R: io::Read<u8>>(reader: &mut R) -> Result<Instruction, Error> {
    use self::Instruction::*;
    use self::opcodes::*;

    let val: u8 = Uint8::deserialize(reader)?.into();
    Ok(match val {
        MEMORY_INIT => {
            if u8::from(Uint8::deserialize(reader)?) != 0 {
                return Err(Error::UnknownOpcode(val))
            }
            MemoryInit(VarUint32::deserialize(reader)?.into())
        }
        MEMORY_DROP => MemoryDrop(VarUint32::deserialize(reader)?.into()),
        MEMORY_FILL => {
            if u8::from(Uint8::deserialize(reader)?) != 0 {
                return Err(Error::UnknownOpcode(val))
            }
            MemoryFill
        }
        MEMORY_COPY => {
            if u8::from(Uint8::deserialize(reader)?) != 0 {
                return Err(Error::UnknownOpcode(val))
            }
            MemoryCopy
        }

        TABLE_INIT => {
            if u8::from(Uint8::deserialize(reader)?) != 0 {
                return Err(Error::UnknownOpcode(val))
            }
            TableInit(VarUint32::deserialize(reader)?.into())
        }
        TABLE_DROP => TableDrop(VarUint32::deserialize(reader)?.into()),
        TABLE_COPY => {
            if u8::from(Uint8::deserialize(reader)?) != 0 {
                return Err(Error::UnknownOpcode(val))
            }
            TableCopy
        }

        _ => return Err(Error::UnknownOpcode(val)),
    })
}

impl Deserialize for MemArg {
    type Error = Error;

    fn deserialize<R: io::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let align = Uint8::deserialize(reader)?;
        let offset = VarUint32::deserialize(reader)?;
        Ok(MemArg { align: align.into(), offset: offset.into() })
    }
}
