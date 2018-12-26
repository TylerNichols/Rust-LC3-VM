// LC-3 VM referenced from https://justinmeiners.github.io/lc3-vm/
use std::io::Read;

enum Register {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC, /* program counter */
    COND,
    COUNT
}

enum MemoryRegister {
    KBSR = 0xFE00, /* keyboard status */
    KBDR = 0xFE02  /* keyboard data */
}

enum TrapCode {
    GETC = 0x20,  /* get character from keyboard */
    OUT = 0x21,   /* output a character */
    PUTS = 0x22,  /* output a word string */
    IN = 0x23,    /* input a string */
    PUTSP = 0x24, /* output a byte string */
    HALT = 0x25,   /* halt the program */
    ERR
}

impl TrapCode {
    fn from_u16(value: u16) -> TrapCode {
        let mask = 0b11111111;
        let trap = mask & value;
        match trap {
            0x20 => TrapCode::GETC, 
            0x21 => TrapCode::OUT, 
            0x22 => TrapCode::PUTS, 
            0x23 => TrapCode::IN, 
            0x24 => TrapCode::PUTSP, 
            0x25 => TrapCode::HALT,
            _ => TrapCode::ERR
        }
    }
}

enum OpCode {
    BR = 0, /* branch */
    ADD,    /* add  */
    LD,     /* load */
    ST,     /* store */
    JSR,    /* jump register */
    AND,    /* bitwise and */
    LDR,    /* load register */
    STR,    /* store register */
    RTI,    /* unused */
    NOT,    /* bitwise not */
    LDI,    /* load indirect */
    STI,    /* store indirect */
    JMP,    /* jump */
    RES,    /* reserved (unused) */
    LEA,    /* load effective address */
    TRAP,    /* execute trap */
    ERR
}

impl OpCode {
    fn from_u16(value: u16) -> OpCode {
        match value {
            0 => OpCode::BR,
            1 => OpCode::ADD,
            2 => OpCode::LD,
            3 => OpCode::ST,
            4 => OpCode::JSR,
            5 => OpCode::AND,
            6 => OpCode::LDR,
            7 => OpCode::STR,
            8 => OpCode::RTI,
            9 => OpCode::NOT,
            10 => OpCode::LDI,
            11 => OpCode::STI,
            12 => OpCode::JMP,
            13 => OpCode::RES,
            14 => OpCode::LEA,
            15 => OpCode::TRAP,
            _ => OpCode::ERR
        }
    }
}

enum FlagLogical {
    POS = 1 << 0, /* P */
    ZRO = 1 << 1, /* Z */
    NEG = 1 << 2, /* N */
}

fn read_program_file(location: String,memory: &mut [u16], mut reg: [u16; Register::COUNT as usize]) -> u16 {
    let path = std::path::Path::new(&location);

    // Open the path in read-only mode, returns `io::Result<File>`
    let file = match std::fs::File::open(&path) {
        Err(_why) => panic!("OH NO"),
        Ok(file) => file,
    };

    // Take first u16 as origin and set PC
    let offset = get_mem_offset(&file);
    print!("offset bytes = {}\n", offset);
    fill_mem_from_offset(file, memory, offset);

    return offset
}

fn fill_mem_from_offset(file: std::fs::File, memory: &mut [u16], offset: u16) {
    let mut b = 0;
    for byte in file.bytes() {
        if (b % 2 == 0) {
            memory[(offset + b/2) as usize] = (byte.unwrap() as u16) << 8;
        } else {
            memory[(offset + b/2) as usize] |= byte.unwrap() as u16;
        }
        b = b + 1;
    }
}

fn get_mem_offset(file: &std::fs::File) -> u16 {
    let mut handle = file.take(2);
    let mut buf = [0u8; 2];
    handle.read(&mut buf);
    return swap16(buf);
}

fn swap16(bytes: [u8; 2]) -> u16 {
    let number = (bytes[0] as u16) << 8 | bytes[1] as u16;
    return number;
}

fn mem_read(address: u16, memory: &mut[u16]) -> u16 {
    if (address == MemoryRegister::KBSR as u16) {
        if (true) {
            memory[MemoryRegister::KBSR as usize] = 1 << 15;
            memory[MemoryRegister::KBDR as usize] = 0;
        }
        else {
            memory[MemoryRegister::KBSR as usize] = 0;
        }
    }
    return memory[address as usize];
}

fn increment_pc(mut reg: [u16; Register::COUNT as usize]) {
    reg[Register::PC as usize] += 1
}

fn mem_write(address: u16, value: u16, mut memory: [u16; std::u16::MAX as usize]) {
    memory[address as usize] = value;
}

fn main() {
    // VM hardware emulation variables
    let mut memory: [u16; std::u16::MAX as usize] = [0; std::u16::MAX as usize];
    let mut reg: [u16; Register::COUNT as usize] = [0; Register::COUNT as usize];
    
    // Reading in program
    let offset = read_program_file("dc126ff2-4c0c-4586-9723-38eda91bbd55".to_string(), &mut memory, reg);
    reg[Register::PC as usize] = offset;
    let running = true;

    while running {
        let instruction = mem_read(reg[Register::PC as usize], &mut memory);
        reg[Register::PC as usize] += 1;
        let operation = instruction >> 12;
        print!("instruction {:b}, PC {} , operation {} ", instruction, reg[Register::PC as usize], operation);
        match OpCode::from_u16(operation) {
            OpCode::ADD => println!("ADD"),
            OpCode::AND => println!("AND"),
            OpCode::NOT => println!("NOT"),
            OpCode::BR => println!("BR"),
            OpCode::JMP => println!("JMP"),
            OpCode::JSR => println!("JSR"),
            OpCode::LD => println!("LD"),
            OpCode::LDI => println!("LDI"),
            OpCode::LDR => println!("LDR"),
            OpCode::LEA => println!("LEA"),
            OpCode::ST => println!("ST"),
            OpCode::STI => println!("STI"),
            OpCode::STR => println!("STR"),
            OpCode::TRAP => {
                println!("TRAP");
                if (!handle_trap(instruction)) {
                    break;
                }
            },
            OpCode::RES => println!("RES"),
            OpCode::RTI => println!("RTI"),
            _ => println!("No match found for op {}", operation),                                            
        }
    }
}

// returns false if we are breaking execution
fn handle_trap(instruction: u16) -> bool { 
    let trapCode = TrapCode::from_u16(instruction);

    match(trapCode) {
        TrapCode::GETC => true,  /* get character from keyboard */
        TrapCode::OUT => true,   /* output a character */
        TrapCode::PUTS => true,  /* output a word string */
        TrapCode::IN => true,    /* input a string */
        TrapCode::PUTSP => true, /* output a byte string */
        TrapCode::HALT => false,   /* halt the program */
        TrapCode::ERR => false
    }
}

