use std::time::Duration;


fn get_macros(scripts: &mut Vec<Vec<String>>, global_macros: &mut Vec<(String, Vec<String>, Vec<Vec<String>>)>) -> Vec<(String, Vec<String>, Vec<Vec<String>>)> {
    let mut macros = vec![];
    let mut line_number = 0;
    while line_number < scripts.len() {
        if scripts[line_number][0] == "!macro" {
            let start_offset = if scripts[line_number][1] == "-export" { 1 } else { 0 };
            // parsing the macro
            let name = scripts[line_number][1 + start_offset].clone();
            let args = scripts[line_number][2 + start_offset..].iter().map(|t| t.clone()).collect::<Vec<String>>();
            // finding the ending line
            let end_line = scripts[line_number + 1..]
                .iter()
                .position(|line| line[0] == "!end")
                .unwrap() + line_number + 1;
            let macro_def = (
                name,
                args,
                scripts[line_number + 1..end_line]
                    .iter()
                    .map(|line| 
                        line.iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>())
                    .collect::<Vec<Vec<String>>>());
            if start_offset == 1 { global_macros.push(macro_def); }
            else { macros.push(macro_def); }
            scripts.drain(line_number..=end_line);
            continue;
        }
        line_number += 1;
    } macros
}

fn expand_macro_calls(lines: &mut Vec<Vec<String>>, macros: &Vec<(String, Vec<String>, Vec<Vec<String>>)>) {
    let mut line_number = 0;
    while line_number < lines.len() {
        if macros.iter().any(|(m,..)| lines[line_number][0] == *m) {
            let starting_line = line_number;
            let mac = lines.remove(line_number);
            let (_name, args, body) = macros.iter().find(|(m,..)| m == &mac[0]).unwrap();
            for line in body {
                // replacing any args
                let new_line = line.iter().map(|t| {
                    if let Some(arg_index) = args.iter().position(|a| a == t) {
                        mac[arg_index + 1].to_string()
                    } else { t.to_string() }
                }).collect::<Vec<String>>();
                lines.insert(line_number, new_line);
                line_number += 1;
            }
            line_number = starting_line;  // making sure a macro can recursively expand additional macros
            continue;
        }
        line_number += 1;
    }
}

