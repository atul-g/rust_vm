use std::fs::File;
use std::path::Path;
use std::io::Read;


pub mod register {
    use std::ops::{Index, IndexMut};

    //This enum is just to index each of our registers
    //We will be storing the register in an array of type
    //u16. Each register storing 16 bits only.
    //R0 to R8 are general purpose registers.
    //PC is program counter register's index. R_COND is
    //the index for register which stores information of
    //previous calculations.
    //COUNT has the index 10, which indicates the count of
    //total registers in our architecture.
    pub enum Reg {
        R0,
        R1,
        R2,
        R3,
        R4,
        R5,
        R6,
        R7,
        PC,
        COND,
        COUNT,
    }

    //In order to use enum for indexing, we will need to cast the value of
    //enum variant to 'usize' everytime. i.e, myvec[myenum::val as usize];
    //To avoid writing 'as usize' everytime, we implement the Index and IndexMut
    //traits for the vectors to use our enums as indexes.
    impl<T> Index<Reg> for Vec<T>
    {
        type Output = T;
        fn index(&self, reg: Reg) -> &T {
            &self[reg as usize]
        }
    }

    impl<T> IndexMut<Reg> for Vec<T>
    {
        fn index_mut(&mut self, reg: Reg) -> &mut T {
            &mut self[reg as usize]
        }
    }

}

pub mod opcodes {
    use std::ops::{Index, IndexMut};

    //Instruction Set.
    //There are 16 opcodes in LC3. Each instruction in LC3 is 16 bits long. Left
    //4 bits store the opcode. Rest of the bits store the parameters on which
    //the opcode should work.
    #[allow(non_camel_case_types)]
    pub enum OpCodes {
        OP_BR = 0, // branch
        OP_ADD,    // add
        OP_LD,     // load
        OP_ST,     // store
        OP_JSR,    // jump register
        OP_AND,    // bitwise and
        OP_LDR,    // load register
        OP_STR,    // store register
        OP_RTI,    // unused
        OP_NOT,    // bitwise not
        OP_LDI,    // load indirect
        OP_STI,    // store indirect
        OP_JMP,    // jump
        OP_RES,    // reserved (unused)
        OP_LEA,    // load effective address
        OP_TRAP    // execute trap
    }

    impl<T> Index<OpCodes> for Vec<T>
    {
        type Output = T;
        fn index(&self, opcode: OpCodes) -> &T {
            &self[opcode as usize]
        }
    }

    impl<T> IndexMut<OpCodes> for Vec<T>
    {
        fn index_mut(&mut self, opcode: OpCodes) -> &mut T {
            &mut self[opcode as usize]
        }
    }
}


pub enum TrapCode {
    GETC = 0x20,  // 32 - get character from keyboard, not echoed onto the terminal
    OUT = 0x21,   // 33 - output a character
    PUTS = 0x22,  // 34 - output a word string
    IN = 0x23,    // 35 - get character from keyboard, echoed onto the terminal
    PUTSP = 0x24, // 36 - output a byte string
    HALT = 0x25   // 37 - halt the program
}


//Condition Flag in register stores information about last calculation
//execution. LC3 only had 3 condition flags in this register which stores
//the sign of the previous calculation.
#[allow(non_camel_case_types)]
pub enum CondFlags {
    FL_POS = 1<<0, //Positive
    FL_ZRO = 1<<1, //Zero
    FL_NEG = 1<<2, //Negative
}


//Memory Mapped Registers
//These special register has address reserved for them in memory. So
//to read and write to this register, we read/write into the memory.
//KBSR identifies whether a key was pressed. KBDR tells us what key was
//pressed
#[allow(non_camel_case_types)]
pub enum MemMapReg {
    MR_KBSR = 0xFE00, //Keyboard Status Register. 0xFE00 = 65024.
    MR_KBDR = 0xFE02, //Keyboard Data Register. 0xFE02 = 65026.
}

use register::Reg;

pub fn sign_extend(mut x: u16, bit_count: u16) -> u16 {
    //this checks if the last bit has a 1 (indicating negative number)
    if (x >> (bit_count-1)) & 1 == 1 {
        //we extend the left side with 1's as it is a -ve number
        x |= (0xFFFF << bit_count);
    }
    x
}


pub fn update_flags(r: usize, reg: &mut Vec<u16>) {
    let val: u16 = reg[r];

    if val == 0 {
        reg[Reg::COND] = CondFlags::FL_ZRO as u16;
    }

    else if val >> 15 == 1 { //1 in leftmost bit means negative
        reg[Reg::COND] = CondFlags::FL_NEG as u16;
    }

    else {
        reg[Reg::COND] = CondFlags::FL_POS as u16;
    }

}


pub fn read_image(image: &str, memory: &mut Vec<u16>) -> bool {
    //println!("read {}", image);

    let path = Path::new(image);
    let mut file = File::open(path).expect("No such file exists.");

    //data is a Vec<u8> where the data is read and stored in the
    //form of bytes using `read_to_end()` method.
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("buffer offerflow");

    //chunks(2) method combines the elements as [[val0, val1], ...]
    //chunks returns an iterator over data vector.
    let mut iter = data.chunks(2);

    //The first element specifies the address in memory where program
    //should start. It is the general value 0x3000 or 12288 in rogue.obj
    let pc = iter.next().unwrap();

    //We are combining two bytes into a u16 word as that is how our memory
    //stores data. That is, word size of our memory is 16 bits.
    let mut pc: usize = ((pc[0] as u16) << 8 | pc[1] as u16) as usize;

    //We now store the rest of the program data into memory
    for elem in iter {
        memory[pc] = (elem[0] as u16) << 8 | elem[1] as u16;
        pc = pc+1;
    }

    true
}


pub fn mem_read(addr: u16, memory: &mut Vec<u16>) -> u16 {
    //let instr: u16 = 0b1111_0000_00100100;
    if addr == MemMapReg::MR_KBSR as u16 {
        let mut buffer = [0; 1];
        std::io::stdin().read_exact(&mut buffer).unwrap();

        if buffer[0] != 0 {
            memory[MemMapReg::MR_KBSR as usize] = 1 << 15;
            memory[MemMapReg::MR_KBDR as usize] = buffer[0] as u16;
        }
        else {
            memory[MemMapReg::MR_KBSR as usize] = 0;
        }
    }

    memory[addr as usize]
}

pub fn mem_write(addr: u16, val: u16, memory: &mut Vec<u16>) {
    memory[addr as usize] = val;
}
