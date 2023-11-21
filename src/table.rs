use super::*;
use log::*;
use std::fmt::{self, Display, Formatter};

#[derive(Default)]
pub struct TomasuloTable {
    rows: Vec<Row>,
}

impl TomasuloTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, instructions: Vec<RiscVOp>, config: &Config) {
        let mut reorder_buffer = ReorderBuffer::from(config);

        let mut i = 0;
        let mut cycle = 1;
        loop {
            if reorder_buffer.get_finished_instructions() >= instructions.len() {
                break;
            }

            if i < instructions.len() {
                let op = instructions[i];
                if reorder_buffer.add(op).is_ok() {
                    while self.rows.len() <= i {
                        self.rows.push(Row {
                            op: None,
                            issued: None,
                            start_ex: None,
                            end_ex: None,
                            mem_access: None,
                            write_back: None,
                            committed: None,
                        });
                    }
                    self.rows[i].issued = Some(cycle);
                    i += 1;
                } else {
                    trace!("Failed to add instruction {i}: {op}");
                }
            } else if reorder_buffer.get_finished_instructions() >= instructions.len() {
                info!("Stopped at instruction {}:", i);
                break;
            }
            let stages = reorder_buffer.get_stages();
            for (instruction_num, op, stage) in stages {
                while self.rows.len() <= instruction_num {
                    self.rows.push(Row {
                        op: None,
                        issued: None,
                        start_ex: None,
                        end_ex: None,
                        mem_access: None,
                        write_back: None,
                        committed: None,
                    });
                }

                self.rows[instruction_num].op = Some(op);
                match stage {
                    Stage::Execute(1) if self.rows[instruction_num].start_ex.is_none() => {
                        self.rows[instruction_num].start_ex = Some(cycle);
                        self.rows[instruction_num].end_ex = Some(cycle);
                    }
                    Stage::Execute(1) if self.rows[instruction_num].start_ex.is_some() => {
                        self.rows[instruction_num].end_ex = Some(cycle)
                    }
                    Stage::Execute(_) if self.rows[instruction_num].start_ex.is_none() => {
                        self.rows[instruction_num].start_ex = Some(cycle)
                    }
                    Stage::MemAccess => self.rows[instruction_num].mem_access = Some(cycle),
                    Stage::WriteBack => self.rows[instruction_num].write_back = Some(cycle),
                    Stage::Commit => self.rows[instruction_num].committed = Some(cycle),
                    _ => {}
                }
            }
            reorder_buffer.tick(config);
            cycle += 1;
            trace!("Cycle {}\n\n{}", cycle, reorder_buffer);

            let stages = reorder_buffer.get_stages();

            for (instruction_num, op, stage) in stages {
                while self.rows.len() <= instruction_num {
                    self.rows.push(Row {
                        op: None,
                        issued: None,
                        start_ex: None,
                        end_ex: None,
                        mem_access: None,
                        write_back: None,
                        committed: None,
                    });
                }

                self.rows[instruction_num].op = Some(op);
                match stage {
                    Stage::Execute(1) if self.rows[instruction_num].start_ex.is_none() => {
                        self.rows[instruction_num].start_ex = Some(cycle);
                        self.rows[instruction_num].end_ex = Some(cycle);
                    }
                    Stage::Execute(1) if self.rows[instruction_num].start_ex.is_some() => {
                        self.rows[instruction_num].end_ex = Some(cycle)
                    }
                    Stage::Execute(_) if self.rows[instruction_num].start_ex.is_none() => {
                        self.rows[instruction_num].start_ex = Some(cycle)
                    }
                    Stage::MemAccess => self.rows[instruction_num].mem_access = Some(cycle),
                    Stage::WriteBack => self.rows[instruction_num].write_back = Some(cycle),
                    Stage::Commit => self.rows[instruction_num].committed = Some(cycle),
                    _ => {}
                }
            }
        }
    }
}

impl Display for TomasuloTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "                    Pipeline Simulation\n-----------------------------------------------------------")?;
        writeln!(f, "                                      Memory Writes\n     Instruction      Issues Executes  Read  Result Commits\n--------------------- ------ -------- ------ ------ -------")?;
        for row in &self.rows {
            writeln!(f, "{}", row)?;
        }
        Ok(())
    }
}

struct Row {
    op: Option<RiscVOp>,
    issued: Option<u64>,
    start_ex: Option<u64>,
    end_ex: Option<u64>,
    mem_access: Option<u64>,
    write_back: Option<u64>,
    committed: Option<u64>,
}

impl Display for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(op) = &self.op {
            write!(f, "{:<22}", format!("{op}"))?;
        } else {
            write!(f, "{:22}", "?")?;
        }

        if let Some(issued) = &self.issued {
            write!(f, "{:>6}", issued)?;
        } else {
            write!(f, "{:>6}", "?")?;
        }

        if let Some(start_ex) = &self.start_ex {
            write!(f, "{:>4}", start_ex)?;
        } else {
            write!(f, "{:>4}", "?")?;
        }

        write!(f, " -")?;

        if let Some(end_ex) = &self.end_ex {
            write!(f, "{:>3}", end_ex)?;
        } else {
            write!(f, "{:>3}", "?")?;
        }

        if let Some(mem_access) = &self.mem_access {
            write!(f, "{:>7}", mem_access)?;
        } else if let Some(op) = self.op {
            if op.accesses_memory() {
                write!(f, "{:>7}", "?")?;
            } else {
                write!(f, "{:>7}", "")?;
            }
        } else {
            write!(f, "{:>7}", "?")?;
        }

        if let Some(write_back) = &self.write_back {
            write!(f, "{:>7}", write_back)?;
        } else if let Some(op) = self.op {
            if op.writes_back() {
                write!(f, "{:>7}", "?")?;
            } else {
                write!(f, "{:>7}", "")?;
            }
        } else {
            write!(f, "{:>7}", "?")?;
        }

        if let Some(committed) = &self.committed {
            write!(f, "{:>8}", committed)?;
        } else {
            write!(f, "{:>8}", "?")?;
        }

        Ok(())
    }
}
