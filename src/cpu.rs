use crate::bus::Bus;
pub struct Cpu{
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub f: u8,
    pub sp: u16,
    pub pc: u16,

    pub call_depth: usize,
    pub dbg_pc: Vec<u16>,
    pub wp_pc: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu{
            a: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            f: 0x00,
            sp: 0x0000,
            pc: 0x0000,

            call_depth: 0,
            dbg_pc: Vec::new(),
            wp_pc: 0,
        }
    }
    pub fn after_bootup(&mut self) {
            self.a = 0x01;
            self.f = 0xB0;
            self.b = 0x00;
            self.c = 0x13;
            self.d = 0x00;
            self.e = 0xD8;
            self.h = 0x01;
            self.l = 0x4D;
            self.pc = 0x0100;
            self.sp = 0xFFFE;
    }
    pub fn pushu16(&mut self, bus: &mut Bus, val: u16) {
        self.sp-=1;
        bus.writeu8(self.sp, Cpu::hi_byte(val));
        self.sp-=1;
        bus.writeu8(self.sp, Cpu::lo_byte(val));
    }

    pub fn popu16(&mut self, bus: &mut Bus) -> u16 {
        let result_lo = bus.readu8(self.sp);
        self.sp+=1;
        let result_hi = bus.readu8(self.sp);
        self.sp+=1;
        Cpu::as_word(result_hi,result_lo)
    }

    pub fn clock(&mut self, bus: &mut Bus) -> usize {

        if (bus.iff & bus.ie) != 0 {
            bus.is_cpu_halt = false;
            if bus.ime {
                bus.ime = false;
                if bus.iff & (1 << 0) != 0 {
                    bus.iff &= !(1 << 0);
                    self.pushu16(bus, self.pc);
                    self.pc = 0x0040;
                }else if bus.iff & (1 << 1) != 0{
                    bus.iff &= !(1 << 1);
                    self.pushu16(bus, self.pc);
                    self.pc = 0x0048;
                }else if bus.iff & (1 << 2) != 0 {
                    bus.iff &= !(1 << 2);
                    self.pushu16(bus, self.pc);
                    self.pc = 0x0050;
                }else if bus.iff & (1 << 3) != 0{
                    bus.iff &= !(1 << 3);
                    self.pushu16(bus, self.pc);
                    self.pc = 0x0058;
                }else if bus.iff & (1 << 4) != 0 {
                    bus.iff &= !(1 << 4);
                    self.pushu16(bus, self.pc);
                    self.pc = 0x0060;
                }
                return 5;
            }
            
        }
        
        if bus.is_cpu_halt{
            return 1;
        }
        
        bus.ime = bus.ime || bus.imebuf;
        bus.imebuf = false;
                     
        self.do_instruction(bus)
                     
        
    }

    pub fn do_instruction(&mut self, bus: &mut Bus) -> usize {
        let opcode = bus.readu8(self.pc);
        self.pc+=1;
        match opcode {
            0x00 => 1,
            0x01 => self.pc.ld_r16_imm16(bus, &mut self.b, &mut self.c),
            0x02 => self.ld_r16_indr_a(bus, self.b, self.c),
            0x03 => Cpu::inc_r16(&mut self.b, &mut self.c),
            0x04 => self.f.inc_r8(&mut self.b),
            0x05 => self.f.dec_r8(&mut self.b),
            0x06 => self.pc.ld_r8_imm8(bus, &mut self.b),
            0x07 => self.rlca(),
            0x08 => self.ld_imm16_sp(bus),
            0x09 => self.add_hl_r16(self.b, self.c),
            0x0A => self.ld_a_r16_indr(bus, self.b, self.c),
            0x0B => Cpu::dec_r16(&mut self.b, &mut self.c),
            0x0C => self.f.inc_r8(&mut self.c),
            0x0D => self.f.dec_r8(&mut self.c),
            0x0E => self.pc.ld_r8_imm8(bus, &mut self.c),
            0x0F => self.rrca(),
            0x10 => panic!("Unsupported STOP instruction at {:x}", self.pc-1), //STOP inst needs support if upgrading
            0x11 => self.pc.ld_r16_imm16(bus, &mut self.d, &mut self.e),
            0x12 => self.ld_r16_indr_a(bus, self.d, self.e),
            0x13 => Cpu::inc_r16(&mut self.d, &mut self.e),
            0x14 => self.f.inc_r8(&mut self.d),
            0x15 => self.f.dec_r8(&mut self.d),
            0x16 => self.pc.ld_r8_imm8(bus, &mut self.d),
            0x17 => self.rla(),
            0x18 => self.jr_cc_e8(bus, true),
            0x19 => self.add_hl_r16(self.d, self.e),
            0x1A => self.ld_a_r16_indr(bus, self.d, self.e),
            0x1B => Cpu::dec_r16(&mut self.d, &mut self.e),
            0x1C => self.f.inc_r8(&mut self.e),
            0x1D => self.f.dec_r8(&mut self.e),
            0x1E => self.pc.ld_r8_imm8(bus, &mut self.e),
            0x1F => self.rra(),
            0x20 => self.jr_cc_e8(bus, (self.f & 0x80) == 0),
            0x21 => self.pc.ld_r16_imm16(bus, &mut self.h, &mut self.l),
            0x22 => self.ldi_hl_indr_a(bus),
            0x23 => Cpu::inc_r16(&mut self.h, &mut self.l),
            0x24 => self.f.inc_r8(&mut self.h),
            0x25 => self.f.dec_r8(&mut self.h),
            0x26 => self.pc.ld_r8_imm8(bus, &mut self.h),
            0x27 => self.daa(),
            0x28 => self.jr_cc_e8(bus, (self.f & 0x80) != 0),
            0x29 => self.add_hl_r16(self.h, self.l),
            0x2A => self.ldi_a_hl_indr(bus),
            0x2B => Cpu::dec_r16(&mut self.h, &mut self.l),
            0x2C => self.f.inc_r8(&mut self.l),
            0x2D => self.f.dec_r8(&mut self.l),
            0x2E => self.pc.ld_r8_imm8(bus, &mut self.l),
            0x2F => self.cpl(),
            0x30 => self.jr_cc_e8(bus, (self.f & 0x10) == 0),
            0x31 => self.ld_sp_imm16(bus),
            0x32 => self.ldd_hl_indr_a(bus),
            0x33 => self.inc_sp(),
            0x34 => self.inc_hl_indr(bus),
            0x35 => self.dec_hl_indr(bus),
            0x36 => self.ld_hl_indr_imm8(bus),
            0x37 => self.scf(),
            0x38 => self.jr_cc_e8(bus, (self.f & 0x10) != 0),
            0x39 => self.add_hl_r16(Cpu::hi_byte(self.sp), Cpu::lo_byte(self.sp)),
            0x3A => self.ldd_a_hl_indr(bus),
            0x3B => self.dec_sp(),
            0x3C => self.f.inc_r8(&mut self.a),
            0x3D => self.f.dec_r8(&mut self.a),
            0x3E => self.pc.ld_r8_imm8(bus, &mut self.a),
            0x3F => self.ccf(),
            0x40 => {
                println!("PC: {:04X}, SP: {:04X}, AF: {:04X}, BC: {:04X}, DE: {:04X}, HL {:04X}",
                    self.pc, self.sp, self.af(), self.bc(), self.de(), self.hl()
                );
                self.b.ld_rr(self.b)
            }
            0x41 => self.b.ld_rr(self.c),
            0x42 => self.b.ld_rr(self.d),
            0x43 => self.b.ld_rr(self.e),
            0x44 => self.b.ld_rr(self.h),
            0x45 => self.b.ld_rr(self.l),
            0x46 => self.b.ld_r8_hl_indr(bus, self.h, self.l),
            0x47 => self.b.ld_rr(self.a),
            0x48 => self.c.ld_rr(self.b),
            0x49 => self.c.ld_rr(self.c),
            0x4A => self.c.ld_rr(self.d),
            0x4B => self.c.ld_rr(self.e),
            0x4C => self.c.ld_rr(self.h),
            0x4D => self.c.ld_rr(self.l),
            0x4E => self.c.ld_r8_hl_indr(bus, self.h, self.l),
            0x4F => self.c.ld_rr(self.a),
            0x50 => self.d.ld_rr(self.b),
            0x51 => self.d.ld_rr(self.c),
            0x52 => self.d.ld_rr(self.d),
            0x53 => self.d.ld_rr(self.e),
            0x54 => self.d.ld_rr(self.h),
            0x55 => self.d.ld_rr(self.l),
            0x56 => self.d.ld_r8_hl_indr(bus, self.h, self.l),
            0x57 => self.d.ld_rr(self.a),
            0x58 => self.e.ld_rr(self.b),
            0x59 => self.e.ld_rr(self.c),
            0x5A => self.e.ld_rr(self.d),
            0x5B => self.e.ld_rr(self.e),
            0x5C => self.e.ld_rr(self.h),
            0x5D => self.e.ld_rr(self.l),
            0x5E => self.e.ld_r8_hl_indr(bus, self.h, self.l),
            0x5F => self.e.ld_rr(self.a),
            0x60 => self.h.ld_rr(self.b),
            0x61 => self.h.ld_rr(self.c),
            0x62 => self.h.ld_rr(self.d),
            0x63 => self.h.ld_rr(self.e),
            0x64 => self.h.ld_rr(self.h),
            0x65 => self.h.ld_rr(self.l),
            0x66 => self.h.ld_r8_hl_indr(bus, self.h, self.l),
            0x67 => self.h.ld_rr(self.a),
            0x68 => self.l.ld_rr(self.b),
            0x69 => self.l.ld_rr(self.c),
            0x6A => self.l.ld_rr(self.d),
            0x6B => self.l.ld_rr(self.e),
            0x6C => self.l.ld_rr(self.h),
            0x6D => self.l.ld_rr(self.l),
            0x6E => self.l.ld_r8_hl_indr(bus, self.h, self.l),
            0x6F => self.l.ld_rr(self.a),
            0x70 => self.ld_hl_indr_r8(bus, self.b),
            0x71 => self.ld_hl_indr_r8(bus, self.c),
            0x72 => self.ld_hl_indr_r8(bus, self.d),
            0x73 => self.ld_hl_indr_r8(bus, self.e),
            0x74 => self.ld_hl_indr_r8(bus, self.h),
            0x75 => self.ld_hl_indr_r8(bus, self.l),
            0x76 => self.halt(bus),
            0x77 => self.ld_hl_indr_r8(bus, self.a),
            0x78 => self.a.ld_rr(self.b),
            0x79 => self.a.ld_rr(self.c),
            0x7A => self.a.ld_rr(self.d),
            0x7B => self.a.ld_rr(self.e),
            0x7C => self.a.ld_rr(self.h),
            0x7D => self.a.ld_rr(self.l),
            0x7E => self.a.ld_r8_hl_indr(bus, self.h, self.l),
            0x7F => self.a.ld_rr(self.a),
            0x80 => self.add_r8(self.b),
            0x81 => self.add_r8(self.c),
            0x82 => self.add_r8(self.d),
            0x83 => self.add_r8(self.e),
            0x84 => self.add_r8(self.h),
            0x85 => self.add_r8(self.l),
            0x86 => self.opp_hl_indr(bus, Cpu::add_r8),
            0x87 => self.add_r8(self.a),
            0x88 => self.adc_r8(self.b),
            0x89 => self.adc_r8(self.c),
            0x8A => self.adc_r8(self.d),
            0x8B => self.adc_r8(self.e),
            0x8C => self.adc_r8(self.h),
            0x8D => self.adc_r8(self.l),
            0x8E => self.opp_hl_indr(bus, Cpu::adc_r8),
            0x8F => self.adc_r8(self.a),
            0x90 => self.sub_r8(self.b),
            0x91 => self.sub_r8(self.c),
            0x92 => self.sub_r8(self.d),
            0x93 => self.sub_r8(self.e),
            0x94 => self.sub_r8(self.h),
            0x95 => self.sub_r8(self.l),
            0x96 => self.opp_hl_indr(bus, Cpu::sub_r8),
            0x97 => self.sub_r8(self.a),
            0x98 => self.sbc_r8(self.b),
            0x99 => self.sbc_r8(self.c),
            0x9A => self.sbc_r8(self.d),
            0x9B => self.sbc_r8(self.e),
            0x9C => self.sbc_r8(self.h),
            0x9D => self.sbc_r8(self.l),
            0x9E => self.opp_hl_indr(bus, Cpu::sbc_r8),
            0x9F => self.sbc_r8(self.a),
            0xA0 => self.and_r8(self.b),
            0xA1 => self.and_r8(self.c),
            0xA2 => self.and_r8(self.d),
            0xA3 => self.and_r8(self.e),
            0xA4 => self.and_r8(self.h),
            0xA5 => self.and_r8(self.l),
            0xA6 => self.opp_hl_indr(bus, Cpu::and_r8),
            0xA7 => self.and_r8(self.a),
            0xA8 => self.xor_r8(self.b),
            0xA9 => self.xor_r8(self.c),
            0xAA => self.xor_r8(self.d),
            0xAB => self.xor_r8(self.e),
            0xAC => self.xor_r8(self.h),
            0xAD => self.xor_r8(self.l),
            0xAE => self.opp_hl_indr(bus, Cpu::xor_r8),
            0xAF => self.xor_r8(self.a),
            0xB0 => self.or_r8(self.b),
            0xB1 => self.or_r8(self.c),
            0xB2 => self.or_r8(self.d),
            0xB3 => self.or_r8(self.e),
            0xB4 => self.or_r8(self.h),
            0xB5 => self.or_r8(self.l),
            0xB6 => self.opp_hl_indr(bus, Cpu::or_r8),
            0xB7 => self.or_r8(self.a),
            0xB8 => self.cp_r8(self.b),
            0xB9 => self.cp_r8(self.c),
            0xBA => self.cp_r8(self.d),
            0xBB => self.cp_r8(self.e),
            0xBC => self.cp_r8(self.h),
            0xBD => self.cp_r8(self.l),
            0xBE => self.opp_hl_indr(bus, Cpu::cp_r8),
            0xBF => self.cp_r8(self.a),
            0xC0 => self.ret_cc(bus, (self.f & 0x80) == 0),
            0xC1 => self.sp.pop_r16(bus, &mut self.b, &mut self.c),
            0xC2 => self.jp_cc(bus, (self.f & 0x80) == 0),
            0xC3 => self.jp_cc(bus, true),
            0xC4 => self.call_cc(bus, (self.f & 0x80) == 0),
            0xC5 => self.push_r16(bus, self.b, self.c),
            0xC6 => self.opp_imm8(bus, Cpu::add_r8),
            0xC7 => self.rst(bus, 0x0000),
            0xC8 => self.ret_cc(bus, (self.f & 0x80) != 0),
            0xC9 => self.ret(bus),
            0xCA => self.jp_cc(bus, (self.f & 0x80) != 0),
            0xCB => self.prefix(bus),
            0xCC => self.call_cc(bus, (self.f & 0x80) != 0),
            0xCD => self.call_cc(bus, true),
            0xCE => self.opp_imm8(bus, Cpu::adc_r8),
            0xCF => self.rst(bus, 0x0008),
            0xD0 => self.ret_cc(bus, (self.f & 0x10) == 0),
            0xD1 => self.sp.pop_r16(bus, &mut self.d, &mut self.e),
            0xD2 => self.jp_cc(bus, (self.f & 0x10) == 0),
            0xD3 => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xD4 => self.call_cc(bus, (self.f & 0x10) == 0),
            0xD5 => self.push_r16(bus, self.d, self.e),
            0xD6 => self.opp_imm8(bus, Cpu::sub_r8),
            0xD7 => self.rst(bus, 0x0010),
            0xD8 => self.ret_cc(bus, (self.f & 0x10) != 0),
            0xD9 => self.reti(bus),
            0xDA => self.jp_cc(bus, (self.f & 0x10) != 0),
            0xDB => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xDC => self.call_cc(bus, (self.f & 0x10) != 0),
            0xDD => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xDE => self.opp_imm8(bus, Cpu::sbc_r8),
            0xDF => self.rst(bus, 0x0018),
            0xE0 => self.ldh_imm8_a(bus),
            0xE1 => self.sp.pop_r16(bus, &mut self.h, &mut self.l),
            0xE2 => self.ld_c_indr_a(bus),
            0xE3 => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xE4 => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xE5 => self.push_r16(bus, self.h, self.l),
            0xE6 => self.opp_imm8(bus, Cpu::and_r8),
            0xE7 => self.rst(bus, 0x0020),
            0xE8 => self.add_sp_e8(bus),
            0xE9 => self.jp_hl(),
            0xEA => self.ld_imm16_a(bus),
            0xEB => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xEC => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xED => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xEE => self.opp_imm8(bus, Cpu::xor_r8),
            0xEF => self.rst(bus, 0x0028),
            0xF0 => self.ldh_a_imm8(bus),
            0xF1 => self.sp.pop_r16(bus, &mut self.a, &mut self.f),
            0xF2 => self.ld_a_c_indr(bus),
            0xF3 => self.di(bus),
            0xF4 => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xF5 => self.push_r16(bus, self.a, self.f),
            0xF6 => self.opp_imm8(bus, Cpu::or_r8),
            0xF7 => self.rst(bus, 0x0030),
            0xF8 => self.ld_hl_sp_e8(bus),
            0xF9 => self.ld_sp_hl(),
            0xFA => self.ld_a_imm16(bus),
            0xFB => self.ei(bus),
            0xFC => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xFD => panic!("Invalid instruction opcode {:x} reached at {:x}", opcode, self.pc - 1),
            0xFE => self.opp_imm8(bus, Cpu::cp_r8),
            0xFF => {
                self.rst(bus, 0x0038);
                panic!();
            }
        }
    }

    pub fn prefix(&mut self, bus: &mut Bus) -> usize {
        let opcode = bus.readu8(self.pc);
        self.pc+=1;
        match opcode {
            0x00 => self.b.rlc_r8(&mut self.f),
            0x01 => self.c.rlc_r8(&mut self.f),
            0x02 => self.d.rlc_r8(&mut self.f),
            0x03 => self.e.rlc_r8(&mut self.f),
            0x04 => self.h.rlc_r8(&mut self.f),
            0x05 => self.l.rlc_r8(&mut self.f),
            0x06 => self.shift_hl_indr(bus, CpuReg8::rlc_r8),
            0x07 => self.a.rlc_r8(&mut self.f),
            0x08 => self.b.rrc_r8(&mut self.f),
            0x09 => self.c.rrc_r8(&mut self.f),
            0x0A => self.d.rrc_r8(&mut self.f),
            0x0B => self.e.rrc_r8(&mut self.f),
            0x0C => self.h.rrc_r8(&mut self.f),
            0x0D => self.l.rrc_r8(&mut self.f),
            0x0E => self.shift_hl_indr(bus, CpuReg8::rrc_r8),
            0x0F => self.a.rrc_r8(&mut self.f),
            0x10 => self.b.rl_r8(&mut self.f),
            0x11 => self.c.rl_r8(&mut self.f),
            0x12 => self.d.rl_r8(&mut self.f),
            0x13 => self.e.rl_r8(&mut self.f),
            0x14 => self.h.rl_r8(&mut self.f),
            0x15 => self.l.rl_r8(&mut self.f),
            0x16 => self.shift_hl_indr(bus, CpuReg8::rl_r8),
            0x17 => self.a.rl_r8(&mut self.f),
            0x18 => self.b.rr_r8(&mut self.f),
            0x19 => self.c.rr_r8(&mut self.f),
            0x1A => self.d.rr_r8(&mut self.f),
            0x1B => self.e.rr_r8(&mut self.f),
            0x1C => self.h.rr_r8(&mut self.f),
            0x1D => self.l.rr_r8(&mut self.f),
            0x1E => self.shift_hl_indr(bus, CpuReg8::rr_r8),
            0x1F => self.a.rr_r8(&mut self.f),
            0x20 => self.b.sla_r8(&mut self.f),
            0x21 => self.c.sla_r8(&mut self.f),
            0x22 => self.d.sla_r8(&mut self.f),
            0x23 => self.e.sla_r8(&mut self.f),
            0x24 => self.h.sla_r8(&mut self.f),
            0x25 => self.l.sla_r8(&mut self.f),
            0x26 => self.shift_hl_indr(bus, CpuReg8::sla_r8),
            0x27 => self.a.sla_r8(&mut self.f),
            0x28 => self.b.sra_r8(&mut self.f),
            0x29 => self.c.sra_r8(&mut self.f),
            0x2A => self.d.sra_r8(&mut self.f),
            0x2B => self.e.sra_r8(&mut self.f),
            0x2C => self.h.sra_r8(&mut self.f),
            0x2D => self.l.sra_r8(&mut self.f),
            0x2E => self.shift_hl_indr(bus, CpuReg8::sra_r8),
            0x2F => self.a.sra_r8(&mut self.f),
            0x30 => self.b.swap_r8(&mut self.f),
            0x31 => self.c.swap_r8(&mut self.f),
            0x32 => self.d.swap_r8(&mut self.f),
            0x33 => self.e.swap_r8(&mut self.f),
            0x34 => self.h.swap_r8(&mut self.f),
            0x35 => self.l.swap_r8(&mut self.f),
            0x36 => self.shift_hl_indr(bus, CpuReg8::swap_r8),
            0x37 => self.a.swap_r8(&mut self.f),
            0x38 => self.b.srl_r8(&mut self.f),
            0x39 => self.c.srl_r8(&mut self.f),
            0x3A => self.d.srl_r8(&mut self.f),
            0x3B => self.e.srl_r8(&mut self.f),
            0x3C => self.h.srl_r8(&mut self.f),
            0x3D => self.l.srl_r8(&mut self.f),
            0x3E => self.shift_hl_indr(bus, CpuReg8::srl_r8),
            0x3F => self.a.srl_r8(&mut self.f),
            0x40 => self.bit_u3_r8(0x0, self.b),
            0x41 => self.bit_u3_r8(0x0, self.c),
            0x42 => self.bit_u3_r8(0x0, self.d),
            0x43 => self.bit_u3_r8(0x0, self.e),
            0x44 => self.bit_u3_r8(0x0, self.h),
            0x45 => self.bit_u3_r8(0x0, self.l),
            0x46 => self.bit_hl_indr(bus, 0x0),
            0x47 => self.bit_u3_r8(0x0, self.a),
            0x48 => self.bit_u3_r8(0x1, self.b),
            0x49 => self.bit_u3_r8(0x1, self.c),
            0x4A => self.bit_u3_r8(0x1, self.d),
            0x4B => self.bit_u3_r8(0x1, self.e),
            0x4C => self.bit_u3_r8(0x1, self.h),
            0x4D => self.bit_u3_r8(0x1, self.l),
            0x4E => self.bit_hl_indr(bus, 0x1),
            0x4F => self.bit_u3_r8(0x1, self.a),
            0x50 => self.bit_u3_r8(0x2, self.b),
            0x51 => self.bit_u3_r8(0x2, self.c),
            0x52 => self.bit_u3_r8(0x2, self.d),
            0x53 => self.bit_u3_r8(0x2, self.e),
            0x54 => self.bit_u3_r8(0x2, self.h),
            0x55 => self.bit_u3_r8(0x2, self.l),
            0x56 => self.bit_hl_indr(bus, 0x2),
            0x57 => self.bit_u3_r8(0x2, self.a),
            0x58 => self.bit_u3_r8(0x3, self.b),
            0x59 => self.bit_u3_r8(0x3, self.c),
            0x5A => self.bit_u3_r8(0x3, self.d),
            0x5B => self.bit_u3_r8(0x3, self.e),
            0x5C => self.bit_u3_r8(0x3, self.h),
            0x5D => self.bit_u3_r8(0x3, self.l),
            0x5E => self.bit_hl_indr(bus, 0x3),
            0x5F => self.bit_u3_r8(0x3, self.a),
            0x60 => self.bit_u3_r8(0x4, self.b),
            0x61 => self.bit_u3_r8(0x4, self.c),
            0x62 => self.bit_u3_r8(0x4, self.d),
            0x63 => self.bit_u3_r8(0x4, self.e),
            0x64 => self.bit_u3_r8(0x4, self.h),
            0x65 => self.bit_u3_r8(0x4, self.l),
            0x66 => self.bit_hl_indr(bus, 0x4),
            0x67 => self.bit_u3_r8(0x4, self.a),
            0x68 => self.bit_u3_r8(0x5, self.b),
            0x69 => self.bit_u3_r8(0x5, self.c),
            0x6A => self.bit_u3_r8(0x5, self.d),
            0x6B => self.bit_u3_r8(0x5, self.e),
            0x6C => self.bit_u3_r8(0x5, self.h),
            0x6D => self.bit_u3_r8(0x5, self.l),
            0x6E => self.bit_hl_indr(bus, 0x5),
            0x6F => self.bit_u3_r8(0x5, self.a),
            0x70 => self.bit_u3_r8(0x6, self.b),
            0x71 => self.bit_u3_r8(0x6, self.c),
            0x72 => self.bit_u3_r8(0x6, self.d),
            0x73 => self.bit_u3_r8(0x6, self.e),
            0x74 => self.bit_u3_r8(0x6, self.h),
            0x75 => self.bit_u3_r8(0x6, self.l),
            0x76 => self.bit_hl_indr(bus, 0x6),
            0x77 => self.bit_u3_r8(0x6, self.a),
            0x78 => self.bit_u3_r8(0x7, self.b),
            0x79 => self.bit_u3_r8(0x7, self.c),
            0x7A => self.bit_u3_r8(0x7, self.d),
            0x7B => self.bit_u3_r8(0x7, self.e),
            0x7C => self.bit_u3_r8(0x7, self.h),
            0x7D => self.bit_u3_r8(0x7, self.l),
            0x7E => self.bit_hl_indr(bus, 0x7),
            0x7F => self.bit_u3_r8(0x7, self.a),
            0x80 => self.b.res_u3_r8(0x0),
            0x81 => self.c.res_u3_r8(0x0),
            0x82 => self.d.res_u3_r8(0x0),
            0x83 => self.e.res_u3_r8(0x0),
            0x84 => self.h.res_u3_r8(0x0),
            0x85 => self.l.res_u3_r8(0x0),
            0x86 => self.setres_hl_indr(bus, 0x0, CpuReg8::res_u3_r8),
            0x87 => self.a.res_u3_r8(0x0),
            0x88 => self.b.res_u3_r8(0x1),
            0x89 => self.c.res_u3_r8(0x1),
            0x8A => self.d.res_u3_r8(0x1),
            0x8B => self.e.res_u3_r8(0x1),
            0x8C => self.h.res_u3_r8(0x1),
            0x8D => self.l.res_u3_r8(0x1),
            0x8E => self.setres_hl_indr(bus, 0x1, CpuReg8::res_u3_r8),
            0x8F => self.a.res_u3_r8(0x1),
            0x90 => self.b.res_u3_r8(0x2),
            0x91 => self.c.res_u3_r8(0x2),
            0x92 => self.d.res_u3_r8(0x2),
            0x93 => self.e.res_u3_r8(0x2),
            0x94 => self.h.res_u3_r8(0x2),
            0x95 => self.l.res_u3_r8(0x2),
            0x96 => self.setres_hl_indr(bus, 0x2, CpuReg8::res_u3_r8),
            0x97 => self.a.res_u3_r8(0x2),
            0x98 => self.b.res_u3_r8(0x3),
            0x99 => self.c.res_u3_r8(0x3),
            0x9A => self.d.res_u3_r8(0x3),
            0x9B => self.e.res_u3_r8(0x3),
            0x9C => self.h.res_u3_r8(0x3),
            0x9D => self.l.res_u3_r8(0x3),
            0x9E => self.setres_hl_indr(bus, 0x3, CpuReg8::res_u3_r8),
            0x9F => self.a.res_u3_r8(0x3),
            0xA0 => self.b.res_u3_r8(0x4),
            0xA1 => self.c.res_u3_r8(0x4),
            0xA2 => self.d.res_u3_r8(0x4),
            0xA3 => self.e.res_u3_r8(0x4),
            0xA4 => self.h.res_u3_r8(0x4),
            0xA5 => self.l.res_u3_r8(0x4),
            0xA6 => self.setres_hl_indr(bus, 0x4, CpuReg8::res_u3_r8),
            0xA7 => self.a.res_u3_r8(0x4),
            0xA8 => self.b.res_u3_r8(0x5),
            0xA9 => self.c.res_u3_r8(0x5),
            0xAA => self.d.res_u3_r8(0x5),
            0xAB => self.e.res_u3_r8(0x5),
            0xAC => self.h.res_u3_r8(0x5),
            0xAD => self.l.res_u3_r8(0x5),
            0xAE => self.setres_hl_indr(bus, 0x5, CpuReg8::res_u3_r8),
            0xAF => self.a.res_u3_r8(0x5),
            0xB0 => self.b.res_u3_r8(0x6),
            0xB1 => self.c.res_u3_r8(0x6),
            0xB2 => self.d.res_u3_r8(0x6),
            0xB3 => self.e.res_u3_r8(0x6),
            0xB4 => self.h.res_u3_r8(0x6),
            0xB5 => self.l.res_u3_r8(0x6),
            0xB6 => self.setres_hl_indr(bus, 0x6, CpuReg8::res_u3_r8),
            0xB7 => self.a.res_u3_r8(0x6),
            0xB8 => self.b.res_u3_r8(0x7),
            0xB9 => self.c.res_u3_r8(0x7),
            0xBA => self.d.res_u3_r8(0x7),
            0xBB => self.e.res_u3_r8(0x7),
            0xBC => self.h.res_u3_r8(0x7),
            0xBD => self.l.res_u3_r8(0x7),
            0xBE => self.setres_hl_indr(bus, 0x7, CpuReg8::res_u3_r8),
            0xBF => self.a.res_u3_r8(0x7),
            0xC0 => self.b.set_u3_r8(0x0),
            0xC1 => self.c.set_u3_r8(0x0),
            0xC2 => self.d.set_u3_r8(0x0),
            0xC3 => self.e.set_u3_r8(0x0),
            0xC4 => self.h.set_u3_r8(0x0),
            0xC5 => self.l.set_u3_r8(0x0),
            0xC6 => self.setres_hl_indr(bus, 0x0, CpuReg8::set_u3_r8),
            0xC7 => self.a.set_u3_r8(0x0),
            0xC8 => self.b.set_u3_r8(0x1),
            0xC9 => self.c.set_u3_r8(0x1),
            0xCA => self.d.set_u3_r8(0x1),
            0xCB => self.e.set_u3_r8(0x1),
            0xCC => self.h.set_u3_r8(0x1),
            0xCD => self.l.set_u3_r8(0x1),
            0xCE => self.setres_hl_indr(bus, 0x1, CpuReg8::set_u3_r8),
            0xCF => self.a.set_u3_r8(0x1),
            0xD0 => self.b.set_u3_r8(0x2),
            0xD1 => self.c.set_u3_r8(0x2),
            0xD2 => self.d.set_u3_r8(0x2),
            0xD3 => self.e.set_u3_r8(0x2),
            0xD4 => self.h.set_u3_r8(0x2),
            0xD5 => self.l.set_u3_r8(0x2),
            0xD6 => self.setres_hl_indr(bus, 0x2, CpuReg8::set_u3_r8),
            0xD7 => self.a.set_u3_r8(0x2),
            0xD8 => self.b.set_u3_r8(0x3),
            0xD9 => self.c.set_u3_r8(0x3),
            0xDA => self.d.set_u3_r8(0x3),
            0xDB => self.e.set_u3_r8(0x3),
            0xDC => self.h.set_u3_r8(0x3),
            0xDD => self.l.set_u3_r8(0x3),
            0xDE => self.setres_hl_indr(bus, 0x3, CpuReg8::set_u3_r8),
            0xDF => self.a.set_u3_r8(0x3),
            0xE0 => self.b.set_u3_r8(0x4),
            0xE1 => self.c.set_u3_r8(0x4),
            0xE2 => self.d.set_u3_r8(0x4),
            0xE3 => self.e.set_u3_r8(0x4),
            0xE4 => self.h.set_u3_r8(0x4),
            0xE5 => self.l.set_u3_r8(0x4),
            0xE6 => self.setres_hl_indr(bus, 0x4, CpuReg8::set_u3_r8),
            0xE7 => self.a.set_u3_r8(0x4),
            0xE8 => self.b.set_u3_r8(0x5),
            0xE9 => self.c.set_u3_r8(0x5),
            0xEA => self.d.set_u3_r8(0x5),
            0xEB => self.e.set_u3_r8(0x5),
            0xEC => self.h.set_u3_r8(0x5),
            0xED => self.l.set_u3_r8(0x5),
            0xEE => self.setres_hl_indr(bus, 0x5, CpuReg8::set_u3_r8),
            0xEF => self.a.set_u3_r8(0x5),
            0xF0 => self.b.set_u3_r8(0x6),
            0xF1 => self.c.set_u3_r8(0x6),
            0xF2 => self.d.set_u3_r8(0x6),
            0xF3 => self.e.set_u3_r8(0x6),
            0xF4 => self.h.set_u3_r8(0x6),
            0xF5 => self.l.set_u3_r8(0x6),
            0xF6 => self.setres_hl_indr(bus, 0x6, CpuReg8::set_u3_r8),
            0xF7 => self.a.set_u3_r8(0x6),
            0xF8 => self.b.set_u3_r8(0x7),
            0xF9 => self.c.set_u3_r8(0x7),
            0xFA => self.d.set_u3_r8(0x7),
            0xFB => self.e.set_u3_r8(0x7),
            0xFC => self.h.set_u3_r8(0x7),
            0xFD => self.l.set_u3_r8(0x7),
            0xFE => self.setres_hl_indr(bus, 0x7, CpuReg8::set_u3_r8),
            0xFF => self.a.set_u3_r8(0x7),
        }
    }

    pub fn as_word(hi: u8, lo: u8) -> u16{
        (u16::from(hi) << 8) | (u16::from(lo))
    }
    pub fn hi_byte(word: u16) -> u8{
        ((word & 0xFF00) >> 8) as u8
    }
    pub fn lo_byte(word: u16) -> u8{
        (word & 0x00FF) as u8
    }
    pub fn as_bytes(word: u16) -> (u8, u8){
        (Cpu::hi_byte(word), Cpu::lo_byte(word))
    }

    pub fn inc_r16(r16hi: &mut u8, r16lo: &mut u8) -> usize {
        let mut r16 = Cpu::as_word(*r16hi, *r16lo);
        r16 = r16.wrapping_add(1);
        *r16hi = Cpu::hi_byte(r16);
        *r16lo = Cpu::lo_byte(r16);
        2
    }

    pub fn dec_r16(r16hi: &mut u8, r16lo: &mut u8) -> usize {
        let mut r16 = Cpu::as_word(*r16hi, *r16lo);
        r16 = r16.wrapping_sub(1);
        *r16hi = Cpu::hi_byte(r16);
        *r16lo = Cpu::lo_byte(r16);
        2
    }
    pub fn inc_hl_indr(&mut self, bus: &mut Bus) -> usize{
        let hl = Cpu::as_word(self.h, self.l);
        let mut byte = bus.readu8(hl);
        let _ = self.f.inc_r8(&mut byte);
        bus.writeu8(hl, byte);
        3 
    }
    pub fn dec_hl_indr(&mut self, bus: &mut Bus) -> usize {
        let hl = Cpu::as_word(self.h, self.l);
        let mut byte = bus.readu8(hl);
        let _ = self.f.dec_r8(&mut byte);
        bus.writeu8(hl, byte);
        3
    }
    pub fn inc_sp(&mut self) -> usize {
        self.sp = self.sp.wrapping_add(1);
        2
    }
    pub fn dec_sp(&mut self) -> usize {
        self.sp = self.sp.wrapping_sub(1);
        2
    }
    pub fn add_sp_e8(&mut self, bus: &mut Bus) -> usize {
        let e8 = bus.readu8(self.pc) as i8 as i16 as u16;
        self.pc+=1;
        let halfcarry = u8::from((((self.sp & 0x000F) + (e8 & 0x000F)) & 0x0010) == 0x10) << 5;
        let carry = u8::from((((self.sp & 0x00FF) + (e8 & 0x00FF)) & 0x0100) == 0x0100) << 4;
        self.sp = self.sp.wrapping_add(e8);
        self.f = (halfcarry | carry) & !((1 << 7) | (1 << 6));
        4
    }
    pub fn ld_hl_sp_e8(&mut self, bus: &mut Bus) -> usize {
        let e8 = bus.readu8(self.pc) as i8 as i16 as u16;
        self.pc+=1;
        let halfcarry = u8::from((((self.sp & 0x000F) + (e8 & 0x000F)) & 0x0010) == 0x0010) << 5;
        let carry = u8::from((((self.sp & 0x00FF) + (e8 & 0x00FF)) & 0x0100) == 0x0100) << 4;
        self.h = Cpu::hi_byte(self.sp.wrapping_add(e8));
        self.l = Cpu::lo_byte(self.sp.wrapping_add(e8));
        self.f = (halfcarry | carry) & !((1 << 7) | (1 <<6));
        3
    }
    pub fn ld_sp_hl(&mut self) -> usize {
        self.sp = Cpu::as_word(self.h, self.l);
        2
    }
    
    
    pub fn ld_rr(&mut self, rdest: &mut u8, rsrc: u8) -> usize {
        *rdest = rsrc;
        1
    }
    pub fn ld_r16_indr_a(&mut self, bus: &mut Bus, r16hi: u8, r16lo: u8) -> usize{
        let r16 = Cpu::as_word(r16hi, r16lo);
        bus.writeu8(r16, self.a);
        2
    }
    pub fn ld_hl_indr_r8(&mut self, bus: &mut Bus, r8: u8) -> usize {
        let hl = Cpu::as_word(self.h, self.l);
        bus.writeu8(hl, r8);
        2
    }
    pub fn ld_hl_indr_imm8(&mut self, bus: &mut Bus) -> usize {
        let byte = bus.readu8(self.pc);
        self.pc+=1;
        let _ = self.ld_hl_indr_r8(bus, byte);
        3
    }
    pub fn ldi_hl_indr_a(&mut self, bus: &mut Bus) -> usize {
        let mut hl = Cpu::as_word(self.h, self.l);
        bus.writeu8(hl, self.a);
        hl = hl.wrapping_add(1);
        (self.h, self.l) = Cpu::as_bytes(hl);
        2
    }
    pub fn ldi_a_hl_indr(&mut self, bus: &mut Bus) -> usize {
        let mut hl = Cpu::as_word(self.h, self.l);
        self.a = bus.readu8(hl);
        hl = hl.wrapping_add(1);
        (self.h, self.l) = Cpu::as_bytes(hl);
        2
    }
    pub fn ldd_hl_indr_a(&mut self, bus: &mut Bus) -> usize {
        let mut hl = Cpu::as_word(self.h, self.l);
        bus.writeu8(hl, self.a);
        hl = hl.wrapping_sub(1);
        (self.h, self.l) = Cpu::as_bytes(hl);
        2
    }
    pub fn ldd_a_hl_indr(&mut self, bus: &mut Bus) -> usize {
        let mut hl = Cpu::as_word(self.h, self.l);
        self.a = bus.readu8(hl);
        hl = hl.wrapping_sub(1);
        (self.h, self.l) = Cpu::as_bytes(hl);
        2
    }
    pub fn ld_sp_imm16(&mut self, bus: &mut Bus) -> usize {
        let sp_lo = bus.readu8(self.pc);
        self.pc+=1;
        let sp_hi = bus.readu8(self.pc);
        self.pc+=1;
        self.sp = Cpu::as_word(sp_hi, sp_lo);
        3
    }
    pub fn ld_a_r16_indr(&mut self, bus: &mut Bus, r16hi: u8, r16lo: u8) -> usize {
        let r16 = Cpu::as_word(r16hi, r16lo);
        self.a = bus.readu8(r16);
        2
    }
    
    pub fn ld_imm16_sp(&mut self, bus: &mut Bus) -> usize {
        let addr_lo = bus.readu8(self.pc);
        self.pc+=1;
        let addr_hi = bus.readu8(self.pc);
        self.pc+=1;
        let addr = Cpu::as_word(addr_hi, addr_lo);
        bus.writeu8(addr,Cpu::lo_byte(self.sp));
        bus.writeu8(addr+1,Cpu::hi_byte(self.sp));
        5
    }
    pub fn ldh_imm8_a(&mut self, bus: &mut Bus) -> usize {
        let offset = bus.readu8(self.pc) as u16;
        self.pc+=1;
        bus.writeu8(0xFF00 + offset, self.a);
        3
    }
    pub fn ldh_a_imm8(&mut self, bus: &mut Bus) -> usize {
        let offset = bus.readu8(self.pc) as u16;
        self.pc+=1;
        self.a = bus.readu8(0xFF00 + offset);
        3 
    }
    pub fn ld_c_indr_a(&mut self, bus: &mut Bus) -> usize {
        bus.writeu8(0xFF00 + (self.c as u16), self.a);
        2
    }
    pub fn ld_a_c_indr(&mut self, bus: &mut Bus) -> usize {
        self.a = bus.readu8(0xFF00 + (self.c as u16));
        2
    }
    pub fn ld_imm16_a(&mut self, bus: &mut Bus) -> usize {
        let addrlo = bus.readu8(self.pc);
        self.pc+=1;
        let addrhi = bus.readu8(self.pc);
        self.pc+=1;
        bus.writeu8(Cpu::as_word(addrhi, addrlo), self.a);
        4
    }
    pub fn ld_a_imm16(&mut self, bus: &mut Bus) -> usize {
        let addrlo = bus.readu8(self.pc);
        self.pc+=1;
        let addrhi = bus.readu8(self.pc);
        self.pc+=1;
        self.a = bus.readu8(Cpu::as_word(addrhi, addrlo));
        4
    }
    pub fn push_r16(&mut self, bus: &mut Bus, r16hi: u8, r16lo: u8) -> usize {
        self.sp-=1;
        bus.writeu8(self.sp, r16hi);
        self.sp-=1;
        bus.writeu8(self.sp, r16lo);
        4
    }
    
    pub fn add_hl_r16(&mut self, r16hi: u8, r16lo: u8) -> usize{
        let mut hl = Cpu::as_word(self.h,self.l);
        let r16 = Cpu::as_word(r16hi, r16lo);
        let halfcarry = u8::from((((hl & 0x0FFF)+(r16 & 0x0FFF)) & 0x1000) == 0x1000) << 5;
        let carry = u8::from(((u32::from(hl)+u32::from(r16)) & 0x00010000) == 0x00010000) << 4;
        hl = hl.wrapping_add(r16);
        self.h = Cpu::hi_byte(hl);
        self.l = Cpu::lo_byte(hl);
        self.f = ((self.f & 0x80) | halfcarry | carry) & !(1 << 6);
        2
    }
    pub fn add_r8(&mut self, r8: u8) -> usize {
        let halfcarry = u8::from((((self.a & 0x0F) + (r8 & 0x0F)) & 0x10) == 0x10) << 5;
        let carry = u8::from(((u16::from(self.a) + u16::from(r8)) & 0x0100) == 0x0100) << 4;
        self.a = self.a.wrapping_add(r8);
        self.f = ((u8::from(self.a == 0) << 7) | halfcarry | carry) & !(1 << 6);
        1
    }
    pub fn adc_r8(&mut self, r8: u8) -> usize {
        let carryin = (self.f & 0x10) >> 4;
        let halfcarry = u8::from((((self.a & 0x0F) + (r8 & 0x0F) + carryin) & 0x10) == 0x10) << 5;
        let carryout = u8::from(((u16::from(self.a) + u16::from(r8) + u16::from(carryin)) & 0x0100) == 0x0100) << 4;
        self.a = self.a.wrapping_add(r8.wrapping_add(carryin));
        self.f = ((u8::from(self.a == 0) << 7) | halfcarry | carryout) & !(1 << 6);
        1
    }
    pub fn sub_r8(&mut self, r8: u8) -> usize {
        let halfcarry = u8::from(((self.a & 0x0F).wrapping_sub(r8 & 0x0F) & 0x10) != 0) << 5;
        let carry = u8::from(self.a < r8) << 4;
        self.a = self.a.wrapping_sub(r8);
        self.f = (u8::from(self.a == 0) << 7) | (1 << 6) | halfcarry | carry;
        1
    }
    pub fn sbc_r8(&mut self, r8: u8) -> usize {
        let carryin = (self.f & 0x10) >> 4;
        let halfcarry = u8::from(((self.a & 0x0F).wrapping_sub(r8 & 0x0F).wrapping_sub(carryin) & 0x10) != 0) << 5;
        let carryout = u8::from((self.a as u16) < ((r8 as u16) + (carryin as u16))) << 4;
        self.a = self.a.wrapping_sub(r8).wrapping_sub(carryin);
        self.f = (u8::from(self.a == 0) << 7) | (1 << 7) | halfcarry | carryout;
        1
    }
    pub fn and_r8(&mut self, r8: u8) -> usize {
        self.a &= r8;
        self.f = ((u8::from(self.a == 0) << 7) | (1 << 5)) & !((1 << 6) | (1 << 4));
        1
    }
    pub fn xor_r8(&mut self, r8: u8) -> usize {
        self.a ^= r8;
        self.f = (u8::from(self.a == 0) << 7) & !((1 << 6) | (1 << 5) | (1 << 4));
        1
    }
    pub fn or_r8(&mut self, r8: u8) -> usize {
        self.a |= r8;
        self.f = (u8::from(self.a == 0) << 7) & !((1 << 6) | (1 << 5) | (1 << 4));
        1
    }
    pub fn cp_r8(&mut self, r8: u8) -> usize {
        let zeroflag = (u8::from(self.a.wrapping_sub(r8) == 0)) << 7;
        let halfcarry = u8::from((self.a & 0x0F) < (r8 & 0x0F)) << 5;
        let carry = u8::from(self.a < r8) << 4;
        self.f = zeroflag | (1 << 6) | halfcarry | carry;
        1
    }
    pub fn opp_hl_indr(&mut self, bus: &mut Bus, func: fn(&mut Cpu, u8) -> usize) -> usize {
        let hl = Cpu::as_word(self.h, self.l);
        let byte = bus.readu8(hl);
        let _ = func(self, byte);
        2
    }
    pub fn opp_imm8(&mut self, bus: &mut Bus, func: fn(&mut Cpu, u8) -> usize) -> usize {
        let byte = bus.readu8(self.pc);
        self.pc+=1;
        let _ = func(self, byte);
        2
    }
    pub fn daa(&mut self) -> usize {
        if self.f & (1 << 6) == 0 {
            if (self.f & (1 << 4) != 0) || (self.a > 0x99) {
                self.a = self.a.wrapping_add(0x60);
                self.f |= 1 << 4;
            }
            if (self.f & (1 << 5) != 0) || ((self.a & 0x0F) > 0x09){
                self.a = self.a.wrapping_add(0x06);
            }
        }else {
            if self.f & (1 << 4) != 0 {
                self.a = self.a.wrapping_sub(0x60);
            }
            if self.f & (1 << 5) != 0{
                self.a = self.a.wrapping_sub(0x06);
            }
        }
        self.f = ((self.f & 0x50) | (u8::from(self.a == 0) << 7)) & !(1 << 5);
        1
    }
    pub fn cpl(&mut self) -> usize {
        self.a = !(self.a);
        self.f |= (1 << 6) | (1 << 5);
        1
    }
    pub fn scf(&mut self) -> usize {
        self.f = ((self.f  & 0x80) | (1 << 4)) & !(0x60);
        1
    }
    pub fn ccf(&mut self) -> usize {
        self.f ^= 1 << 4;
        1
    }
    pub fn rlca(&mut self) -> usize {
        let carry = (self.a & 0x80) >> 3;
        self.a = ((self.a & 0x7F) << 1)|((self.a & 0x80) >> 7);
        self.f = carry & !(0xE0);
        1
    }
    pub fn rrca(&mut self) -> usize {
        let carry = (self.a & 0x01) << 4;
        self.a = ((self.a & 0x01) << 7) | ((self.a & 0xFE) >> 1);
        self.f = carry & !(0xE0);
        1
    }
    pub fn rla(&mut self) -> usize {
        let carry = (self.a & 0x80) >> 3;
        self.a = ((self.a & 0x7F) << 1)|((self.f & 0x10) >> 4);
        self.f = carry & !(0xE0);
        1
    }
    pub fn rra(&mut self) -> usize {
        let carry = (self.a & 0x01) << 4;
        self.a = ((self.a & 0xFE) >> 1)|((self.f & 0x10) << 3);
        self.f = carry & !(0xE0);
        1 
    }

    pub fn bit_u3_r8(&mut self, u3: u8, r8: u8) -> usize {
        let zeroflag = u8::from((r8 & (1 << (u3 & 0x07))) == 0) << 7;
        self.f = zeroflag | (1 << 5) | (self.f & 0x10);
        2
    }
    
    pub fn shift_hl_indr(&mut self, bus: &mut Bus, func: fn(&mut u8, &mut u8) -> usize) -> usize {
        let hl = Cpu::as_word(self.h, self.l);
        let mut byte = bus.readu8(hl);
        let _ = func(&mut byte, &mut self.f);
        bus.writeu8(hl, byte);
        4
    }
    pub fn bit_hl_indr(&mut self, bus: &mut Bus, u3: u8) -> usize {
        let hl = Cpu::as_word(self.h, self.l);
        let byte = bus.readu8(hl);
        let _ = self.bit_u3_r8(u3, byte);
        3
    }
    pub fn setres_hl_indr(&mut self, bus: &mut Bus, u3: u8, func: fn(&mut u8, u8) -> usize) -> usize {
        let hl = Cpu::as_word(self.h, self.l);
        let mut byte = bus.readu8(hl);
        let _ = func(&mut byte, u3);
        bus.writeu8(hl, byte);
        4
    }
    pub fn jr_cc_e8(&mut self, bus: &mut Bus, cc: bool) -> usize {
        let e8 = bus.readu8(self.pc) as i8;
        self.pc += 1;
        if cc {
            self.pc = self.pc.wrapping_add(e8 as u16);
            return 3;
        }
        2
    }
    pub fn jp_cc(&mut self, bus: &mut Bus, cc: bool) -> usize {
        let addrlo = bus.readu8(self.pc);
        self.pc+=1;
        let addrhi = bus.readu8(self.pc);
        self.pc+=1;
        if cc {
            self.pc = Cpu::as_word(addrhi, addrlo);
            return 4;
        }
        3
    }
    pub fn jp_hl(&mut self) -> usize {
        self.pc = Cpu::as_word(self.h, self.l);
        1
    }
    pub fn call_cc(&mut self, bus: &mut Bus, cc: bool) -> usize {
        let addrlo = bus.readu8(self.pc);
        self.pc+=1;
        let addrhi = bus.readu8(self.pc);
        self.pc+=1;
        if cc {
            self.sp-=1;
            bus.writeu8(self.sp, Cpu::hi_byte(self.pc));
            self.sp-=1;
            bus.writeu8(self.sp, Cpu::lo_byte(self.pc));
            self.pc = Cpu::as_word(addrhi, addrlo);
            return 6;
        }
        return 3;
    }
    pub fn ret_cc(&mut self, bus: &mut Bus, cc: bool) -> usize {
        if cc {
            let pclo = bus.readu8(self.sp);
            self.sp+=1;
            let pchi = bus.readu8(self.sp);
            self.sp+=1;
            self.pc = Cpu::as_word(pchi, pclo);
            return 5;
        }
        2
    }
    pub fn ret(&mut self, bus: &mut Bus) -> usize {
        let pclo = bus.readu8(self.sp);
        self.sp+=1;
        let pchi = bus.readu8(self.sp);
        self.sp+=1;
        self.pc = Cpu::as_word(pchi, pclo);
        4
    }
    pub fn rst(&mut self, bus: &mut Bus, vec: u16) -> usize {
        self.sp = self.sp.wrapping_sub(1);
        bus.writeu8(self.sp, Cpu::hi_byte(self.pc));
        self.sp = self.sp.wrapping_sub(1);
        bus.writeu8(self.sp, Cpu::lo_byte(self.pc));
        self.pc = vec;
        4
    }
    pub fn di(&mut self, bus: &mut Bus) -> usize {
        bus.ime = false;
        1
    }
    pub fn ei(&mut self, bus: &mut Bus) -> usize {
        // println!("Interrupts enabled \
        //     PC: {:04X}, SP: {:04X}, AF: {:04X}, BC: {:04X}, DE: {:04X}, HL {:04X}",
        //     self.pc, self.sp, self.af(), self.bc(), self.de(), self.hl()
        // );
        bus.imebuf = true;
        1
    }
    pub fn reti(&mut self, bus: &mut Bus) -> usize {
        bus.ime = true;
        self.ret(bus)
    }
    pub fn halt(&mut self, bus: &mut Bus) -> usize {
        bus.is_cpu_halt = true;
        1
    }
    pub fn af(&self) -> u16 {
        Cpu::as_word(self.a, self.f)
    }
    pub fn bc(&self) -> u16 {
        Cpu::as_word(self.b, self.c)
    }
    pub fn de(&self) -> u16 {
        Cpu::as_word(self.d, self.e)
    }
    pub fn hl(&self) -> u16 {
        Cpu::as_word(self.h, self.l)
    }
    pub fn debug_print(&self, bus: &mut Bus) {
        let op1 = bus.readu8(self.pc);
        let op2 = bus.readu8(self.pc+1);
        let op3 = bus.readu8(self.pc+2);
        println!("PC: {:04X}, SP: {:04X}, AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, if: {:02X}, ie: {:02X}, ime: {}  | {}",
            self.pc, self.sp, self.af(), self.bc(), self.de(), self.hl(), bus.iff, bus.ie, bus.ime, disassemble(op1, op2, op3)
        );   
    }
}
pub trait CpuReg16 {
    fn ld_r16_imm16(&mut self, bus: &mut Bus, r16hi: &mut u8, r16lo: &mut u8) -> usize;
    fn ld_r8_imm8(&mut self, bus: &mut Bus, r8: &mut u8) -> usize;
    fn pop_r16(&mut self, bus: &mut Bus, r16hi: &mut u8, r16lo: &mut u8) -> usize;
}
impl CpuReg16 for u16 {
    fn ld_r16_imm16(&mut self, bus: &mut Bus, r16hi: &mut u8, r16lo: &mut u8) -> usize {
        *r16lo = bus.readu8(*self);
        *self+=1;
        *r16hi = bus.readu8(*self);
        *self+=1;
        3
    }

