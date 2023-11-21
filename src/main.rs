use log::*;
use tomasulos::*;

const CONFIG: Config = Config {
    eff_addr_buffer_entries: 2,
    fp_add_buffer_entries: 3,
    fp_mul_buffer_entries: 3,
    int_buffer_entries: 2,
    reorder_buffer_entries: 5,

    fp_add_buffer_latency: 2,
    fp_sub_buffer_latency: 2,
    fp_mul_buffer_latency: 5,
    fp_div_buffer_latency: 10,
};

#[allow(dead_code)]
fn parse_file(filename: &str) -> Vec<RiscVOp> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let mut instructions = Vec::new();
    for line in reader.lines() {
        let line = line.unwrap();
        let op = RiscVOp::parse(&line);
        instructions.push(op);
    }
    instructions
}

fn parse_stdin() -> Vec<RiscVOp> {
    use std::io::{BufRead, BufReader};

    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin);
    let mut instructions = Vec::new();
    for line in reader.lines() {
        let line = line.unwrap();
        let op = RiscVOp::parse(&line);
        instructions.push(op);
    }
    instructions
}

fn main() {
    env_logger::init();

    let config = Config::parse("config.txt").unwrap_or_else(|e| {
        error!("{}", e);
        std::process::exit(1);
    });
    info!("{}", config);

    let instructions = parse_stdin();
    for (i, op) in instructions.iter().enumerate() {
        info!("{}: {}", i, op);
    }

    let mut table = TomasuloTable::new();
    println!("{}", CONFIG);
    table.run(instructions, &config);
    println!("{}", table);
}
