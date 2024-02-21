use rand::random;

pub const SCREEN_COLS: usize = 64;
pub const SCREEN_ROWS: usize = 32;

const SPRITE_ROW_SIZE: u16 = 8;
const RAM_SIZE: usize = 4096;
const NUM_GP_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONT_WIDTH: u16 = 8;
const FONT_HEIGHT: u16 = 5;
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

// bitflags! {
//     struct OpCode: u16 {
//         const hx0 = 0x000f;
//         const hx1 = 0x00f0;
//         const hx2 = 0x0f00;
//         const hx3 = 0xf000;
//     }
// }

pub struct Emulator {
    pc: u16,
    memory: [u8; RAM_SIZE],
    v_reg: [u8; NUM_GP_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    screen: [bool; SCREEN_COLS * SCREEN_ROWS],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut emu = Self {
            pc: START_ADDR,
            memory: [0; RAM_SIZE],
            v_reg: [0; NUM_GP_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            screen: [false; SCREEN_COLS * SCREEN_ROWS],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        // Copy fontset to the beginning of the memory
        emu.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.memory = [0; RAM_SIZE];
        self.v_reg = [0; NUM_GP_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.screen = [false; SCREEN_COLS * SCREEN_ROWS];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, is_pressed: bool) {
        self.keys[idx] = is_pressed;
    }

    pub fn load_to_memory(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = START_ADDR as usize + data.len();
        self.memory[start..end].copy_from_slice(data);
    }

    pub fn timer_tick(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // beep
            }
            self.st -= 1;
        }
    }

    pub fn tick(&mut self) {
        // Handle cpu cycle: Fetch -> Decode -> Execute
        // Fetch
        let opcode = self.fetch();

        // Decode & Execute
        let hx0 = (opcode & 0x000f);
        let hx1 = (opcode & 0x00f0) >> 4;
        let hx2 = (opcode & 0x0f00) >> 8;
        let hx3 = (opcode & 0xf000) >> 12;

        match (hx3, hx2, hx1, hx0) {
            (0x0, 0x0, 0x0, 0x0) => return, // NOP
            (0x0, 0x0, 0xe, 0x0) => {       // CLS
                self.screen = [false; SCREEN_COLS * SCREEN_ROWS];
            }
            (0x0, 0x0, 0xe, 0xe) => {       // RET
                let addr = self.pop();
                self.pc = addr;
            }
            (0x1, _, _, _) => {       // JMP
                let addr = opcode & 0x0fff;
                self.pc = addr;
            }
            (0x2, _, _, _) => {       // CALL
                let addr = opcode & 0x0fff;
                self.push(self.pc);
                self.pc = addr;
            }
            (0x3, _, _, _) => {       // SKIP IF VX == N
                let x = hx2 as usize;
                let n = (opcode & 0x00ff) as u8;
                if self.v_reg[x] == n {
                    self.pc += 2;
                }
            }
            (0x4, _, _, _) => {       // SKIP IF NOT VX != N
                let x = hx2 as usize;
                let n = (opcode & 0x00ff) as u8;
                if self.v_reg[x] != n {
                    self.pc += 2;
                }
            }
            (0x5, _, _, 0x0) => {       // SKIP IF VX == VY
                let x = hx2 as usize;
                let y = hx1 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            (0x6, _, _, _) => {       // SET VX = N
                let x = hx2 as usize;
                let n = (opcode & 0x00ff) as u8;
                self.v_reg[x] = n;
            }
            (0x7, _, _, _) => {       // INC VX BY N
                let x = hx2 as usize;
                let n = (opcode & 0x00ff) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(n);
            }
            (0x8, _, _, 0x0) => {       // SET VX = VY
                let x = hx2 as usize;
                let y = hx1 as usize;
                self.v_reg[x] = self.v_reg[y];
            }
            (0x8, _, _, 0x1) => {       // VX = VX OR VY
                let x = hx2 as usize;
                let y = hx1 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            (0x8, _, _, 0x2) => {       // VX = VX AND VY
                let x = hx2 as usize;
                let y = hx1 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }
            (0x8, _, _, 0x3) => {       // VX = VX XOR VY
                let x = hx2 as usize;
                let y = hx1 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }
            (0x8, _, _, 0x4) => {       // INC VX BY N, CARRY
                let x = hx2 as usize;
                let y = hx1 as usize;
                let (vx, c) = self.v_reg[x].overflowing_add(self.v_reg[y]);

                self.v_reg[x] = vx;
                self.v_reg[0xf] = if c { 1 } else { 0 };
            }
            (0x8, _, _, 0x5) => {       // DEC VX BY N, BORROW
                let x = hx2 as usize;
                let y = hx1 as usize;
                let (vx, b) = self.v_reg[x].overflowing_sub(self.v_reg[y]);

                self.v_reg[x] = vx;
                self.v_reg[0xf] = if b { 0 } else { 1 };
            }
            (0x8, _, _, 0x6) => {       // RSH
                let x = hx2 as usize;
                let lsb = self.v_reg[x] & 0x1;

                self.v_reg[x] >>= 1;
                self.v_reg[0xf] = lsb;
            }
            (0x8, _, _, 0x7) => {       // SUB VY BY VX AND ASSIGN IT TO VX, BORROW
                let x = hx2 as usize;
                let y = hx1 as usize;
                let (vx, b) = self.v_reg[y].overflowing_sub(self.v_reg[x]);

                self.v_reg[x] = vx;
                self.v_reg[0xf] = if b { 0 } else { 1 };
            }
            (0x8, _, _, 0xe) => {       // LSH
                let x = hx2 as usize;
                let msb = self.v_reg[x] & (0x1 << 7);

                self.v_reg[x] <<= 1;
                self.v_reg[0xf] = msb;
            }
            (0x9, _, _, 0x0) => {       // SKIP IF VX != VY
                let x = hx2 as usize;
                let y = hx1 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            (0xa, _, _, _) => {
                let addr = opcode & 0x0fff;
                self.i_reg = addr;
            }
            (0xc, _, _, _) => {       // SKIP IF VX != VY
                let x = hx2 as usize;
                let n = (opcode & 0x00ff) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & n;
            }
            (0xd, _, _, _) => {       // DRAW AT X, Y, N PIXELS FOR SPRITE HEIGHT
                let coord_x = self.v_reg[hx2 as usize] as u16;
                let coord_y = self.v_reg[hx1 as usize] as u16;
                let n = hx0;

                let mut flipped = false;
                for sprite_y in 0..n {
                    let addr = self.i_reg + sprite_y;
                    let pixels = self.memory[addr as usize];

                    for sprite_x in 0..SPRITE_ROW_SIZE {
                        if (pixels & (1 << sprite_x)) != 0 {
                            // Wrap around overflow
                            let x = (coord_x + sprite_x) as usize % SCREEN_COLS;
                            let y = (coord_y + sprite_y) as usize % SCREEN_ROWS;

                            let idx = x + y * SCREEN_COLS; // index of pixel in `screen` buffer

                            flipped |= self.screen[idx]; // check if any pixel got flipped
                            self.screen[idx] ^= true; // toggle the pixel
                        }
                    }
                }

                // in case of pixel flip detected set VF flag
                if flipped {
                    self.v_reg[0xf] = 1;
                } else {
                    self.v_reg[0xf] = 0;
                }
            }
            (0xe, _, 0x9, 0xe) => {       // SKIP IF KEY IS PRESSED
                let x = hx2 as usize;
                let key = self.keys[self.v_reg[x] as usize];
                if key {
                    self.pc += 2;
                }
            }
            (0xe, _, 0xa, 0x1) => {       // SKIP IF KEY IS RELEASED
                let x = hx2 as usize;
                let key = self.keys[self.v_reg[x] as usize];
                if !key {
                    self.pc += 2;
                }
            }
            (0xf, _, 0x0, 0x7) => {       // VX = DT
                let x = hx2 as usize;
                self.v_reg[x] = self.dt;
            }
            (0xf, _, 0x0, 0xa) => {       // WAIT KEY
                let x = hx2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }
            (0xf, _, 0x1, 0x5) => {       // DT = VX
                let x = hx2 as usize;
                self.dt = self.v_reg[x];
            }
            (0xf, _, 0x1, 0x8) => {       // ST = VX
                let x = hx2 as usize;
                self.st = self.v_reg[x];
            }
            (0xf, _, 0x1, 0xe) => {       // I += VX
                let x = hx2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            (0xf, _, 0x2, 0x9) => {       // LOAD CHARACTER FONT ADDR IN I
                let x = hx2 as usize;
                let char = self.v_reg[x] as u16;
                self.i_reg = char * FONT_HEIGHT;
            }
            (0xf, _, 0x3, 0x3) => {       // I = BCD of VX
                let x = hx2 as usize;
                let vx = self.v_reg[x] as f32;

                // d2,d1,d0
                let d2 = (vx / 100f32).floor() as u8;
                let d1 = ((vx / 10f32) % 10f32).floor() as u8;
                let d0 = (vx % 10f32) as u8;

                let i = self.i_reg as usize;
                self.memory[i] = d2;
                self.memory[i + 1] = d1;
                self.memory[i + 2] = d0;
            }
            (0xf, _, 0x5, 0x5) => {       // STORE V0 TILL VX INTO MEMORY
                let x = hx2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.memory[i + idx] = self.v_reg[idx];
                }
            }
            (0xf, _, 0x6, 0x5) => {       // LOAD FROM MEMORY INTO V0 TILL VX
                let x = hx2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.memory[i + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unknown OpCode: {:4x}", opcode)
        }
    }

    fn fetch(&mut self) -> u16 {
        let hi_byte = self.memory[self.pc as usize] as u16;
        let lo_byte = self.memory[(self.pc + 1) as usize] as u16;
        let opcode = (hi_byte << 8) | lo_byte;
        self.pc += 2;
        opcode
    }

    fn push(&mut self, x: u16) {
        self.stack[self.sp as usize] = x;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[(self.sp - 1) as usize]
    }
}