    fn ld_r8_imm8(&mut self, bus: &mut Bus, r8: &mut u8) -> usize {
        *r8 = bus.readu8(*self);
        *self+=1;
        2
    }
    
    fn pop_r16(&mut self, bus: &mut Bus, r16hi: &mut u8, r16lo: &mut u8) -> usize {
        *r16lo = bus.readu8(*self);
        *self+=1;
        *r16hi = bus.readu8(*self);
        *self+=1;
        3
    }
}
pub trait CpuFlag {
    fn inc_r8(&mut self, r8: &mut u8) -> usize;
    fn dec_r8(&mut self, r8: &mut u8) -> usize;
}
impl CpuFlag for u8 {
    fn inc_r8(&mut self, r8: &mut u8) -> usize {
        let halfcarry = u8::from((((*r8 & 0x0F)+1) & 0x10 ) == 0x10) << 5;
        *r8 = (*r8).wrapping_add(1);
        *self = ((*self & 0x10) | (u8::from(*r8 == 0) << 7) | halfcarry) & !(1 << 6);
        1
    }
    fn dec_r8(&mut self, r8: &mut u8) -> usize {
        let halfcarry = u8::from(((*r8 & 0x0F).wrapping_sub(1) & 0x10) == 0x10) << 5;
        *r8 = (*r8).wrapping_sub(1);
        *self = (*self & 0x10) | (u8::from(*r8 == 0) << 7) | (1 << 6) | halfcarry;
        1
    }
}
pub trait CpuReg8 {
    fn ld_rr(&mut self, src: u8) -> usize;
    fn ld_r8_hl_indr(&mut self, bus: &mut Bus, regh: u8, regl: u8) -> usize;
    fn rlc_r8(&mut self, regf: &mut u8) -> usize;
    fn rrc_r8(&mut self, regf: &mut u8) -> usize;
    fn rl_r8(&mut self, regf: &mut u8) -> usize;
    fn rr_r8(&mut self, regf: &mut u8) -> usize;
    fn sla_r8(&mut self, regf: &mut u8) -> usize;
    fn sra_r8(&mut self, regf: &mut u8) -> usize;
    fn swap_r8(&mut self, regf: &mut u8) -> usize;
    fn srl_r8(&mut self, regf: &mut u8) -> usize;
    fn res_u3_r8(&mut self, u3: u8) -> usize;
    fn set_u3_r8(&mut self, u3: u8) -> usize;
}
impl CpuReg8 for u8 {
    fn ld_r8_hl_indr(&mut self, bus: &mut Bus, regh: u8, regl: u8) -> usize {
        let hl = Cpu::as_word(regh, regl);
        *self = bus.readu8(hl);
        2
    }
    fn ld_rr(&mut self, src: u8) -> usize {
        *self = src;
        1
    }
    fn rlc_r8(&mut self, regf: &mut u8) -> usize {
        let carry = (*self & 0x80) >> 3;
        *self = ((*self & 0x7F) << 1) | ((*self & 0x80) >> 7);
        *regf = ( (u8::from(*self == 0) << 7) | carry ) & !((1 << 6) | (1 << 5));
        2
    }
    fn rrc_r8(&mut self, regf: &mut u8) -> usize {
        let carry = (*self & 0x01) << 4;
        *self = ((*self & 0x01) << 7) | ((*self & 0xFE) >> 1);
        *regf = ( (u8::from(*self == 0) << 7) | carry ) & !((1 << 6) | (1 << 5));
        2
    }
    fn rl_r8(&mut self, regf: &mut u8) -> usize {
        let carry = (*self & 0x80) >> 3;
        *self = ((*self & 0x7F) << 1) | ((*regf & 0x10) >> 4);
        *regf = ((u8::from(*self == 0) << 7) | carry) & !((1 << 6) | (1 << 5));
        2
    }
    fn rr_r8(&mut self, regf: &mut u8) -> usize {
        let carry = (*self & 0x01) << 4;
        *self = ((*self & 0xFE) >> 1) | ((*regf & 0x10) << 3);
        *regf = ((u8::from(*self == 0) << 7) | carry) & !((1 << 6) | (1 << 5));
        2
    }
    fn sla_r8(&mut self, regf: &mut u8) -> usize {
        let carry = (*self & 0x80) >> 3;
        *self = (*self & 0x7F) << 1;
        *regf = ((u8::from(*self == 0) << 7) | carry) & !((1 << 6) | (1 << 5));
        2
    }
    fn sra_r8(&mut self, regf: &mut u8) -> usize {
        let carry = (*self & 0x01) << 4;
        *self = (*self & 0x80) | ((*self & 0xFE) >> 1);
        *regf = ((u8::from(*self == 0) << 7) | carry) & !((1 << 6) | (1 << 5));
        2
    }
    fn swap_r8(&mut self, regf: &mut u8) -> usize {
        *self = ((*self & 0xF0) >> 4) | ((*self & 0x0F) << 4);
        *regf = u8::from(*self == 0) << 7;
        2
    }
    fn srl_r8(&mut self, regf: &mut u8) -> usize {
        let carry = (*self & 0x01) << 4;
        *self = (*self & 0xFE) >> 1;
        *regf = (u8::from(*self == 0) << 7) | carry;
        2
    }

