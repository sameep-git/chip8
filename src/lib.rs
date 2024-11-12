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
        // Decode
        // Execute
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

}