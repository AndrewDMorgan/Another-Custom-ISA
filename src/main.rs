
fn get_macros(scripts: &mut Vec<Vec<String>>) -> Vec<(String, Vec<String>, Vec<Vec<String>>)> {
    let mut macros = vec![];
    let mut line_number = 0;
    while line_number < scripts.len() {
        if scripts[line_number][0] == "!macro" {
            // parsing the macro
            let name = scripts[line_number][1].clone();
            let args = scripts[line_number][2..].iter().map(|t| t.clone()).collect::<Vec<String>>();
            // finding the ending line
            let end_line = scripts[line_number + 1..]
                .iter()
                .position(|line| line[0] == "!end")
                .unwrap() + line_number + 1;
            macros.push((
                name,
                args,
                scripts[line_number + 1..end_line]
                    .iter()
                    .map(|line| 
                        line.iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>())
                    .collect::<Vec<Vec<String>>>()));
            scripts.drain(line_number..=end_line);
            continue;
        }
        line_number += 1;
    } macros
}

fn expand_macro_calls(lines: &mut Vec<Vec<String>>, macros: Vec<(String, Vec<String>, Vec<Vec<String>>)>) {
    let mut line_number = 0;
    while line_number < lines.len() {
        if macros.iter().any(|(m,..)| lines[line_number][0] == *m) {
            let mac = lines.remove(line_number);
            let (_name, args, body) = macros.iter().find(|(m,..)| m == &mac[0]).unwrap();
            for line in body {
                // replacing any args
                let new_line = line.iter().map(|t| {
                    if let Some(arg_index) = args.iter().position(|a| a == t) {
                        mac[arg_index + 1].to_string()
                    } else {
                        t.to_string()
                    }
                }).collect::<Vec<String>>();
                lines.insert(line_number, new_line);
                line_number += 1;
            }
            continue;
        }
        line_number += 1;
    }
}

// op_code/op_type combined, num args, name
static OP_CODES: &[(u8, usize, &str)] = &[
    (0b000_00000, 0, "Nop"),
    (0b000_00001, 0, "SetDspInX"),
    (0b000_00010, 0, "SetDspInY"),
    (0b000_00011, 0, "SetDspInCol"),
    (0b000_00100, 2, "Plot"),
    (0b000_00101, 2, "Color"),
    (0b000_00110, 0, "SetPtr"),
    (0b000_00111, 0, "PgcL"),
    (0b000_01000, 0, "PgcR"),
    (0b001_00000, 0, "Add"),
    (0b001_00001, 0, "Sub"),
    (0b001_00010, 0, "Inc"),
    (0b001_00011, 0, "Dec"),
    (0b001_00100, 0, "ThruL"),
    (0b001_00101, 0, "ThruR"),
    (0b001_00110, 0, "And"),
    (0b001_00111, 0, "Or"),
    (0b001_01000, 0, "ShftL"),
    (0b001_01001, 0, "ShftR"),
    (0b001_01010, 1, "LdiL"),
    (0b001_01011, 1, "LdiR"),
    (0b010_00000, 0, "Eq"),
    (0b010_00001, 0, "Less"),
    (0b010_00010, 0, "Grtr"),
    (0b010_00011, 0, "OvrFlow"),
    (0b010_00100, 0, "SetC"),
    (0b010_00101, 0, "RsetC"),
    (0b011_00000, 1, "LodL"),
    (0b011_00001, 1, "LodR"),
    (0b011_00010, 1, "WrtO"),
    (0b011_00011, 0, "PtrL"),
    (0b011_00100, 0, "PtrR"),
    (0b011_00101, 0, "PtrO"),
    (0b011_00110, 2, "Ldi"),
    (0b100_00000, 1, "RLodL"),
    (0b100_00001, 1, "RLodR"),
    (0b100_00010, 1, "RWrtO"),
    (0b100_00011, 0, "RPtrL"),
    (0b100_00100, 0, "RPtrR"),
    (0b100_00101, 0, "RPtrO"),
    (0b100_00110, 2, "RLdi"),
    (0b101_00000, 1, "DLodL"),
    (0b101_00001, 1, "DLodR"),
    (0b101_00010, 1, "DWrtO"),
    (0b101_00011, 0, "DPtrL"),
    (0b101_00100, 0, "DPtrR"),
    (0b101_00101, 0, "DPtrO"),
    (0b101_00110, 2, "DLdi"),
    (0b110_00000, 1, "Jmp"),
    (0b110_00001, 1, "Jiz"),
    (0b110_00010, 1, "Jnz"),
    (0b110_00011, 0, "JmpPtr"),
    (0b110_00100, 0, "JizPtr"),
    (0b110_00101, 0, "JnzPtr"),
    (0b110_00110, 1, "JmpR"),
    (0b111_00000, 0, "Pop"),
    (0b111_00001, 0, "TopL"),
    (0b111_00010, 0, "TopR"),
    (0b111_00011, 0, "PshO"),
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

fn compile_script(mut script: Vec<Vec<String>>) -> Vec<u32> {
    // calculating header indexes
    let mut true_index = 0;
    let mut headers = vec![];
    for (_index, line) in script.iter().enumerate() {
        // checking for a header defintion
        if ["!header", "!end", "!loop"].contains(&&*line[0]) {
            // getting the name
            headers.push((line[1].clone(), true_index));
            continue;  // this isn't a valid instruction and as such shouldn't be included in the bytecode
        }
        // invalid instruction, skipping (maybe a comment or something)
        if OP_CODES.iter().find(|(_, _, name)| name == &line[0]).is_none() {  continue; }
        true_index += 1;
    }
    for line in script.iter_mut() {
        for token in line.iter_mut() {
            if let Some(reg_index) = REGISTERS.iter().position(|r| r == token) {
                *token = (reg_index as u32).to_string();
            }
            if let Some(header_index) = headers.iter().position(|h| h.0 == *token) {
                *token = (headers[header_index].1).to_string();
            }
        }
    }
    println!("Final Tokens: {:?}", script);
    let mut bytecode = vec![];
    for line in script {
        // replacing any headers mentioned with their index
        let op = OP_CODES.iter().find(|(_, _, name)| name == &line[0]);
        if let Some(op) = op {
            let mut instruction = (op.0 as u32) << 24;
            for i in 0..op.1 {
                instruction |= (line[i + 1].parse::<u8>().unwrap() as u32) << (24 - 8 * i);
            }
            bytecode.push(instruction);
        }
    } bytecode
}

fn main() {
    let script = std::fs::read_to_string("../scripts/program_file.mca").unwrap();
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
    println!("Tokens: {:?}", script);
    // collecting all macros
    let macros = get_macros(&mut script);
    println!("Macros: {:?}", macros);
    println!("Tokens: {:?}", script);
    expand_macro_calls(&mut script, macros);
    println!("Tokens: {:?}", script);
    let bytes = compile_script(script);
    println!("{}", bytes.iter().enumerate()
        .map(|(index, byte)| format!("{:>3}: {}\n\n", index, format!("{:032b}", byte)
            .chars().map(|c| format!(" {} ", c)).collect::<String>()
        )).collect::<String>());
}