// op_code/op_type combined, num args, name
static OP_CODES: &[(u8, usize, [usize; 3], &str)] = &[
    (0b000_00000, 0, [0, 0, 0], "Nop"),
    (0b000_00001, 0, [0, 0, 0], "SetDspInX"),
    (0b000_00010, 0, [0, 0, 0], "SetDspInY"),
    (0b000_00011, 0, [0, 0, 0], "SetDspInCol"),
    (0b000_00100, 2, [1, 0, 0], "Plot"),
    (0b000_00101, 0, [0, 0, 0], "Kill"),
    (0b000_00110, 0, [0, 0, 0], "SetPtr"),
    (0b000_00111, 0, [0, 0, 0], "PgcL"),
    (0b000_01000, 0, [0, 0, 0], "PgcR"),
    (0b000_01001, 0, [0, 0, 0], "Plt"),
    (0b000_01010, 1, [1, 0, 0], "SetPage"),
    (0b000_01011, 2, [1, 0, 0], "Goto"),
    (0b000_01100, 2, [0, 1, 0], "GotoReg"),
    (0b000_01101, 1, [0, 0, 0], "SetPageReg"),
    (0b001_00000, 0, [0, 0, 0], "Add"),
    (0b001_00001, 0, [0, 0, 0], "Sub"),
    (0b001_00010, 0, [0, 0, 0], "Inc"),
    (0b001_00011, 0, [0, 0, 0], "Dec"),
    (0b001_00100, 0, [0, 0, 0], "ThruL"),
    (0b001_00101, 0, [0, 0, 0], "ThruR"),
    (0b001_00110, 0, [0, 0, 0], "And"),
    (0b001_00111, 0, [0, 0, 0], "Or"),
    (0b001_01000, 0, [0, 0, 0], "ShftL"),
    (0b001_01001, 0, [0, 0, 0], "ShftR"),
    (0b001_01010, 1, [1, 0, 0], "LdiL"),
    (0b001_01011, 1, [1, 0, 0], "LdiR"),
    (0b010_00000, 0, [0, 0, 0], "Eq"),
    (0b010_00001, 0, [0, 0, 0], "Less"),
    (0b010_00010, 0, [0, 0, 0], "Grtr"),
    (0b010_00011, 0, [0, 0, 0], "OvrFlow"),
    (0b010_00100, 0, [0, 0, 0], "SetC"),
    (0b010_00101, 0, [0, 0, 0], "RsetC"),
    (0b010_00110, 0, [0, 0, 0], "Zero"),
    (0b010_00111, 0, [0, 0, 0], "RsetO"),
    (0b011_00000, 1, [0, 0, 0], "LodL"),
    (0b011_00001, 1, [0, 0, 0], "LodR"),
    (0b011_00010, 1, [0, 0, 0], "WrtO"),
    (0b011_00011, 0, [0, 0, 0], "PtrL"),
    (0b011_00100, 0, [0, 0, 0], "PtrR"),
    (0b011_00101, 0, [0, 0, 0], "PtrO"),
    (0b011_00110, 2, [0, 0, 0], "Ldi"),
    (0b100_00000, 1, [0, 0, 0], "RLodL"),
    (0b100_00001, 1, [0, 0, 0], "RLodR"),
    (0b100_00010, 1, [0, 0, 0], "RWrtO"),
    (0b100_00011, 0, [0, 0, 0], "RPtrL"),
    (0b100_00100, 0, [0, 0, 0], "RPtrR"),
    (0b100_00101, 0, [0, 0, 0], "RPtrO"),
    (0b100_00110, 2, [0, 0, 0], "RLdi"),
    (0b101_00000, 1, [0, 0, 0], "DLodL"),
    (0b101_00001, 1, [0, 0, 0], "DLodR"),
    (0b101_00010, 1, [0, 0, 0], "DWrtO"),
    (0b101_00011, 0, [0, 0, 0], "DPtrL"),
    (0b101_00100, 0, [0, 0, 0], "DPtrR"),
    (0b101_00101, 0, [0, 0, 0], "DPtrO"),
    (0b101_00110, 2, [0, 0, 0], "DLdi"),
    (0b110_00000, 1, [1, 0, 0], "Jmp"),
    (0b110_00001, 1, [1, 0, 0], "Jiz"),
    (0b110_00010, 1, [1, 0, 0], "Jnz"),
    (0b110_00011, 1, [0, 0, 0], "JmpR"),
    (0b110_00100, 1, [0, 0, 0], "JizR"),
    (0b110_00101, 1, [0, 0, 0], "JnzR"),
    (0b111_00000, 0, [0, 0, 0], "Pop"),
    (0b111_00001, 0, [0, 0, 0], "TopL"),
    (0b111_00010, 0, [0, 0, 0], "TopR"),
    (0b111_00011, 0, [0, 0, 0], "PshO"),
    (0b111_00100, 1, [1, 0, 0],	"PshCon"),
];
static REGISTERS: &[&str] = &[
    "rda",
    "rdb",
    "rdc",
    "rdd",
    "rde",
    "rdf",
    "rdg",
    "rdh",
    "rdi",
    "rdj",
    "rdk",
    "rdl",
];

fn compile_script(pages: &mut Vec<(Vec<Vec<String>>, String)>, headers: &Vec<(String, usize, usize)>, script_index: usize) -> Vec<u32> {
    for line_index in 0..pages[script_index].0.len() {
        for token_index in 0..pages[script_index].0[line_index].len() {
            if let Some(reg_index) = REGISTERS.iter().position(|r| r == &pages[script_index].0[line_index][token_index]) {
                pages[script_index].0[line_index][token_index] = (reg_index as u32).to_string();
            }
            if let Some(header_index) = headers.iter().position(|h| h.0 == *pages[script_index].0[line_index][token_index]) {
                pages[script_index].0[line_index][token_index] = (headers[header_index].1).to_string();
            }
            let token = &pages[script_index].0[line_index][token_index];
            if let Some(page_index) = pages.iter().position(|(_, page_name)| page_name == token) {
                pages[script_index].0[line_index][token_index] = page_index.to_string();
            }
        }
    }
    println!("Final Tokens: {:?}", pages[script_index].0);
    let mut bytecode = vec![];
    for line in &pages[script_index].0 {
        // replacing any headers mentioned with their index
        let op = OP_CODES.iter().find(|(_, _, _, name)| name == &line[0]);
        if let Some(op) = op {
            let mut instruction = (op.0 as u32) << 24;
            for i in 0..op.1 {
                instruction |= (line[i + 1].parse::<u8>().unwrap() as u32) << (24 - 8 * (i + 1 + op.2[i]));
            }
            bytecode.push(instruction);
        }
    } bytecode
}

