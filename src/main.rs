use tomasulos::*;
use log::*;

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

fn main() {
    env_logger::init();

    let mut reorder_buffer = ReorderBuffer::from(&CONFIG);
//     // let mut stations = ReservationStations::new(&CONFIG);
    // reorder_buffer.add(RiscVOp::parse("flw f6,32(x2):0")).unwrap();
    // info!("{}", reorder_buffer);
    // reorder_buffer.tick(&CONFIG);
    // reorder_buffer.add(RiscVOp::parse("flw f2,48(x3):4")).unwrap();
    // info!("{}", reorder_buffer);
    // reorder_buffer.tick(&CONFIG);
    // reorder_buffer.add(RiscVOp::parse("fmul f0,f2,f4")).unwrap();
    // info!("{}", reorder_buffer);
    // reorder_buffer.tick(&CONFIG);
    // reorder_buffer.add(RiscVOp::parse("fsub f8,f6,f2")).unwrap();
    // info!("{}", reorder_buffer);
    // reorder_buffer.tick(&CONFIG);
    // reorder_buffer.add(RiscVOp::parse("fdiv f10,f0,f6")).unwrap();
    // info!("{}", reorder_buffer);
    // reorder_buffer.tick(&CONFIG);
    // reorder_buffer.add(RiscVOp::parse("fadd f6,f8,f2")).unwrap();
    // info!("{}", reorder_buffer);
    // reorder_buffer.tick(&CONFIG);
    // info!("{}", reorder_buffer);
    // reorder_buffer.tick(&CONFIG);
    // info!("{}", reorder_buffer);
    // reorder_buffer.tick(&CONFIG);

    let instructions = vec![
        RiscVOp::parse("flw f6,32(x2):0"),
        RiscVOp::parse("flw f2,48(x3):4"),
        RiscVOp::parse("fmul f0,f2,f4"),
        RiscVOp::parse("fsub f8,f6,f2"),
        RiscVOp::parse("fdiv f10,f0,f6"),
        RiscVOp::parse("fadd f6,f8,f2"),
        RiscVOp::parse("fdiv f13,f10,f6"),
    ];

    let mut table = TomasuloTable::new();
    println!("{}", CONFIG);
    table.run(instructions, &CONFIG);
    println!("{}", table);

    // let mut i = 0;
    // let mut cycle = 0;
    // loop {
    //     if cycle > 30 {
    //         break;
    //     }
    //     if reorder_buffer.get_finished_instructions() >= instructions.len() {
    //         break;
    //     }
    //     if i < instructions.len() {
    //         let op = instructions[i].clone();
    //         if reorder_buffer.add(op).is_ok() {
    //             i += 1;
    //         }
    //     }
    //     cycle += 1;
    //     info!("Cycle {}\n\n{}", cycle, reorder_buffer);
    //     reorder_buffer.tick(&CONFIG);
    // }

// flw    f2,48(x3):4
// fmul.s f0,f2,f4
// fsub.s f8,f6,f2
// fdiv.s f10,f0,f6
// fadd.s f6,f8,f2
//     // reorder_buffer.tick(0, &CONFIG);
//     info!("Cycle 1\n\n{}", reorder_buffer);
//     reorder_buffer.add(RiscVOp::parse("flw f2,48(x3):4"), 1);
//     info!("Cycle 2\n\n{}", reorder_buffer);
//     reorder_buffer.add(RiscVOp::parse("fmul f0,f2,f4"), 2);
//     info!("Cycle 3\n\n{}", reorder_buffer);
//     reorder_buffer.add(RiscVOp::parse("fsub f8,f6,f2"), 3);
    // info!("{}", reorder_buffer);
}
