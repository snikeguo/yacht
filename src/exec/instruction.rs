use crate::metadata::token::*;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Instruction {
    Ldnull,
    Ldstr(u32),
    Ldc_I4_0,
    Ldc_I4_1,
    Ldc_I4_2,
    Ldc_I4_3,
    Ldc_I4_4,
    Ldc_I4_5,
    Ldc_I4_6,
    Ldc_I4_7,
    Ldc_I4_8,
    Ldc_I4_S(i32),
    Ldc_I4(i32),
    Ldarg_0,
    Ldarg_1,
    Ldarg_2,
    Ldarg_3,
    Ldloc_0,
    Ldloc_1,
    Ldloc_2,
    Ldloc_3,
    Ldloc_S(u8),
    Ldfld(Token),
    Ldelem_I1,
    Ldelem_U1,
    Ldelem_I4,
    Stloc_0,
    Stloc_1,
    Stloc_2,
    Stloc_3,
    Stloc_S(u8),
    Stfld(Token),
    Stelem_I1,
    Stelem_I4,
    Ldlen,
    Conv_I4,
    Dup,
    Pop,
    Beq(usize),
    Bne_un(usize),
    Bge(usize),
    Bgt(usize),
    Blt(usize),
    Ble(usize),
    Brfalse(usize),
    Brtrue(usize),
    Br(usize),
    Clt,
    Ceq,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Call(Token),
    CallVirt(Token),
    Box(Token),
    Newobj(Token),
    Newarr(Token),
    Ret,
}

#[rustfmt::skip]
pub mod il_instr {
    pub const LDNULL   : u8 = 0x14;
    pub const LDSTR    : u8 = 0x72;
    pub const CALL     : u8 = 0x28;
    pub const CALLVIRT : u8 = 0x6f;
    pub const LDC_I4_0 : u8 = 0x16;
    pub const LDC_I4_1 : u8 = 0x17;
    pub const LDC_I4_2 : u8 = 0x18;
    pub const LDC_I4_3 : u8 = 0x19;
    pub const LDC_I4_4 : u8 = 0x1a;
    pub const LDC_I4_5 : u8 = 0x1b;
    pub const LDC_I4_6 : u8 = 0x1c;
    pub const LDC_I4_7 : u8 = 0x1d;
    pub const LDC_I4_8 : u8 = 0x1e;
    pub const LDC_I4_S : u8 = 0x1f;
    pub const LDC_I4   : u8 = 0x20;
    pub const LDARG_0  : u8 = 0x02;
    pub const LDARG_1  : u8 = 0x03;
    pub const LDARG_2  : u8 = 0x04;
    pub const LDARG_3  : u8 = 0x05;
    pub const LDLOC_0  : u8 = 0x06;
    pub const LDLOC_1  : u8 = 0x07;
    pub const LDLOC_2  : u8 = 0x08;
    pub const LDLOC_3  : u8 = 0x09;
    pub const LDLOC_S  : u8 = 0x11;
    pub const LDFLD    : u8 = 0x7b;
    pub const LDELEM_I1: u8 = 0x90;
    pub const LDELEM_U1: u8 = 0x91;
    pub const LDELEM_I4: u8 = 0x94;
    pub const STLOC_0  : u8 = 0x0a;
    pub const STLOC_1  : u8 = 0x0b;
    pub const STLOC_2  : u8 = 0x0c;
    pub const STLOC_3  : u8 = 0x0d;
    pub const STLOC_S  : u8 = 0x13;
    pub const STFLD    : u8 = 0x7d;
    pub const STELEM_I1: u8 = 0x9c;
    pub const STELEM_I4: u8 = 0x9e;
    pub const LDLEN    : u8 = 0x8e;
    pub const CONV_I4  : u8 = 0x69;
    pub const DUP      : u8 = 0x25;
    pub const POP      : u8 = 0x26;
    pub const BR       : u8 = 0x38;
    pub const BGE      : u8 = 0x3c;
    pub const BGT      : u8 = 0x3d;
    pub const BLE      : u8 = 0x3e;
    pub const BLT      : u8 = 0x3f;
    pub const BEQ      : u8 = 0x3b;
    pub const BNE_UN   : u8 = 0x40;
    pub const BRFALSE  : u8 = 0x39;
    pub const BRTRUE   : u8 = 0x3a;
    pub const CLT      : u8 = 0x04; // 0xfe leads
    pub const CEQ      : u8 = 0x01; // 0xfe leads
    pub const ADD      : u8 = 0x58;
    pub const SUB      : u8 = 0x59;
    pub const MUL      : u8 = 0x5a;
    pub const DIV      : u8 = 0x5b;
    pub const REM      : u8 = 0x5d;
    pub const BOX      : u8 = 0x8c;
    pub const NEWOBJ   : u8 = 0x73;
    pub const NEWARR   : u8 = 0x8d;
    pub const RET      : u8 = 0x2a;

    pub fn get_instr_size<'a>(instr: u8) -> usize {
        match instr {
            LDNULL | LDSTR | 
            CALL | CALLVIRT |
            NEWOBJ | NEWARR | BOX |
            STFLD | LDFLD |
            BGE | BR | BLT | BNE_UN | BRFALSE | BGT
             | BRTRUE | BLE | BEQ |
            LDC_I4 => 5, 
            LDC_I4_0 | LDC_I4_1 | LDC_I4_2 | LDC_I4_3 
             | LDC_I4_4 | LDC_I4_5 | LDC_I4_6 
             | LDC_I4_7 | LDC_I4_8 |
            LDARG_0 | LDARG_1 | LDARG_2 | LDARG_3 | 
            LDLOC_0 | LDLOC_1 | LDLOC_2 | LDLOC_3 |
            LDELEM_I4 | LDELEM_I1 | LDELEM_U1 |
            STLOC_0 | STLOC_1 | STLOC_2 | STLOC_3 |
            STELEM_I4 | STELEM_I1 |
            ADD | SUB | MUL | DIV | REM |
            RET | POP | DUP |
            CONV_I4 |
            LDLEN => 1,
            LDLOC_S |
            STLOC_S |
            LDC_I4_S => 2,
            e => panic!("Not an instruction: {}", e),
        }
    }

    pub fn get_instr2_size<'a>(instr: u8) -> usize {
        match instr {
            CLT | CEQ => 2,
            e => panic!("Not an instruction: {}", e),
        }
    }
}
