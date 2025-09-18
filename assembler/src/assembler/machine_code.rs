use crate::tokens::Register;

use super::Assembler;

impl Assembler {
    /// Constructs and returns the byte that tells the CPU where to move the data for mov-like
    /// instructions
    fn create_dest_byte(destination: Register) -> u8 {
        let mut dest_byte: u8 = 0;
        let dst_index = destination.index();

        // The first 5 bits stores the register
        dest_byte |= dst_index << 3;

        let size = destination.size();
        match size {
            1 => dest_byte |= 0b000,
            2 => dest_byte |= 0b010,
            4 => dest_byte |= 0b100,
            8 => dest_byte |= 0b110,
            _ => unreachable!(),
        }

        dest_byte
    }

    /// Constructs and returns the byte that tells the CPU where the data that gets copied to the
    /// destination comes from
    fn create_source_byte(source: Register) -> u8 {
        let mut src_byte: u8 = 0;
        let src_index = source.index();

        // The first 5 bits stores the register
        src_byte |= src_index << 3;

        src_byte

    }
}

impl Assembler {
    pub fn assemble_mov_reg_reg(&mut self, destination: Register, source: Register, mc: &mut Vec<u8>) {
        let dest_byte = Self::create_dest_byte(destination);

        let src_byte = Self::create_source_byte(source);

        mc.push(dest_byte);
        mc.push(src_byte);
    }

    pub fn assemble_mov_reg_imm8(&mut self, dest: Register, value: u8, mc: &mut Vec<u8>) {
        let dest_byte = Self::create_dest_byte(dest);

        mc.push(dest_byte);
        mc.push(value);
    }

    pub fn assemble_mov_reg_imm16(&mut self, dest: Register, value: u16, mc: &mut Vec<u8>) {
        let dest_byte = Self::create_dest_byte(dest);

        mc.push(dest_byte);
        mc.extend_from_slice(&value.to_le_bytes());
        // mc.push(value.to_le_bytes());
    }

    pub fn assemble_mov_reg_imm32(&mut self, dest: Register, value: u32, mc: &mut Vec<u8>) {
        let dest_byte = Self::create_dest_byte(dest);

        mc.push(dest_byte);
        mc.extend_from_slice(&value.to_le_bytes());
    }

    pub fn assemble_mov_reg_imm64(&mut self, dest: Register, value: u64, mc: &mut Vec<u8>) {
        let dest_byte = Self::create_dest_byte(dest);

        mc.push(dest_byte);
        mc.extend_from_slice(&value.to_le_bytes());
    }

    pub fn assemble_int(&mut self, code: u8, mc: &mut Vec<u8>) {
        mc.push(code);  
    }
}
