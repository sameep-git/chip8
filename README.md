# Chip8 emulator
This is a Chip8 emulator build on Rust.
Currently working on developing the Emulator interface and writing code for the operations that help run code written for Chip8, likes games, etc.

Opcodes:
Each opcode is 16bits. References to registers are usually refered to by `X` and `Y`. Immediate values as `N`, `NN`, and `NNN`. `NNN` used for 12bit addresses. Registers include 8bit `V0` through `VF`, 12bit memory location `I`, 12bit program counter `PC`, 8bit delay timer `DT`, 8bit sound timer `ST`. There is also a 12bit stack pointer `SP`.

| opcode |mnemonic| description|
|--|--|--|
| 0000 | NOP | Do nothing |
| 00E0 | CLS | Clear screen |
| 00EE | RET | Return from subroutine |
| 1NNN | JMP NNN | Jump to given address NNN |
| 2NNN | CALL NNN | Call subroutine at address NNN, adding current PC onto stack so we can return here |
| 3XNN | SKIP VX == NN | skip line if VX == NN |
| 4XNN | SKIP VX != NN | skip line if VX != NN |
| 5XY0 | SKIP VX == VY | skip line if VX == VY |
| 6XNN | VX = NN | sets VX = NN |
| 7XNN | VX += NN | increments register VX by NN |
| 8XY0 | VX = VY | sets register VX = VY |
| 8XY1 | VX \|= VY | ORs VX with VY and stores the result in VX
| 8XY2 | VX &= VY | ANDs VX with VY and stores the result in VX
| 8XY3 | VX ^= VY | XORs VX with VY and stores the result in VX
| 8XY4 | VX += VY | increments VX with VY; sets VF if carry
| 8XY5 | VX -= VY | decrements VX with VY; sets VF to 0 if underflow and 1 if not
| 8XY6 | VX >>= VY | Right shift; stores dropped bit in VF
| 8XY7 | VX = VY - VX | sets VF to 0 if underflow and 1 if not


TODO:

 - [ ] User Interface
	 - [ ] User input
	 - [ ] Loading files
	 - [ ] Drawing to screen
 - [ ] WebAssembly integration
 
 

