use std::collections::HashMap;
use std::env;
use std::fs;

const MAGIC: &[u8; 4] = b"STAT";
const VERSION: u16 = 1;
const HEADER_SIZE: usize = 32;

fn register(s: &str) -> Result<u8, String> {
  let s = s.trim().trim_end_matches(',');

  if !s.starts_with('R') && !s.starts_with('r') {
    return Err(format!("invalid register {}", s));
  }

  let value = s[1..].parse::<u8>().map_err(|_| format!("invalid register {}", s))?;

  if value > 15 {
    return Err(format!("register out of range {}", s));
  }

  Ok(value)
}

fn number(s: &str) -> Result<u32, String> {
  let s = s.trim().trim_end_matches(',');

  if let Some(v) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
    u32::from_str_radix(v, 16).map_err(|_| format!("invalid number {}", s))
  } else if let Some(v) = s.strip_prefix('-') {
    let value = v.parse::<i64>().map_err(|_| format!("invalid number {}", s))?;
    Ok(value as u32)
  } else {
    s.parse::<u32>().map_err(|_| format!("invalid number {}", s))
  }
}

fn tokens(line: &str) -> Vec<String> {
  line.replace(',', " ").split_whitespace().map(str::to_string).collect()
}

fn size(t: &[String]) -> Result<usize, String> {
  match t[0].to_uppercase().as_str() {
    "HALT" | "NOP" | "WFI" | "ERET" | "BREAK" | "RET" | "MRET" | "SRET" | "IRET" => Ok(1),

    "ADDI" | "SUBI" | "ANDI" | "ORI" | "XORI" | "MULI" | "SLTI" | "SLTIU" => Ok(7),

    | "ADD"
    | "SUB"
    | "MUL"
    | "DIV"
    | "DIVU"
    | "REM"
    | "REMU"
    | "MIN"
    | "MAX"
    | "AND"
    | "OR"
    | "XOR"
    | "SHL"
    | "SHR"
    | "SAR"
    | "ROL"
    | "ROR" => Ok(4),

    "CMP" | "NOT" | "CLZ" | "CTZ" | "POPCNT" | "LRW" | "SCW" | "PUSH" | "POP" | "SETMODE" => Ok(3),

    "LB" | "LBU" | "LH" | "LHU" | "LW" | "SB" | "SH" | "SW" => Ok(7),

    | "JMP"
    | "JZ"
    | "JNZ"
    | "JC"
    | "JN"
    | "JO"
    | "JGE"
    | "JL"
    | "JGT"
    | "JLE"
    | "JCS"
    | "JCC"
    | "CALL"
    | "INT" => Ok(5),

    "CSRR" | "CSRW" | "CSRS" | "CSRC" => Ok(4),

    _ => Err(format!("unknown instruction {}", t[0])),
  }
}

fn resolve(s: &str, labels: &HashMap<String, u32>) -> Result<u32, String> {
  if let Some(v) = labels.get(s) { Ok(*v) } else { number(s) }
}

fn emit_jump(op: u8, arg: &str, labels: &HashMap<String, u32>) -> Result<Vec<u8>, String> {
  let value = resolve(arg, labels)?;
  let mut out = vec![op];
  out.extend_from_slice(&value.to_le_bytes());
  Ok(out)
}