    fn res_u3_r8(&mut self, u3: u8) -> usize {
        *self &= !(1 << (u3 & 0x07));
        2 
    }
    fn set_u3_r8(&mut self, u3: u8) -> usize {
        *self |= 1 << (u3 & 0x07);
        2
    }
}

pub fn disassemble(op1: u8, op2: u8, op3: u8) -> String {
    if op1 == 0xCB {
        return CB_PREFIX[op2 as usize].to_owned();
    }
    if OPCODES[op1 as usize].contains("u8") {
        return OPCODES[op1 as usize].replace("u8",format!("{:#04X}", op2).as_str());
    }
    if OPCODES[op1 as usize].contains("u16") {
        return OPCODES[op1 as usize].replace("u16", format!("{:#06X}", u16::from_le_bytes([op2, op3])).as_str());
    }
    if OPCODES[op1 as usize].contains("i8") {
        let sign = if op2 == 0 {""} else if op2 > 127 {"-"} else {"+"};
        let signed = (op2 as i8).abs_diff(0);  
        return OPCODES[op1 as usize].replace("i8", format!("{sign}{signed:#04X}").as_str());
    }
    OPCODES[op1 as usize].to_owned()
}

pub const OPCODES: &[&'static str; 256] = &[
//  x0/x8         x1/x9         x2/xA         x3/xB         x4/xC         x5/xD         x6/xE         x7/xF    
    "NOP"        ,"LD BC,u16"  ,"LD [BC],A"  ,"INC BC"     ,"INC B"      ,"DEC B"      ,"LD B,u8"    ,"RLCA"       , // 0x
    "LD [u16],SP","ADD HL,BC"  ,"LD A,[BC]"  ,"DEC BC"     ,"INC C"      ,"DEC C"      ,"LD C,u8"    ,"RRCA"       , 
    "STOP u8"    ,"LD DE,u16"  ,"LD [DE],A"  ,"INC DE"     ,"INC D"      ,"DEC D"      ,"LD D,u8"    ,"RLA"        , // 1x
    "JR i8"      ,"ADD HL,DE"  ,"LD A,[DE]"  ,"DEC DE"     ,"INC E"      ,"DEC E"      ,"LD E,u8"    ,"RRA"        , 
    "JR NZ i8"   ,"LD HL u16"  ,"LD [HLI],A" ,"INC HL"     ,"INC H"      ,"DEC H"      ,"LD H,u8"    ,"DAA"        , // 2x
    "JR Z i8"    ,"ADD HL,HL"  ,"LD A,[HLI]" ,"DEC HL"     ,"INC L"      ,"DEC L"      ,"LD L,u8"    ,"CPL"        ,
    "JR NC i8"   ,"LD SP,u16"  ,"LD [HLD],A" ,"INC SP"     ,"INC [HL]"   ,"DEC [HL]"   ,"LD [HL],u8" ,"SCF"        , // 3x
    "JR C i8"    ,"ADD HL,SP"  ,"LD A,[HLD]" ,"DEC SP"     ,"INC A"      ,"DEC A"      ,"LD A,u8"    ,"CCF"        ,
    "LD B,B"     ,"LD B,C"     ,"LD B,D"     ,"LD B,E"     ,"LD B,H"     ,"LD B,L"     ,"LD B,[HL]"  ,"LD B,A"     , // 4x
    "LD C,B"     ,"LD C,C"     ,"LD C,D"     ,"LD C,E"     ,"LD C,H"     ,"LD C,L"     ,"LD C,[HL]"  ,"LD C,A"     , 
    "LD D,B"     ,"LD D,C"     ,"LD D,D"     ,"LD D,E"     ,"LD D,H"     ,"LD D,L"     ,"LD D,[HL]"  ,"LD D,A"     , // 5x
    "LD E,B"     ,"LD E,C"     ,"LD E,D"     ,"LD E,E"     ,"LD E,H"     ,"LD E,L"     ,"LD E,[HL]"  ,"LD E,A"     ,
    "LD H,B"     ,"LD H,C"     ,"LD H,D"     ,"LD H,E"     ,"LD H,H"     ,"LD H,L"     ,"LD H,[HL]"  ,"LD H,A"     , // 6x
    "LD L,B"     ,"LD L,C"     ,"LD L,D"     ,"LD L,E"     ,"LD L,H"     ,"LD L,L"     ,"LD L,[HL]"  ,"LD L,A"     ,
    "LD [HL],B"  ,"LD [HL],C"  ,"LD [HL],D"  ,"LD [HL],E"  ,"LD [HL],H"  ,"LD [HL],L"  ,"HALT"       ,"LD [HL],A"  , // 7x
    "LD A,B"     ,"LD A,C"     ,"LD A,D"     ,"LD A,E"     ,"LD A,H"     ,"LD A,L"     ,"LD A,[HL]"  ,"LD A,A"     ,
    "ADD A,B"    ,"ADD A,C"    ,"ADD A,D"    ,"ADD A,E"    ,"ADD A,H"    ,"ADD A,L"    ,"ADD A,[HL]" ,"ADD A,A"    , // 8x
    "ADC A,B"    ,"ADC A,C"    ,"ADC A,D"    ,"ADC A,E"    ,"ADC A,H"    ,"ADC A,L"    ,"ADC A,[HL]" ,"ADC A,A"    ,
    "SUB A,B"    ,"SUB A,C"    ,"SUB A,D"    ,"SUB A,E"    ,"SUB A,H"    ,"SUB A,L"    ,"SUB A,[HL]" ,"SUB A,A"    , // 9x
    "SBC A,B"    ,"SBC A,C"    ,"SBC A,D"    ,"SBC A,E"    ,"SBC A,H"    ,"SBC A,L"    ,"SBC A,[HL]" ,"SBC A,A"    , 
    "AND A,B"    ,"AND A,C"    ,"AND A,D"    ,"AND A,E"    ,"AND A,H"    ,"AND A,L"    ,"AND A,[HL]" ,"AND A,A"    , // Ax
    "XOR A,B"    ,"XOR A,C"    ,"XOR A,D"    ,"XOR A,E"    ,"XOR A,H"    ,"XOR A,L"    ,"XOR A,[HL]" ,"XOR A,A"    ,
    "OR A,B"     ,"OR A,C"     ,"OR A,D"     ,"OR A,E"     ,"OR A,H"     ,"OR A,L"     ,"OR A,[HL]"  ,"OR A,A"     , // Bx
    "CP A,B"     ,"CP A,C"     ,"CP A,D"     ,"CP A,E"     ,"CP A,H"     ,"CP A,L"     ,"CP A,[HL]"  ,"CP A,A"     ,
    "RET NZ"     ,"POP BC"     ,"JP NZ u16"  ,"JP u16"     ,"CALL NZ u16","PUSH BC"    ,"ADD A,u8"   ,"RST 0x00"   , // Cx
    "RET Z"      ,"RET"        ,"JP Z u16"   ,"PREFIX"     ,"CALL Z u16" ,"CALL u16"   ,"ADC A,u8"   ,"RST 0x08"   ,
    "RET NC"     ,"POP DE"     ,"JP NC u16"  ,"ILLEGAL"    ,"CALL NC u16","PUSH DE"    ,"SUB A,u8"   ,"RST 0x10"   , // Dx
    "RET C"      ,"RETI"       ,"JP C u16"   ,"ILLEGAL"    ,"CALL C u16" ,"ILLEGAL"    ,"SBC A,u8"   ,"RST 0x18"   ,
    "LDH [u8],A" ,"POP HL"     ,"LDH [C],A"  ,"ILLEGAL"    ,"ILLEGAL"    ,"PUSH HL"    ,"AND A,u8"   ,"RST 0x20"   , // Ex
    "ADD SP,i8"  ,"JP HL"      ,"LD [u16],A" ,"ILLEGAL"    ,"ILLEGAL"    ,"ILLEGAL"    ,"XOR A,u8"   ,"RST 0x28"   ,
    "LDH A,[u8]" ,"POP AF"     ,"LDH A,[C]"  ,"DI"         ,"ILLEGAL"    ,"PUSH AF"    ,"OR A,u8"    ,"RST 0x30"   , // Fx
    "LD HL,SP i8","LD SP,HL"   ,"LD A,[u16]" ,"EI"         ,"ILLEGAL"    ,"ILLEGAL"    ,"CP A,u8"    ,"RST 0x38"   ,      
];

