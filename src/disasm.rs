//
// Author: Patrick Walton
//

use mem::Mem;


pub struct Disassembler<'a, M: Mem + 'a> {
    pub pc: u16,
    pub mem: &'a mut M
}

impl<'a, M: Mem> Disassembler<'a, M> {
    //
    // Loads and byte-to-string conversion
    //

    fn loadb_bump_pc(&mut self) -> u8 {
        let val = (&mut *self.mem).loadb(self.pc);
        self.pc += 1;
        val
    }
    fn loadw_bump_pc(&mut self) -> u16 {
        let bottom = self.loadb_bump_pc() as u16;
        let top = (self.loadb_bump_pc() as u16) << 8;
        bottom | top
    }

    fn disb_bump_pc(&mut self) -> String {
        format!("${:02X}", self.loadb_bump_pc() as usize)
    }
    fn disw_bump_pc(&mut self) -> String {
        format!("${:04X}", self.loadw_bump_pc() as usize)
    }

    //
    // Mnemonics
    //

    // TODO: When we get method macros some of this ugly duplication can go away.

    // Loads
    fn lda(&mut self, am: String) -> String { format!("LDA {}", am) }
    fn ldx(&mut self, am: String) -> String { format!("LDX {}", am) }
    fn ldy(&mut self, am: String) -> String { format!("LDY {}", am) }

    // Stores
    fn sta(&mut self, am: String) -> String { format!("STA {}", am) }
    fn stx(&mut self, am: String) -> String { format!("STX {}", am) }
    fn sty(&mut self, am: String) -> String { format!("STY {}", am) }

    // Arithmetic
    fn adc(&mut self, am: String) -> String { format!("ADC {}", am) }
    fn sbc(&mut self, am: String) -> String { format!("SBC {}", am) }

    // Comparisons
    fn cmp(&mut self, am: String) -> String { format!("CMP {}", am) }
    fn cpx(&mut self, am: String) -> String { format!("CPX {}", am) }
    fn cpy(&mut self, am: String) -> String { format!("CPY {}", am) }

    // Bitwise operations
    fn and(&mut self, am: String) -> String { format!("AND {}", am) }
    fn ora(&mut self, am: String) -> String { format!("ORA {}", am) }
    fn eor(&mut self, am: String) -> String { format!("EOR {}", am) }
    fn bit(&mut self, am: String) -> String { format!("BIT {}", am) }

    // Shifts and rotates
    fn rol(&mut self, am: String) -> String { format!("ROL {}", am) }
    fn ror(&mut self, am: String) -> String { format!("ROR {}", am) }
    fn asl(&mut self, am: String) -> String { format!("ASL {}", am) }
    fn lsr(&mut self, am: String) -> String { format!("LSR {}", am) }

    // Increments and decrements
    fn inc(&mut self, am: String) -> String { format!("INC {}", am) }
    fn dec(&mut self, am: String) -> String { format!("DEC {}", am) }
    fn inx(&mut self) -> String           { "INX".to_owned()       }
    fn dex(&mut self) -> String           { "DEX".to_owned()       }
    fn iny(&mut self) -> String           { "INY".to_owned()       }
    fn dey(&mut self) -> String           { "DEY".to_owned()       }

    // Register moves
    fn tax(&mut self) -> String           { "TAX".to_owned()       }
    fn tay(&mut self) -> String           { "TAY".to_owned()       }
    fn txa(&mut self) -> String           { "TXA".to_owned()       }
    fn tya(&mut self) -> String           { "TYA".to_owned()       }
    fn txs(&mut self) -> String           { "TXS".to_owned()       }
    fn tsx(&mut self) -> String           { "TSX".to_owned()       }

    // Flag operations
    fn clc(&mut self) -> String           { "CLC".to_owned()       }
    fn sec(&mut self) -> String           { "SEC".to_owned()       }
    fn cli(&mut self) -> String           { "CLI".to_owned()       }
    fn sei(&mut self) -> String           { "SEI".to_owned()       }
    fn clv(&mut self) -> String           { "CLV".to_owned()       }
    fn cld(&mut self) -> String           { "CLD".to_owned()       }
    fn sed(&mut self) -> String           { "SED".to_owned()       }

    // Branches
    // FIXME: Should disassemble the displacement!
    fn bpl(&mut self) -> String           { "BPL xx".to_owned()    }
    fn bmi(&mut self) -> String           { "BMI xx".to_owned()    }
    fn bvc(&mut self) -> String           { "BVC xx".to_owned()    }
    fn bvs(&mut self) -> String           { "BVS xx".to_owned()    }
    fn bcc(&mut self) -> String           { "BCC xx".to_owned()    }
    fn bcs(&mut self) -> String           { "BCS xx".to_owned()    }
    fn bne(&mut self) -> String           { "BNE xx".to_owned()    }
    fn beq(&mut self) -> String           { "BEQ xx".to_owned()    }

    // Jumps
    // FIXME: Should disassemble the address!
    fn jmp(&mut self) -> String           { format!("JMP {}", self.disw_bump_pc()) }
    fn jmpi(&mut self) -> String          { format!("JMP ({})", self.disw_bump_pc())  }

    // Procedure calls
    // FIXME: Should disassemble the address!
    fn jsr(&mut self) -> String           { "JSR xx".to_owned()    }
    fn rts(&mut self) -> String           { "RTS".to_owned()       }
    fn brk(&mut self) -> String           { "BRK".to_owned()       }
    fn rti(&mut self) -> String           { "RTI".to_owned()       }

    // Stack operations
    fn pha(&mut self) -> String           { "PHA".to_owned()       }
    fn pla(&mut self) -> String           { "PLA".to_owned()       }
    fn php(&mut self) -> String           { "PHP".to_owned()       }
    fn plp(&mut self) -> String           { "PLP".to_owned()       }

    // No operation
    fn nop(&mut self) -> String           { "NOP".to_owned()       }

    // Addressing modes
    fn immediate(&mut self) -> String {
        format!("{}{}", "#", self.disb_bump_pc())
    }
    fn accumulator(&mut self) -> String {
        String::new()
    }
    fn zero_page(&mut self) -> String {
        self.disb_bump_pc()
    }
    fn zero_page_x(&mut self) -> String {
        let mut buf = self.disb_bump_pc();
        buf.push_str(",X");
        buf
    }
    fn zero_page_y(&mut self) -> String {
        let mut buf = self.disb_bump_pc();
        buf.push_str(",Y");
        buf
    }
    fn absolute(&mut self) -> String           { self.disw_bump_pc()                       }
    fn absolute_x(&mut self) -> String {
        let mut buf = self.disw_bump_pc();
        buf.push_str(",X");
        buf
    }
    fn absolute_y(&mut self) -> String {
        let mut buf = self.disw_bump_pc();
        buf.push_str(",Y");
        buf
    }
    fn indexed_indirect_x(&mut self) -> String {
        format!("({},X)", self.disb_bump_pc())
    }
    fn indirect_indexed_y(&mut self) -> String {
        format!("({}),Y", self.disb_bump_pc())
    }

    // The main disassembly routine.
    #[inline(never)]
    pub fn disassemble(&mut self) -> String {
        let op = self.loadb_bump_pc();
        decode_op!(op, self)
    }
}
