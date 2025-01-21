use crate::backend::util::byte::Bytes;
use strum::{Display, EnumIter};

pub type Word = Bytes<4>;
pub type Byte = Bytes<1>;

pub mod mux_sel {
    pub mod pc {
        pub const PC_PLUS4: u8 = 0x00;
        pub const ALU_OUT: u8 = 0x01;
        pub const ALU_MOD2: u8 = 0x02;
    }

    pub mod mar {
        pub const PC_OUT: u8 = 0x00;
        pub const ALU_OUT: u8 = 0x01;
    }

    pub mod cmp {
        pub const RS2_OUT: u8 = 0x00;
        pub const I_IMM: u8 = 0x01;
    }

    pub mod alu1 {
        pub const RS1_OUT: u8 = 0x00;
        pub const PC_OUT: u8 = 0x01;
    }

    pub mod alu2 {
        pub const I_IMM: u8 = 0x00;
        pub const U_IMM: u8 = 0x01;
        pub const B_IMM: u8 = 0x02;
        pub const S_IMM: u8 = 0x03;
        pub const J_IMM: u8 = 0x04;
        pub const RS2_OUT: u8 = 0x05;
    }

    pub mod regfile {
        pub const ALU_OUT: u8 = 0x00;
        pub const BR_EN: u8 = 0x01;
        pub const U_IMM: u8 = 0x02;
        pub const LW: u8 = 0x03;
        pub const PC_PLUS4: u8 = 0x04;
        pub const LB: u8 = 0x05;
        pub const LBU: u8 = 0x06;
        pub const LH: u8 = 0x07;
        pub const LHU: u8 = 0x08;
    }
}

pub mod opcode {
    pub const LUI: u8 = 0b00110111;
    pub const AUIPC: u8 = 0b00010111;
    pub const JAL: u8 = 0b01101111;
    pub const JALR: u8 = 0b01100111;
    pub const BR: u8 = 0b01100011;
    pub const LOAD: u8 = 0b00000011;
    pub const STORE: u8 = 0b00100011;
    pub const IMM: u8 = 0b00010011;
    pub const REG: u8 = 0b00110011;
}

pub mod funct3 {
    pub mod branch {
        pub const BEQ: u8 = 0b000;
        pub const BNE: u8 = 0b001;
        pub const BLT: u8 = 0b100;
        pub const BGE: u8 = 0b101;
        pub const BLTU: u8 = 0b110;
        pub const BGEU: u8 = 0b111;
    }

    pub mod load {
        pub const LB: u8 = 0b000;
        pub const LH: u8 = 0b001;
        pub const LW: u8 = 0b010;
        pub const LBU: u8 = 0b100;
        pub const LHU: u8 = 0b101;
    }

    pub mod store {
        pub const SB: u8 = 0b000;
        pub const SH: u8 = 0b001;
        pub const SW: u8 = 0b010;
    }

    pub mod arith {
        pub const ADD: u8 = 0b000;
        pub const SLT: u8 = 0b010;
        pub const SLTU: u8 = 0b011;
        pub const SR: u8 = 0b101;
    }
}

pub mod alu_op {
    pub const ADD: u8 = 0b000;
    pub const SLL: u8 = 0b001;
    pub const SRA: u8 = 0b010;
    pub const SUB: u8 = 0b011;
    pub const XOR: u8 = 0b100;
    pub const SRL: u8 = 0b101;
    pub const OR: u8 = 0b110;
    pub const AND: u8 = 0b111;
}

#[derive(Display, Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum States {
    Fetch1,
    Fetch2,
    Fetch3,
    Decode,
    Imm,
    Lui,
    Br,
    Auipc,
    AddrCalc,
    Load1,
    Load2,
    Store1,
    Store2,
    Jal,
    Jalr,
    Reg,
}
