extern crate termios;

use std::env;
use std::process;
use termios::*;

use rust_vm::{TrapCode, read_image, mem_read};
use rust_vm::register::Reg;
use rust_vm::opcodes::OpCodes;

mod opcode_fn;
use opcode_fn::*;

mod trapcode_fn;
use trapcode_fn::*;


fn main() {
    //Collect CLI arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2  {
        println!("Error: provide atleast one VM image");
        println!("Usage: rust-vm <image-file1> [image-file2]..");
        process::exit(2);
    }

    //memory of the computer.
    //LC3 has 65536 memory locations, each storing 16 bits.
    //So in total, it has a memory of 128KBs
    let mut memory = vec![0u16; 65536]; //0u16 stands for 0 of type u16

    for i in 1..args.len() {
        if !read_image(&args[i], &mut memory)  {
            println!("Failed to load image: {}", args[i]);
            process::exit(1);
        }
    }


    //Platform Specifics (Unix here)
    //Setting terminal input/output behaviour such as accepting
    //character without the need for a newline character
    //Refer: https://stackoverflow.com/questions/26321592/how-can-i-read-one-character-from-stdin-without-having-to-hit-enter
    let stdin = 0; 

    let termios = Termios::from_fd(stdin).unwrap();

    let mut new_termios = termios.clone(); // make a mutable copy of termios
    // that we will modify
    new_termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
    new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();

    //Platform specific end

    let args: Vec<String> = env::args().collect();


    #[allow(non_snake_case)]
    let PC_START: u16 = 0x3000; //default starting address for PC

    //LC3 register
    let mut registers: Vec<u16> = vec![0; Reg::COUNT as usize];

    registers[Reg::PC] = PC_START; //Default starting address

    let mut running: bool = true;

    while running {
        let instr: u16 = mem_read(registers[Reg::PC], &mut memory);

        registers[Reg::PC] = registers[Reg::PC] + 1; //increment PC

        let op: u16 = instr >> 12; //opcode is in left 4 bits.
        //println!("Executing Instr {:#018b} and Opcode bit: {}", instr, op);

        match op {
            op if op == OpCodes::OP_BR as u16 => {
                //println!("Executing BRANCH, Instr {:#018b}", instr);
                op_branch(&mut registers, instr);
            },

            op if op == OpCodes::OP_ADD as u16 => {
                //println!("Executing ADD, Instr {:#018b}", instr);
                op_add(&mut registers, instr);
            },

            op if op == OpCodes::OP_LD as u16 => {
                //println!("Executing LD , Instr {:#018b}", instr);
                op_load(&mut registers, instr, &mut memory);
            },

            op if op == OpCodes::OP_ST as u16 => {
                //println!("Executing ST , Instr {:#018b}", instr);
                op_st(&mut registers, instr, &mut memory);
            },

            op if op == OpCodes::OP_JSR as u16 => {
                //println!("Executing JSR, Instr {:#018b}", instr);
                op_jsr(&mut registers, instr);
            },

            op if op == OpCodes::OP_AND as u16 => {
                //println!("Executing AND, Instr {:#018b}", instr);
                op_and(&mut registers, instr);
            },

            op if op == OpCodes::OP_LDR as u16 => {
                //println!("Executing LDR, Instr {:#018b}", instr);
                op_ldr(&mut registers, instr, &mut memory);
            }, 

            op if op == OpCodes::OP_STR as u16 => {
                //println!("Executing STR, Instr {:#018b}", instr);
                op_str(&mut registers, instr, &mut memory);
            },

            op if op == OpCodes::OP_RTI as u16 => {
                println!("Bad OpCode 'RTI' received. Aborting.");
                process::exit(10);
            },

            op if op == OpCodes::OP_NOT as u16 => {
                //println!("Executing NOT, Instr {:#018b}", instr);
                op_not(&mut registers, instr);
            },

            op if op == OpCodes::OP_LDI as u16 => {
                //println!("Executing LDI, Instr {:#018b}", instr);
                op_ldi(&mut registers, instr, &mut memory);
            },

            op if op == OpCodes::OP_STI as u16 => {
                //println!("Executing STI, Instr {:#018b}", instr);
                op_sti(&mut registers, instr, &mut memory);
            },

            op if op == OpCodes::OP_JMP as u16 => {
                //println!("Executing JMP, Instr {:#018b}", instr);
                op_jump(&mut registers, instr);
            },

            op if op == OpCodes::OP_RES as u16 => {
                println!("Bad OpCode 'RES' received. Aborting.");
                process::exit(10);
            },

            op if op == OpCodes::OP_LEA as u16 => {
                //println!("Executing ADD, Instr {:#018b}", instr);
                op_lea(&mut registers, instr);
            },

            //first 4 bits = 1111, is for trap code
            op if op == OpCodes::OP_TRAP as u16 => {
                //0xFF = 255, trapcode is identified by the last 8
                //bits of the instruction
                let trap: u16 = instr & 0xFF;
                match trap {
                    trap if trap == TrapCode::GETC as u16 => {
                        //println!("Executing GETC TRAP, Instr {:#018b}", instr);
                        trap_getc(&mut registers);
                    },

                    trap if trap == TrapCode::OUT as u16 => {
                        //println!("Executing OUT TRAP, Instr {:#018b}", instr);
                        trap_out(&mut registers);
                    },

                    trap if trap == TrapCode::PUTS as u16 => {
                        //println!("Executing PUTS TRAP, Instr {:#018b}", instr);
                        trap_puts(&mut registers, &mut memory);
                    },

                    trap if trap == TrapCode::IN as u16 => {
                        //println!("Executing IN  TRAP, Instr {:#018b}", instr);
                        trap_in(&mut registers);
                    },

                    trap if trap == TrapCode::PUTSP as u16 => {
                        //println!("Executing PUTSP TRAP, Instr {:#018b}", instr);
                        trap_putsp(&mut registers, &mut memory);
                    },

                    trap if trap == TrapCode::HALT as u16 => { 
                        println!("HALT Trapcode received, Halting.");
                        running = false;
                    },

                    _ => {
                        println!("Invalid Trap Code received, aborting.");
                        process::exit(21);
                    }
                }
            },

            _ => {
                println!("Invalid Opcode recieved, aborting current image.");
                process::exit(20);
            }
        }

    }

    // reset the stdin to original termios data


    tcsetattr(stdin, TCSANOW, &termios).unwrap();
    println!("Shutting Down VM...");
}
