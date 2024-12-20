use rand::Rng;

pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH: usize = 64;

const RAM_SIZE: usize = 4096;   // RAM is 4KB for chip8
const NUM_REGS: usize = 16;     // 16 8-bit registers V0-VF
const STACK_SIZE: usize = 16;   // Stack Size in numbers
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;  // all programs are loaded into RAM starting at 0x200

const FONTSET_SIZE: usize = 80;

/// font set for 0-F char displayed on screen
/// they all have black right halfs
/// each requires 5 bytes of memory
/// to be stored in the first empty 512 bytes in mem.
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

/*
    Main struct to access information about the system.
    public so front-end can access the information
*/
pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    /// array storing 1 or 0 as chip8 only supports black or white color output
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    /// 8-bit (u8)
    v_reg: [u8; NUM_REGS],        
    /// 16-bit register used to index into RAM for reads and writes                  
    i_reg: u16,
    /// Stack Pointer: points to the top of the stack
    sp: u16,
    /// holds STACK_SIZE number of values
    stack: [u16; STACK_SIZE],
    /// can be pressed down by the user
    keys: [bool; NUM_KEYS],
    /// delay timer: counts down every clock cycle and performs an action if = 0
    dt: u8,
    /// sound timer: emits a sound if = 0
    st: u8,
}

impl Emu {
    /// Creates an Emulator 

    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0
        };
        
        // copying the fontset to the first FONTSET_SIZE bytes in the RAM
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        
        new_emu
    }

    /// Reset the system without having to create a new object Emu
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    /// pushes a given value to the stack
    /// 
    /// **Arguments**:
    /// 
    /// * 'val': given value to be pushed
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    /// pops a value from the stack at sp
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode & Executer
        self.execute(op);
    }

    /// Passes pointer to our screen buffer array to the frontend
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    /// Record a keypress
    pub fn keypress(&mut self, idx: usize, pressed:bool) {
        self.keys[idx] = pressed;
    }

    /// load a file into the RAM
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    /// Implements tick timers, each frame dt and st decrement
    /// if st == 1 before decrement, beeps
    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                // BEEP TODO (might not be implemented due to complexity)
            }
            self.st -= 1;
        }
    }

    /// Fetches the opcode for the current instruction
    /// Each opcode is exactly 2 bytes
    fn fetch(&mut self) -> u16 {
        // RAM stores values in u8, so we need to combine the higher and lower bytes
        let higher_byte= self.ram[self.pc as usize] as u16;
        let lower_byte= self.ram[(self.pc + 1) as usize] as u16;
        // Big Endian representation
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    /// Executes operation on the Emulator
    /// * 'op': given opcode that needs to be executed
    fn execute(&mut self, op: u16) {

        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // 0000
            // NOP : No operation
            (0, 0, 0, 0) => return,
            // 00E0
            // CLS : clear screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // 00EE
            // RET : return from subroutine
            // pop from stack and execute from that address
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },
            // 1NNN
            // JMP NNN : jump to given address NNN
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            },
            // 2NNN
            // CALL NNN : call subroutine at address NNN
            // we push the current pc on the stack and then
            // change pc to nnn
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            },
            // 3XNN
            // SKIP VX == NN : skip line if VX == NN
            // gives a similar functionality like an if else block
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },
            // 4XNN
            // SKIP VX != NN : skip line if VX != NN
            // gives a similar functiinality like an if else block
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },
            // 5XY0
            // SKIP VX == VY : skip line if VX == VY
            (5, _, _, _) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // 6XNN
            // VX = NN : sets the register VX to NN
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            },
            // 7XNN
            // VX += NN : increments register VX by NN
            // We use wrapping_add to avoid a panic from rustc
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },
            // 8XY0
            // VX = VY : sets register VX to VY
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },
            // 8XY1
            // VX |= VY
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },
            // 8XY2
            // VX &= VY
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },
            // 8XY3
            // VX ^= VY
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },
            // 8XY4
            // VX += VY
            // We need to set the carry flag, VF if there is an overflow
            // We use overflowing add and check for errors to avoid panic
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                
                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry {1} else {0};
                
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // 8XY5
            // VX -= VY
            // We need to set the carry flag, VF if there is an overflow
            // We use overflowing sub and check for errors to avoid panic
            // For underflow, CF (VF) is set to 0 and if there is no underflow
            // it is set to 1.
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                
                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow {0} else {1};
                
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // 8XY6
            // VX >>= 1
            // We need to catch the dropped bit and store it into the VF register
            // the dropped bit is the least significant bit (lsb)
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },
            // 8XY7
            // VX = VY - VX
            // Check underflow and set CF to 0 if there is an underflow, else 1
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                
                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow {0} else {1};

                self.v_reg[0xF] = new_vf;
                self.v_reg[x] = new_vx;
            },
            // 8XYE
            // VX <<= 1
            // Overflowed value is stored in VF
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[0xF] = msb;
                self.v_reg[x] <<= 1;
            },
            // 9XY0
            // SKIP VX != VY : skip line if VX != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // ANNN
            // I = NNN : sets I register to nnn
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            },
            // BNNN
            // JMP V0 + NNN : jumps to the value of V0 + nnn
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },
            // CXNN
            // VX = rand() & NN : gets a random number, AND it with nn
            // We use the random() function to generate a random num
            // We have to define u8 for rng so random() knows what
            // type of number to generate
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = rand::thread_rng().gen();
                self.v_reg[x] = rng & nn;
            },
            // DXYN
            // DRAW - unimplemented
            // Digit2 is x_coordinate
            // Digit3 is y_coordinate
            // Digit4 is number of rows
            (0xD, _, _, _) => {
                // Get the coordinates and number of rows
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                let num_rows = digit4;
                
                // Mutable flipped variable
                let mut flipped = false;

                // Iterate through each line in num_rows
                for y_line in 0..num_rows {
                    // get the row of pixels
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    // iterate through each pixel
                    for x_line in 0..8 {
                        // if it is set find the coordinates for x and y on screen
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            
                            // Find index of pixel in the screen as it is a 1-D array
                            let idx = x + SCREEN_WIDTH * y;
                            
                            // check if we are flipping the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                // set value of VX acc to flipped
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            // EX9E
            // SKIP KEY PRESS : skip the next line if the key stored in VX is pressed
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            // EXA1
            // SKIP KEY RELEASE : skip the next line if the key stored in VX is not pressed
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            // FX07
            // VX = DT : sets value of VX to that of DT
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            },
            // FX0A
            // WAIT KEY : waits for a key press, blocks execution of further ops
            // until any key is pressed and then stores that key into VX
            // if multiple keys are pressed the lowest indexed key is stored
            // We cannot use a loop outside of the inner loop as it would prevent
            // any key presses from being registered and thus making it a infinite loop
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    // Redo opcode
                    self.pc -= 2;
                }
            },
            // FX15
            // DT = VX
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            },
            // FX18
            // ST = VX
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            // FX1E
            // I += VX : adds VX to I register, if overflow set to 0
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },
            // FX29
            // I = FONT : set I to font_address
            // finds the address of the sprite to be printed and stores
            // it into the I register
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            },
            // FX33
            // I = BCD of VX
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },
            // FX55
            // STORE V0 - VX
            // Stores V0 thru VX in the RAM using the address in register I
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x{
                    self.ram[i + idx] = self.v_reg[idx];
                }
            },
            // FX65
            // LOAD V0 - VX
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x{
                    self.v_reg[idx] = self.ram[i + idx];
                }
            },
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}