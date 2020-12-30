// Programs generally start at address 0x3000 only because
// the lower address are left empty for trap routine codes.

use std::io::Read;
use rust_vm::register::Reg;

// perhaps take a slice for memory?
pub fn trap_puts(reg: &mut Vec<u16>, memory: &mut Vec<u16>) {
    let mut index: usize = reg[Reg::R0] as usize;

    while index < memory.len() && memory[index] != 0 {
        //the `as` cast truncates the upper 8 bits while going
        //from u16 -> u8
        print!("{}", (memory[index] as u8) as char);
        index = index + 1;
    }
}


pub fn trap_getc(reg: &mut Vec<u16>) {
    //Credits to erfur for an easy way to input only a single
    //character from stdin: 
    //https://github.com/erfur/lc3-vm-rust/blob/61679739c7d498dc932e34d6c74c8ba0564b18aa/src/main.rs#L257
    let mut buffer = [0 as u8; 1];
    std::io::stdin().read_exact(&mut buffer).unwrap();
    reg[Reg::R0] = buffer[0].into();
}


pub fn trap_out(reg: &mut Vec<u16>) {
    print!("{}", (reg[Reg::R0] as u8) as char);
}


pub fn trap_in(reg: &mut Vec<u16>) {
    print!("Enter a character: ");
    //Alternative way to get a character input
    reg[Reg::R0] = std::io::stdin()
        .bytes()
        .next()
        .and_then(|result| result.ok())
        .map(|byte| byte as u16)
        .unwrap();
}


pub fn trap_putsp(reg: &mut Vec<u16>, memory: &mut Vec<u16>) {
    let mut index: usize = reg[Reg::R0] as usize;

    while index < memory.len() && memory[index] != 0 {
        //A word in our VM is 16 bits
        let word: u16 = memory[index];

        //We get the two bytes from our word. bytes here is an array of u8
        let bytes = word.to_be_bytes();
        
        print!("{}", bytes[1] as char);

        if bytes[0] != 0  {
            print!("{}", bytes[0] as char);
        }

        index = index + 1;
    }

}
