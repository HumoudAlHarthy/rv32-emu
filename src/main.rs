//! A tiny RV32I emulator

struct Cpu {
    regs: [u32; 32],
    pc: u32,
    mem: Vec<u8>,
}

impl Cpu {
    fn new(mem_size: usize) -> Self {
        Cpu { regs: [0; 32], pc: 0, mem: vec![0; mem_size] }
    }

    fn load_program(&mut self, words: &[u32]) {
        for (i, w) in words.iter().enumerate() {
            self.mem[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
    }

    fn read32(&self, addr: u32) -> u32 {
        let a = addr as usize;
        u32::from_le_bytes(self.mem[a..a + 4].try_into().unwrap())
    }

    fn write32(&mut self, addr: u32, val: u32) {
        let a = addr as usize;
        self.mem[a..a + 4].copy_from_slice(&val.to_le_bytes());
    }

    /// x0 is hardwired to zero, so writes to it are ignored.
    fn set_reg(&mut self, rd: usize, val: u32) {
        if rd != 0 {
            self.regs[rd] = val;
        }
    }

    /// Execute one instruction. Returns false when the program halts (ECALL).
    fn step(&mut self) -> bool {
        let inst = self.read32(self.pc);
        let opcode = inst & 0x7f;
        let rd = ((inst >> 7) & 0x1f) as usize;
        let funct3 = (inst >> 12) & 0x7;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let funct7 = inst >> 25;
        let a = self.regs[rs1];
        let b = self.regs[rs2];

        // Sign-extended immediates for each instruction format.
        let imm_i = (inst as i32) >> 20;
        let imm_s = (((inst as i32) >> 25) << 5) | ((inst >> 7) & 0x1f) as i32;
        let imm_b = (((inst as i32) >> 31) << 12)
            | ((((inst >> 7) & 1) << 11) as i32)
            | ((((inst >> 25) & 0x3f) << 5) as i32)
            | ((((inst >> 8) & 0xf) << 1) as i32);
        let imm_u = inst & 0xffff_f000;
        let imm_j = (((inst as i32) >> 31) << 20)
            | ((((inst >> 12) & 0xff) << 12) as i32)
            | ((((inst >> 20) & 1) << 11) as i32)
            | ((((inst >> 21) & 0x3ff) << 1) as i32);

        let mut next_pc = self.pc.wrapping_add(4);

        match opcode {
            0x37 => self.set_reg(rd, imm_u), // LUI
            0x6f => {
                // JAL
                self.set_reg(rd, next_pc);
                next_pc = self.pc.wrapping_add(imm_j as u32);
            }
            0x63 => {
                // Branches: BEQ / BNE
                let taken = match funct3 {
                    0 => a == b,
                    1 => a != b,
                    _ => panic!("unsupported branch funct3 {funct3}"),
                };
                if taken {
                    next_pc = self.pc.wrapping_add(imm_b as u32);
                }
            }
            0x03 => {
                // LW
                let v = self.read32(a.wrapping_add(imm_i as u32));
                self.set_reg(rd, v);
            }
            0x23 => {
                // SW
                self.write32(a.wrapping_add(imm_s as u32), b);
            }
            0x13 => match funct3 {
                0 => self.set_reg(rd, a.wrapping_add(imm_i as u32)), // ADDI
                _ => panic!("unsupported OP-IMM funct3 {funct3}"),
            },
            0x33 => match (funct3, funct7) {
                (0, 0x00) => self.set_reg(rd, a.wrapping_add(b)), // ADD
                (0, 0x20) => self.set_reg(rd, a.wrapping_sub(b)), // SUB
                _ => panic!("unsupported OP funct3={funct3} funct7={funct7}"),
            },
            0x73 => return false, // ECALL: treat as halt
            _ => panic!("unsupported opcode {opcode:#x} at pc {:#x}", self.pc),
        }

        self.pc = next_pc;
        true
    }
}

// --- Assembler to write in Rust

fn i_type(imm: i32, rs1: u32, funct3: u32, rd: u32, opcode: u32) -> u32 {
    (((imm as u32) & 0xfff) << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
}

fn addi(rd: u32, rs1: u32, imm: i32) -> u32 {
    i_type(imm, rs1, 0, rd, 0x13)
}

fn add(rd: u32, rs1: u32, rs2: u32) -> u32 {
    (rs2 << 20) | (rs1 << 15) | (rd << 7) | 0x33
}

fn bne(rs1: u32, rs2: u32, offset: i32) -> u32 {
    let imm = offset as u32;
    (((imm >> 12) & 1) << 31)
        | (((imm >> 5) & 0x3f) << 25)
        | (rs2 << 20)
        | (rs1 << 15)
        | (1 << 12)
        | (((imm >> 1) & 0xf) << 8)
        | (((imm >> 11) & 1) << 7)
        | 0x63
}

fn ecall() -> u32 {
    0x73
}

fn main() {
    // Sum the numbers 10 down to 1 into x10. Expected result: 55.
    let program = [
        addi(11, 0, 10),  // x11 = 10
        add(10, 10, 11),  // loop: x10 += x11
        addi(11, 11, -1), // x11 -= 1
        bne(11, 0, -8),   // if x11 != 0, jump back to loop
        ecall(),          // halt
    ];

    let mut cpu = Cpu::new(64 * 1024);
    cpu.load_program(&program);

    let mut steps = 0;
    while cpu.step() {
        steps += 1;
    }

    println!("halted after {steps} instructions");
    println!("x10 = {}", cpu.regs[10]);
    assert_eq!(cpu.regs[10], 55);
}
