//For reference: https://justinmeiners.github.io/lc3-vm/supplies/lc3-isa.pdf

use rust_vm::register::Reg;
use rust_vm::{sign_extend, update_flags, mem_read, mem_write};


//NOTE: The assembly codes that will be passed to our emulator
//relies heavily on integer overflow additions to wrap around.
//Rust doesn't allow this in normal additions, eg: let a: u16 = 65535 + 1
//will yield an error. For this we used the u16::wrapping_add() function.
//u16::wrapping_add(65536, 1) is same as 65535 + 1 yielding 0 in this case.

//Add
pub fn op_add(reg: &mut Vec<u16>, instr: u16) {
    //NOTE: into() is used to convert u16 to usize here:
    let r0: usize = ((instr >> 9) & 0x07).into(); //getting destination register
    let r1: usize = ((instr >> 6) & 0x07).into(); //getting first operand register
    let imm_flag: u16 = (instr >> 5) & 0x01; //immediate mode?

    if imm_flag == 1 {
        //extract last 5 bits of instr, which is the imm number
        //and also extend it
        let imm5: u16 = sign_extend(instr & 0x1f, 5);
        reg[r0] = u16::wrapping_add(reg[r1], imm5);
    }
    else {
        let r2: usize = (instr & 0x07).into(); //last 3 bits of instruction is second operand
        reg[r0] = u16::wrapping_add(reg[r1], reg[r2]);
    }

    update_flags(r0, reg);
}

//Load Indirect - Load a value from a location in memory into register
pub fn op_ldi(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16> ) {
    let r0: usize = ((instr >> 9) & 0x07).into();
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);

    //Here we first add PC to pc_offset. We use mem_read() on this value.
    //mem_read() returns an address which contains the actual value we
    //want to load. So we use mem_read() again on this adress, get the value
    //and store it in destination register.
    reg[r0] = mem_read(mem_read(u16::wrapping_add(reg[Reg::PC], pc_offset), memory), memory);
    update_flags(r0, reg);
}

//Bitwise And
pub fn op_and(reg: &mut Vec<u16>, instr: u16) {
    let r0: usize = ((instr >> 9) & 0x07).into();
    let r1: usize = ((instr >> 6) & 0x07).into();
    let imm_flag: u16 = (instr >> 5) & 0x01;

    if imm_flag == 1 {
        let imm5: u16 = sign_extend(instr & 0x1f, 5);
        reg[r0] = reg[r1] & imm5;
    }
    else {
        let r2: usize = (instr & 0x7).into();
        reg[r0] = reg[r1] & reg[r2];
    }

    update_flags(r0, reg);
}

//Bitwise Not
pub fn op_not(reg: &mut Vec<u16>, instr: u16) {
    let r0: usize = ((instr >> 9) & 0x07).into();
    let r1: usize = ((instr >> 6) & 0x07).into();

    reg[r0] = !reg[r1];
    update_flags(r0, reg);
}

//Branch
pub fn op_branch(reg: &mut Vec<u16>, instr: u16) {
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);
    let cond_flag: u16 = (instr >> 9) & 0x7;
    //reg[Reg::COND] can be either 1, 2, 4 denoting
    //Zero, Negative, Positive.
    //If cond_flag matches the condition set in reg[Reg::COND]
    //then we add pc_offset to current PC and branch of to 
    //that instructions.
    if (cond_flag & reg[Reg::COND]) > 0 {
        reg[Reg::PC] = u16::wrapping_add(reg[Reg::PC], pc_offset);
    }
}


//Note: RET is actually just a special case of JUMP
pub fn op_jump(reg: &mut Vec<u16>, instr: u16) {
    let r1: usize = ((instr >> 6) & 0x07).into();
    reg[Reg::PC] = reg[r1];
}


//Jump Register
pub fn op_jsr(reg: &mut Vec<u16>, instr: u16) {
    let long_flag: u16 = (instr >> 11) & 1;
    //We save the incremented PC to Register 7 as this
    //helps in allowing us to go back to the sub-routine
    //that initially called and resume the work
    reg[Reg::R7] = reg[Reg::PC];

    if long_flag == 1 {
        let long_pc_offset = sign_extend(instr & 0x7FF, 11);
        reg[Reg::PC] = u16::wrapping_add(reg[Reg::PC], long_pc_offset);
    }
    else {
        let r1: usize = ((instr >> 6) & 0x07).into();
        reg[Reg::PC] = reg[r1];
    }
}


//"Load - An address is computed by sign-extending bits [8:0]
//to 16 bits and adding this value to the incremented PC. The
//contents of memory at this address are loaded into DR. The
//condition codes are set, based on whether the value loaded
//is negative, zero, or positive."
pub fn op_load(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x7).into();
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);

    reg[r0] = mem_read(u16::wrapping_add(reg[Reg::PC], pc_offset), memory);
    update_flags(r0, reg);
}


//"Load Register - An address is computed by sign-extending bits
//[5:0] to 16 bits and adding this value to the contents of the
//register specified by bits [8:6]. The contents of memory at
//this address are loaded into DR.
pub fn op_ldr(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x7).into();
    let r1: usize = ((instr >> 6) & 0x7).into();
    let offset: u16 = sign_extend(instr & 0x3F, 6);

    reg[r0] = mem_read(u16::wrapping_add(reg[r1], offset), memory);
    update_flags(r0, reg);
}


//"Load Effective Address - An address is computed by sign-extending
//bits [8:0] to 16 bits and adding this value to the incremented PC.
//This address is loaded into DR."
pub fn op_lea(reg: &mut Vec<u16>, instr: u16) {
    let r0: usize = ((instr >> 9) & 0x7).into();
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);

    reg[r0] = u16::wrapping_add(reg[Reg::PC], pc_offset); //lea differs from load in this line
    update_flags(r0, reg);
}


//"Store - The contents of the register specified by SR are stored
//in the memory location whose address is computed by sign-extending
//bits [8:0] to 16 bits and adding this value to the incremented PC."
pub fn op_st(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x7).into();
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);
    mem_write(u16::wrapping_add(reg[Reg::PC], pc_offset), reg[r0], memory); 
}


//"Store Indirect Address - The contents of the register specified
//by SR are stored in the memory location whose address is obtained as
//follows: Bits [8:0] are sign-extended to 16 bits and added to the
//incremented PC. What is in memory at this address is the address of
//the location to which the data in SR is stored."
pub fn op_sti(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x7).into();
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);
    mem_write(mem_read(u16::wrapping_add(reg[Reg::PC], pc_offset), memory), reg[r0], memory);
}


//"Store Register - The contents of the register specified by SR
//are stored in the memory location whose address is computed by
//sign-extending bits [5:0] to 16 bits and adding this value to
//the contents of the register specified by bits [8:6]."
pub fn op_str(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x7).into();
    let r1: usize = ((instr >> 6) & 0x7).into();

    let offset: u16 = sign_extend(instr & 0x3F, 6);

    mem_write(u16::wrapping_add(reg[r1], offset), reg[r0], memory);
}
