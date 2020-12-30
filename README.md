# Rust VM
A Virtual Machine written in Rust to emulate the LC3 computer (Little Computer 3) and a re-implementation of the C code versioni of the same.

## Usage
1. Running it is simple, download the `rust_vm` binary from this repo along with ideally any LC3 assembly code (or you can just download the `2048.obj` or `rogue.obj` from this repo).
2. Run `./rust_vm /path/to/lc3_assembly`. Example: `./rust_vm rogue.obj`.
3. NOTE: This VM code has been written specifically to run in Unix like Operating Systems. The binary may or may not run in Windows machines.

## Preview
### Rogue game

### 2048 game


## More About the VM
1. Here is a simple diagram I made representing the workflow of the VM:

2. The memory and register are the main hardware that are emulated. Rest of our code is mainly on reading the instructions from memory, determining what type of instruction it is (Op-code), and performing the corresponding action. To understand the different op-codes/type of instructions that our VM should be able to do, refer this [pdf](https://courses.engr.illinois.edu/ece411/fa2019/mp/LC3b_ISA.pdf).

## Notes
1. The LC3 assembly codes are stored in Big-Endian byte order while my PC has an x86-64 architecturestoring words in Little-Endian Formats. That's why, in the code, while reading program into our emulated memory,certain swapping was done to store the bytes in Little Endian order.
2. Rust doesn't directly provide wrapping of integer overflow which is normal in C code. The LC3 assembly code also uses the wrapping of integer overflow extensively while adding addresses with offsets (see code in `src\opcode_fn.rs`). I had to use Rust's `wrapping_add` function for this case.
3. Default behaviour of standard consoles are to get input from a user and process them only when a newline character was entered (hitting the enter button). In order to play the games, the default behaviour for the terminal needed to be changed. I referred this [answer](https://stackoverflow.com/a/37416107/11105624) in stackoverflow and used an external crate `termios` to solve this. This is also the part where certain platform specific codes were written.
4. I wrote Index trait implementations for certain enums in order to use them as indexes for vectors directly.