pub const CB_PREFIX: &[&'static str; 256] = &[
//  x0/x8       x1/x9       x2/xA       x3/xB       x4/xC       x5/xD       x6/xE       x7/xF
    "RLC B"     ,"RLC C"     ,"RLC D"     ,"RLC E"     ,"RLC H"     ,"RLC L"     ,"RLC [HL]"  ,"RLC A"     , // 0x
    "RRC B"     ,"RRC C"     ,"RRC D"     ,"RRC E"     ,"RRC H"     ,"RRC L"     ,"RRC [HL]"  ,"RRC A"     ,   
    "RL B"      ,"RL C"      ,"RL D"      ,"RL E"      ,"RL H"      ,"RL L"      ,"RL [HL]"   ,"RL A"      , // 1x
    "RR B"      ,"RR C"      ,"RR D"      ,"RR E"      ,"RR H"      ,"RR L"      ,"RR [HL]"   ,"RR A"      , 
    "SLA B"     ,"SLA C"     ,"SLA D"     ,"SLA E"     ,"SLA H"     ,"SLA L"     ,"SLA [HL]"  ,"SLA A"     , // 2x
    "SRA B"     ,"SRA C"     ,"SRA D"     ,"SRA E"     ,"SRA H"     ,"SRA L"     ,"SRA [HL]"  ,"SRA A"     , 
    "SWAP B"    ,"SWAP C"    ,"SWAP D"    ,"SWAP E"    ,"SWAP H"    ,"SWAP L"    ,"SWAP [HL]" ,"SWAP A"    , // 3x
    "SRL B"     ,"SRL C"     ,"SRL D"     ,"SRL E"     ,"SRL H"     ,"SRL L"     ,"SRL [HL]"  ,"SRL A"     , 
    "BIT 0,B"   ,"BIT 0,C"   ,"BIT 0,D"   ,"BIT 0,E"   ,"BIT 0,H"   ,"BIT 0,L"   ,"BIT 0,[HL]","BIT 0,A"   , // 4x
    "BIT 1,B"   ,"BIT 1,C"   ,"BIT 1,D"   ,"BIT 1,E"   ,"BIT 1,H"   ,"BIT 1,L"   ,"BIT 1,[HL]","BIT 1,A"   , 
    "BIT 2,B"   ,"BIT 2,C"   ,"BIT 2,D"   ,"BIT 2,E"   ,"BIT 2,H"   ,"BIT 2,L"   ,"BIT 2,[HL]","BIT 2,A"   , // 5x
    "BIT 3,B"   ,"BIT 3,C"   ,"BIT 3,D"   ,"BIT 3,E"   ,"BIT 3,H"   ,"BIT 3,L"   ,"BIT 3,[HL]","BIT 3,A"   , 
    "BIT 4,B"   ,"BIT 4,C"   ,"BIT 4,D"   ,"BIT 4,E"   ,"BIT 4,H"   ,"BIT 4,L"   ,"BIT 4,[HL]","BIT 4,A"   , // 6x
    "BIT 5,B"   ,"BIT 5,C"   ,"BIT 5,D"   ,"BIT 5,E"   ,"BIT 5,H"   ,"BIT 5,L"   ,"BIT 5,[HL]","BIT 5,A"   , 
    "BIT 6,B"   ,"BIT 6,C"   ,"BIT 6,D"   ,"BIT 6,E"   ,"BIT 6,H"   ,"BIT 6,L"   ,"BIT 6,[HL]","BIT 6,A"   , // 7x
    "BIT 7,B"   ,"BIT 7,C"   ,"BIT 7,D"   ,"BIT 7,E"   ,"BIT 7,H"   ,"BIT 7,L"   ,"BIT 7,[HL]","BIT 7,A"   , 
    "RES 0,B"   ,"RES 0,C"   ,"RES 0,D"   ,"RES 0,E"   ,"RES 0,H"   ,"RES 0,L"   ,"RES 0,[HL]","RES 0,A"   , // 8x 
    "RES 1,B"   ,"RES 1,C"   ,"RES 1,D"   ,"RES 1,E"   ,"RES 1,H"   ,"RES 1,L"   ,"RES 1,[HL]","RES 1,A"   , 
    "RES 2,B"   ,"RES 2,C"   ,"RES 2,D"   ,"RES 2,E"   ,"RES 2,H"   ,"RES 2,L"   ,"RES 2,[HL]","RES 2,A"   , // 9x
    "RES 3,B"   ,"RES 3,C"   ,"RES 3,D"   ,"RES 3,E"   ,"RES 3,H"   ,"RES 3,L"   ,"RES 3,[HL]","RES 3,A"   , 
    "RES 4,B"   ,"RES 4,C"   ,"RES 4,D"   ,"RES 4,E"   ,"RES 4,H"   ,"RES 4,L"   ,"RES 4,[HL]","RES 4,A"   , // Ax
    "RES 5,B"   ,"RES 5,C"   ,"RES 5,D"   ,"RES 5,E"   ,"RES 5,H"   ,"RES 5,L"   ,"RES 5,[HL]","RES 5,A"   , 
    "RES 6,B"   ,"RES 6,C"   ,"RES 6,D"   ,"RES 6,E"   ,"RES 6,H"   ,"RES 6,L"   ,"RES 6,[HL]","RES 6,A"   , // Bx
    "RES 7,B"   ,"RES 7,C"   ,"RES 7,D"   ,"RES 7,E"   ,"RES 7,H"   ,"RES 7,L"   ,"RES 7,[HL]","RES 7,A"   , 
    "SET 0,B"   ,"SET 0,C"   ,"SET 0,D"   ,"SET 0,E"   ,"SET 0,H"   ,"SET 0,L"   ,"SET 0,[HL]","SET 0,A"   , // Cx
    "SET 1,B"   ,"SET 1,C"   ,"SET 1,D"   ,"SET 1,E"   ,"SET 1,H"   ,"SET 1,L"   ,"SET 1,[HL]","SET 1,A"   , 
    "SET 2,B"   ,"SET 2,C"   ,"SET 2,D"   ,"SET 2,E"   ,"SET 2,H"   ,"SET 2,L"   ,"SET 2,[HL]","SET 2,A"   , // Dx
    "SET 3,B"   ,"SET 3,C"   ,"SET 3,D"   ,"SET 3,E"   ,"SET 3,H"   ,"SET 3,L"   ,"SET 3,[HL]","SET 3,A"   , 
    "SET 4,B"   ,"SET 4,C"   ,"SET 4,D"   ,"SET 4,E"   ,"SET 4,H"   ,"SET 4,L"   ,"SET 4,[HL]","SET 4,A"   , // Ex
    "SET 5,B"   ,"SET 5,C"   ,"SET 5,D"   ,"SET 5,E"   ,"SET 5,H"   ,"SET 5,L"   ,"SET 5,[HL]","SET 5,A"   , 
    "SET 6,B"   ,"SET 6,C"   ,"SET 6,D"   ,"SET 6,E"   ,"SET 6,H"   ,"SET 6,L"   ,"SET 6,[HL]","SET 6,A"   , // Fx
    "SET 7,B"   ,"SET 7,C"   ,"SET 7,D"   ,"SET 7,E"   ,"SET 7,H"   ,"SET 7,L"   ,"SET 7,[HL]","SET 7,A"   , 
];
//LD A,[u16]
//0123456789012
//CALL NC,FFFFh
//LD SP,[FFFFh]
//ILLEGAL
//LD HL,SP+r8