// name line page
fn generate_headers(script: &Vec<Vec<String>>, page: usize) -> Vec<(String, usize, usize)> {
    // calculating header indexes
    let mut true_index = 0;
    let mut headers = vec![];
    for (_index, line) in script.iter().enumerate() {
        // checking for a header defintion
        if ["!header", "!end", "!loop"].contains(&&*line[0]) {
            // getting the name
            headers.push((line[1].clone(), true_index, page));
            continue;  // this isn't a valid instruction and as such shouldn't be included in the bytecode
        }
        // invalid instruction, skipping (maybe a comment or something)
        if OP_CODES.iter().find(|(_, _, _, name)| name == &line[0]).is_none() {  continue; }
        true_index += 1;
    } headers
}

fn main() {
    let script = std::fs::read_to_string("scripts/test_program.mca").unwrap();
    let mut script = script
        .lines()
        .map(|line| line.split(" ").collect::<Vec<&str>>())
        .map(|mut line| {
            line.retain(|token| !token.is_empty());
            line.into_iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>();
    script.retain(|line| !line.is_empty());
    let mut pages = vec![(vec![], "main".to_string())];
    let mut global_macros = vec![];
    for line in script {
        if line[0] == "!page" { pages.push((vec![], line[1].clone())); }
        else { pages.last_mut().unwrap().0.push(line); }
    }
    let mut headers = vec![];
    for (page, script) in pages.iter_mut().enumerate() {
        //println!("Tokens: {:?}", script);
        // collecting all macros
        let macros = get_macros(&mut script.0, &mut global_macros);
        //println!("Macros: {:?}", macros);
        //println!("Tokens: {:?}", script);
        expand_macro_calls(&mut script.0, &macros);
        expand_macro_calls(&mut script.0, &global_macros);
        //println!("Tokens: {:?}", script);
        headers.append(&mut generate_headers(&mut script.0, page));
    }

    let mut program_bytes = vec![];
    for script_index in 0..pages.len() {
        let bytes = compile_script(&mut pages, &headers, script_index);
        println!("{}", bytes.iter().enumerate()
            .map(|(index, byte)| format!("{:>3}: {}\n\n", index, format!("{:032b}", byte)
                .chars().map(|c| format!(" {} ", c)).collect::<String>()
            )).collect::<String>());
        program_bytes.push(bytes);
    }

    // running the emulator
    run_emmulator(program_bytes);
}

fn run_emmulator(program_bytes: Vec<Vec<u32>>) {
    // memory
    let mut registers = [0u8; 256];
    let mut ram = [0u8; 256];
    let mut stack = [0u8; 64];
    let mut disc = [0u8; 256];

    // dedicated registers
    let mut program_counter = 0u16;
    let mut alu_left  = 0u8;
    let mut alu_right = 0u8;
    let mut alu_out   = 0u8;
    let mut pointer_reg = 0u8;
    let mut overflow_flag = false;
    let mut condition_flag = false;
    let mut next_page_reg = 0u8;
    let mut x_coord_reg = 0u8;
    let mut y_coord_reg = 0u8;
    let mut color_reg = 0u8;

    let mut cycle = 0;  // just for debug stuff ig
    let time_start = std::time::Instant::now();

    loop {
        cycle += 1;

        let instruction = program_bytes[(program_counter >> 8) as usize][(program_counter & 0xFF) as usize];
        let op_code = (instruction >> 24) as u8;
        let reg_or_add = ((instruction >> 16) & 0xFF) as u8;
        let immediate = ((instruction >> 8) & 0xFF) as u8;
        let immediate_2 = (instruction & 0xFF) as u8;
        let mut jumped = false;
        match op_code {
            0b000_00001 => { x_coord_reg = alu_out; },  // "SetDspInX"
            0b000_00010 => { y_coord_reg = alu_out; },  // "SetDspInY"
            0b000_00011 => { color_reg = alu_out; },  // "SetDspInCol"
            0b000_00100 => {  },  // "Plot"
            0b000_00101 => { break; },  // "Kill"
            0b000_00110 => { pointer_reg = alu_out; },  // "SetPtr"
            0b000_00111 => { alu_left = (program_counter & 0xFF) as u8; },  // "PgcL"
            0b000_01000 => { alu_right = (program_counter & 0xFF) as u8; },  // "PgcR"
            0b000_01001 => {  },  // "Plt"
            0b000_01010 => { next_page_reg = immediate_2; },  // "SetPage"
            0b000_01011 => { run_lu(op_code, condition_flag, next_page_reg, &mut program_counter, &registers, &mut jumped, immediate, immediate_2, reg_or_add); },  // "Goto"
            0b000_01100 => { run_lu(op_code, condition_flag, next_page_reg, &mut program_counter, &registers, &mut jumped, immediate, immediate_2, reg_or_add); },  // "GotoReg"
            0b000_01101 => { next_page_reg = registers[reg_or_add as usize]; },  // "SetPageReg"
            0b001_00000 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "Add"
            0b001_00001 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "Sub"
            0b001_00010 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "Inc"
            0b001_00011 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "Dec"
            0b001_00100 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "ThruL"
            0b001_00101 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "ThruR"
            0b001_00110 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "And"
            0b001_00111 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "Or"
            0b001_01000 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "ShftL"
            0b001_01001 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "ShftR"
            0b001_01010 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "LdiL"
            0b001_01011 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "LdiR"
            0b010_00000 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "Eq"
            0b010_00001 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "Less"
            0b010_00010 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "Grtr"
            0b010_00011 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "OvrFlow"
            0b010_00100 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "SetC"
            0b010_00101 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "RsetC"
            0b010_00110 => { run_alu(op_code, immediate, &mut alu_left, &mut alu_right, &mut alu_out, &mut overflow_flag, &mut condition_flag); },  // "Zero"
            0b011_00000 => { alu_left = registers[reg_or_add as usize]; },  // "LodL"
            0b011_00001 => { alu_right = registers[reg_or_add as usize]; },  // "LodR"
            0b011_00010 => { registers[reg_or_add as usize] = alu_out; },  // "WrtO"
            0b011_00011 => { alu_left = pointer_reg; },  // "PtrL"
            0b011_00100 => { alu_right = pointer_reg; },  // "PtrR"
            0b011_00101 => { pointer_reg = alu_out; },  // "PtrO"
            0b011_00110 => { registers[reg_or_add as usize] = immediate; },  // "Ldi"
            0b100_00000 => { alu_left = ram[reg_or_add as usize]; },  // "RLodL"
            0b100_00001 => { alu_right = ram[reg_or_add as usize]; },  // "RLodR"
            0b100_00010 => { ram[reg_or_add as usize] = alu_out; },  // "RWrtO"
            0b100_00011 => { alu_left = ram[pointer_reg as usize]; },  // "RPtrL"
            0b100_00100 => { alu_right = ram[pointer_reg as usize]; },  // "RPtrR"
            0b100_00101 => { ram[pointer_reg as usize] = alu_out; },  // "RPtrO"
            0b100_00110 => { ram[reg_or_add as usize] = immediate; },  // "RLdi"
            0b101_00000 => { alu_left = disc[reg_or_add as usize]; },  // "DLodL"
            0b101_00001 => { alu_right = disc[reg_or_add as usize]; },  // "DLodR"
            0b101_00010 => { disc[reg_or_add as usize] = alu_out; },  // "DWrtO"
            0b101_00011 => { alu_left = disc[pointer_reg as usize]; },  // "DPtrL"
            0b101_00100 => { alu_right = disc[pointer_reg as usize]; },  // "DPtrR"
            0b101_00101 => { disc[pointer_reg as usize] = alu_out; },  // "DPtrO"
            0b101_00110 => { disc[reg_or_add as usize] = immediate; },  // "DLdi"
            0b110_00000 => { run_lu(op_code, condition_flag, next_page_reg, &mut program_counter, &registers, &mut jumped, immediate, immediate_2, reg_or_add) },  // "Jmp"
            0b110_00001 => { run_lu(op_code, condition_flag, next_page_reg, &mut program_counter, &registers, &mut jumped, immediate, immediate_2, reg_or_add) },  // "Jiz"
            0b110_00010 => { run_lu(op_code, condition_flag, next_page_reg, &mut program_counter, &registers, &mut jumped, immediate, immediate_2, reg_or_add) },  // "Jnz"
            0b110_00011 => { run_lu(op_code, condition_flag, next_page_reg, &mut program_counter, &registers, &mut jumped, immediate, immediate_2, reg_or_add) },  // "JmpR"
            0b110_00100 => { run_lu(op_code, condition_flag, next_page_reg, &mut program_counter, &registers, &mut jumped, immediate, immediate_2, reg_or_add) },  // "JizR"
            0b110_00101 => { run_lu(op_code, condition_flag, next_page_reg, &mut program_counter, &registers, &mut jumped, immediate, immediate_2, reg_or_add) },  // "JnzR"
            0b111_00000 => { for i in 0..stack.len() - 1 { stack[i] = stack[i + 1] } stack[stack.len() - 1] = 0; },  // "Pop"
            0b111_00001 => { alu_left = stack[0]; },  // "TopL"
            0b111_00010 => { alu_right = stack[0]; },  // "TopR"
            0b111_00011 => { for i in (0..stack.len() - 1).rev() { stack[i + 1] = stack[i] } stack[0] = alu_out; },  // "PshO"
            0b111_00100 => { for i in (0..stack.len() - 1).rev() { stack[i + 1] = stack[i] } stack[0] = immediate; },  // "PshCon"
            _ => {}
        }
        if !jumped {
            program_counter += 1;
        }

        //println!("PC: {}, ALU Left: {}, ALU Right: {}, ALU Out: {}, Overflow Flag: {}, Condition Flag: {}", program_counter, alu_left, alu_right, alu_out, overflow_flag, condition_flag);
        //println!("First 10 registers: {:?}", &registers[..10]);
        //println!("First 20 of stack: {:?}", &stack[..20]);
        //std::thread::sleep(Duration::from_secs_f32(0.1));
    }

    let end = time_start.elapsed().as_secs_f64() / cycle as f64;
    println!("Average Cycle Time: {} seconds, which is about {} operations per second", end, 1.0 / end);
}

fn run_alu(op_code: u8, immediate: u8, left: &mut u8, right: &mut u8, out: &mut u8, overflow_flag: &mut bool, condition_flag: &mut bool) {
    match op_code {
        0b001_00000 => {
            if left.checked_add(*right).is_none() { *overflow_flag = true; }
            *out = *left + *right;
        },  // "Add"
        0b001_00001 => { *out = *left - *right; },  // "Sub"
        0b001_00010 => {
            if left.checked_add(1).is_none() { *overflow_flag = true; }
            *out = *left + 1;
        },  // "Inc"
        0b001_00011 => { *out = *left - 1; },  // "Dec"
        0b001_00100 => { *out = *left; },  // "ThruL"
        0b001_00101 => { *out = *right; },  // "ThruR"
        0b001_00110 => { *out = *left & *right; },  // "And"
        0b001_00111 => { *out = *left | *right; },  // "Or"
        0b001_01000 => {
            if left.checked_shl(1).is_none() { *overflow_flag = true; }
            *out = *left << 1;
        },  // "ShftL"
        0b001_01001 => { *out = *left >> 1; },  // "ShftR"
        0b001_01010 => { *left = immediate; },  // "LdiL"
        0b001_01011 => { *right = immediate; },  // "LdiR"
        0b010_00000 => { *condition_flag = *left == *right; },  // "Eq"
        0b010_00001 => { *condition_flag = *left < *right; },  // "Less"
        0b010_00010 => { *condition_flag = *left > *right; },  // "Grtr"
        0b010_00011 => { *condition_flag = *overflow_flag; },  // "OvrFlow"
        0b010_00100 => { *condition_flag = true; },  // "SetC"
        0b010_00101 => { *condition_flag = false; },  // "RsetC"
        0b010_00110 => { *condition_flag = *out == 0; },  // "Zero"
        _ => {}
    }
}

fn run_lu(op_code: u8, condition_flag: bool, next_page_reg: u8, program_counter_reg: &mut u16, registers: &[u8; 256], jumped: &mut bool, immediate: u8, immediate_2: u8, reg_or_add: u8) {
    match op_code {
        0b000_01011 => { *jumped = true; *program_counter_reg = (immediate as u16 | ((immediate_2 as u16) << 8)); },  // "Goto"
        0b000_01100 => { *jumped = true; *program_counter_reg = (registers[reg_or_add as usize]) as u16 | ((next_page_reg as u16) << 8); },  // "GotoReg"
        0b110_00000 => { *jumped = true; *program_counter_reg = reg_or_add as u16 | ((next_page_reg as u16) << 8); },  // "Jmp"
        0b110_00001 => { if condition_flag { *jumped = true; *program_counter_reg = immediate as u16 | ((next_page_reg as u16) << 8); } },  // "Jiz"
        0b110_00010 => { if !condition_flag { *jumped = true; *program_counter_reg = immediate as u16 | ((next_page_reg as u16) << 8); } },  // "Jnz"
        0b110_00011 => { *jumped = true; *program_counter_reg = (registers[reg_or_add as usize]) as u16 | ((next_page_reg as u16) << 8); },  // "JmpR"
        0b110_00100 => { if condition_flag { *jumped = true; *program_counter_reg = (registers[reg_or_add as usize]) as u16 | ((next_page_reg as u16) << 8); } },  // "JizR"
        0b110_00101 => { if !condition_flag { *jumped = true; *program_counter_reg = (registers[reg_or_add as usize]) as u16 | ((next_page_reg as u16) << 8); } },  // "JnzR"
        _ => {}
    }
}

