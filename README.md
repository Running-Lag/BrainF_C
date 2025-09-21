# BrainF Compiler
This compiles BrainF programms to machine code
## Current Status
### Done
* \><br>
* <<br>
* +<br>
* -<br>
* [<br>
* ]<br>
* .<br>

### TODO
* ,<br>

## Usage
1. Compile the BrainF code<br>
`cargo run --release demo.bf demo.o`
2. Compile the stdlib <br>
`nasm bf_stdlib_linux.asm -f elf64 -o bf_stdlib_linux.o`
3. Link both generated object files <br>
`ld -o demo demo.o bf_stdlib_linux.o -e main`
4. Run it <br>
`./demo`