fn emit(t: &[String], labels: &HashMap<String, u32>) -> Result<Vec<u8>, String> {
  let op = t[0].to_uppercase();

  let r = |n: usize| register(t[n].as_str());
  let n = |x: &str| resolve(x, labels);

  let tri = |opcode: u8| { Ok(vec![opcode, r(1)?, r(2)?, r(3)?]) };

  let imm = |opcode: u8| {
    let mut v = vec![opcode, r(1)?, r(2)?];
    v.extend_from_slice(&n(&t[3])?.to_le_bytes());
    Ok(v)
  };

  let mem = |opcode: u8| {
    let mut v = vec![opcode, r(1)?, r(2)?];
    v.extend_from_slice(&n(&t[3])?.to_le_bytes());
    Ok(v)
  };

  match op.as_str() {
    "HALT" => Ok(vec![0x00]),
    "NOP" => Ok(vec![0x01]),
    "WFI" => Ok(vec![0x02]),
    "ERET" => Ok(vec![0x03]),
    "BREAK" => Ok(vec![0x04]),

    "ADDI" => imm(0x10),
    "SUBI" => imm(0x11),
    "ANDI" => imm(0x12),
    "ORI" => imm(0x13),
    "XORI" => imm(0x14),
    "MULI" => imm(0x15),
    "SLTI" => imm(0x16),
    "SLTIU" => imm(0x17),

    "ADD" => tri(0x20),
    "SUB" => tri(0x21),
    "MUL" => tri(0x22),
    "DIV" => tri(0x23),
    "DIVU" => tri(0x24),
    "REM" => tri(0x25),
    "REMU" => tri(0x26),
    "MIN" => tri(0x28),
    "MAX" => tri(0x29),

    "CMP" => Ok(vec![0x27, r(1)?, r(2)?]),
    "AND" => tri(0x30),
    "OR" => tri(0x31),
    "XOR" => tri(0x32),
    "NOT" => Ok(vec![0x33, r(1)?, r(2)?]),
    "CLZ" => Ok(vec![0x34, r(1)?, r(2)?]),
    "CTZ" => Ok(vec![0x35, r(1)?, r(2)?]),
    "POPCNT" => Ok(vec![0x36, r(1)?, r(2)?]),

    "SHL" => tri(0x40),
    "SHR" => tri(0x41),
    "SAR" => tri(0x42),
    "ROL" => tri(0x43),
    "ROR" => tri(0x44),

    "LB" => mem(0x50),
    "LBU" => mem(0x51),
    "LH" => mem(0x52),
    "LHU" => mem(0x53),
    "LW" => mem(0x54),
    "SB" => mem(0x55),
    "SH" => mem(0x56),
    "SW" => mem(0x57),

    "LRW" => Ok(vec![0x58, r(1)?, r(2)?]),
    "SCW" => Ok(vec![0x59, r(1)?, r(2)?]),

    "JMP" => emit_jump(0x60, &t[1], labels),
    "JZ" => emit_jump(0x61, &t[1], labels),
    "JNZ" => emit_jump(0x62, &t[1], labels),
    "JC" => emit_jump(0x63, &t[1], labels),
    "JN" => emit_jump(0x64, &t[1], labels),
    "JO" => emit_jump(0x65, &t[1], labels),
    "JGE" => emit_jump(0x66, &t[1], labels),
    "JL" => emit_jump(0x67, &t[1], labels),
    "JGT" => emit_jump(0x68, &t[1], labels),
    "JLE" => emit_jump(0x69, &t[1], labels),
    "JCS" => emit_jump(0x6a, &t[1], labels),
    "JCC" => emit_jump(0x6b, &t[1], labels),

    "PUSH" => Ok(vec![0x70, r(1)?, 0]),
    "POP" => Ok(vec![0x71, r(1)?, 0]),

    "CALL" => emit_jump(0x72, &t[1], labels),
    "RET" => Ok(vec![0x73]),

    "CSRR" => {
      let mut v = vec![0x90, r(1)?];
      v.extend_from_slice(&number(&t[2])?.to_le_bytes()[..2]);
      Ok(v)
    }

    "CSRW" => {
      let mut v = vec![0x91, r(1)?];
      v.extend_from_slice(&number(&t[2])?.to_le_bytes()[..2]);
      Ok(v)
    }

    "CSRS" => {
      let mut v = vec![0x92, r(1)?];
      v.extend_from_slice(&number(&t[2])?.to_le_bytes()[..2]);
      Ok(v)
    }

    "CSRC" => {
      let mut v = vec![0x93, r(1)?];
      v.extend_from_slice(&number(&t[2])?.to_le_bytes()[..2]);
      Ok(v)
    }

    "MRET" => Ok(vec![0xa0]),
    "SRET" => Ok(vec![0xa1]),
    "SETMODE" => Ok(vec![0xa2, r(1)?, 0]),
    "INT" => emit_jump(0xb0, &t[1], labels),
    "IRET" => Ok(vec![0xb1]),

    _ => Err(format!("unknown instruction {}", t[0])),
  }
}

fn data(t: &[String]) -> Result<Vec<u8>, String> {
  match t[0].to_lowercase().as_str() {
    ".byte" =>
      t[1..]
        .iter()
        .map(|x| number(x).map(|v| v as u8))
        .collect(),

    ".half" => {
      let mut out = Vec::new();
      for x in &t[1..] {
        out.extend_from_slice(&(number(x)? as u16).to_le_bytes());
      }
      Ok(out)
    }

    ".word" => {
      let mut out = Vec::new();
      for x in &t[1..] {
        out.extend_from_slice(&number(x)?.to_le_bytes());
      }
      Ok(out)
    }

    ".ascii" => {
      let text = t[1..].join(" ");
      Ok(text.trim_matches('"').as_bytes().to_vec())
    }

    _ => Err(format!("unknown directive {}", t[0])),
  }
}

fn main() -> Result<(), String> {
  let input = env
    ::args()
    .nth(1)
    .unwrap_or_else(|| "program.sasm".into());

  let output_path = env
    ::args()
    .nth(2)
    .unwrap_or_else(|| "program.status".into());

  let source = fs::read_to_string(&input).map_err(|e| e.to_string())?;

  let lines: Vec<Vec<String>> = source
    .lines()
    .map(|line| line.split(';').next().unwrap_or(""))
    .map(tokens)
    .filter(|x| !x.is_empty())
    .collect();

  let mut code = Vec::new();
  let mut data_lines = Vec::new();
  let mut labels = HashMap::new();
  let mut in_data = false;
  let mut pc = 0u32;

  for line in &lines {
    if line[0].eq_ignore_ascii_case(".data") {
      in_data = true;
      continue;
    }

    if in_data {
      data_lines.push(line.clone());
      continue;
    }

    let mut start = 0;

    if line[0].ends_with(':') {
      let name = line[0].trim_end_matches(':').to_string();
      labels.insert(name, pc);
      start = 1;
    }

    if start < line.len() {
      pc += size(&line[start..])? as u32;
    }
  }

  let code_size = pc;

  let mut data_bytes = Vec::new();
  let mut data_offset = code_size;

  for line in &data_lines {
    let mut start = 0;

    if line[0].ends_with(':') {
      let name = line[0].trim_end_matches(':').to_string();
      labels.insert(name, data_offset);
      start = 1;
    }

    if start < line.len() {
      let bytes = data(&line[start..])?;
      data_offset += bytes.len() as u32;
      data_bytes.extend(bytes);
    }
  }

  for line in &lines {
    if line[0].eq_ignore_ascii_case(".data") {
      break;
    }

    let mut start = 0;

    if line[0].ends_with(':') {
      start = 1;
    }

    if start < line.len() {
      code.extend(emit(&line[start..], &labels)?);
    }
  }

  let code_offset = HEADER_SIZE as u32;
  let data_offset = code_offset + (code.len() as u32);

  let mut output = Vec::new();

  output.extend_from_slice(MAGIC);
  output.extend_from_slice(&VERSION.to_le_bytes());
  output.extend_from_slice(&(0u16).to_le_bytes());
  output.extend_from_slice(&(0u32).to_le_bytes());
  output.extend_from_slice(&code_offset.to_le_bytes());
  output.extend_from_slice(&(code.len() as u32).to_le_bytes());
  output.extend_from_slice(&data_offset.to_le_bytes());
  output.extend_from_slice(&(data_bytes.len() as u32).to_le_bytes());
  output.extend_from_slice(&(0u32).to_le_bytes());
  output.extend_from_slice(&code);
  output.extend_from_slice(&data_bytes);

  fs::write(&output_path, output).map_err(|e| e.to_string())?;

  println!("StatusCPU assembler");
  println!("Code: {} bytes", code.len());
  println!("Data: {} bytes", data_bytes.len());
  println!("Output: {}", output_path);

  Ok(())
}
