use super::*;
use log::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Display, Formatter},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Issue,
    Execute(u64), // u64 is the cycles left to execute
    MemAccess,
    WriteBack,
    WaitingToCommit,
    Commit,
}

pub struct ReorderBuffer {
    register_mapping: BTreeMap<Register, u64>,
    addresses_stored: BTreeSet<u64>,
    addresses_loaded: BTreeSet<u64>,

    available_reservation_stations: BTreeMap<FunctionalUnit, usize>,

    // The instructions in the reorder buffer are stored in a circular buffer.
    // Entries must be in order, but the head and tail can be anywhere.
    entries: Vec<Option<(usize, RiscVOp, Stage)>>,
    issue_count: usize,

    head: usize,
    tail: usize,
    size: usize,
    entries_used: usize,
    entries_committed: usize,
}

impl From<&Config> for ReorderBuffer {
    fn from(config: &Config) -> Self {
        Self::new(config)
    }
}

impl ReorderBuffer {
    pub fn new(config: &Config) -> Self {
        let size = config.reorder_buffer_entries as usize;
        let mut entries = Vec::with_capacity(size);
        entries.resize_with(size, || None);

        let mut available_reservation_stations = BTreeMap::new();
        available_reservation_stations
            .insert(FunctionalUnit::ALU, config.int_buffer_entries as usize);
        available_reservation_stations.insert(
            FunctionalUnit::FPUAdd,
            config.fp_add_buffer_entries as usize,
        );
        available_reservation_stations.insert(
            FunctionalUnit::FPUMul,
            config.fp_mul_buffer_entries as usize,
        );
        available_reservation_stations.insert(
            FunctionalUnit::EffectAddr,
            config.eff_addr_buffer_entries as usize,
        );

        Self {
            register_mapping: BTreeMap::new(),
            addresses_loaded: BTreeSet::new(),
            addresses_stored: BTreeSet::new(),
            available_reservation_stations,

            entries,
            issue_count: 0,
            head: 0,
            tail: 0,
            size,
            entries_used: 0,
            entries_committed: 0,
        }
    }

    /// Return a list of all the issued instructions in order,
    /// with the stages they're in
    pub fn get_stages(&self) -> Vec<(usize, RiscVOp, Stage)> {
        let mut result = Vec::new();
        for i in self.tail..self.tail + self.entries_used {
            if let Some((issued, op, s)) = &self.entries[i % self.size] {
                result.push((*issued, *op, *s));
            }
        }
        result
    }

    pub fn add(&mut self, op: RiscVOp) -> Result<(), ()> {
        if self.entries_used >= self.size {
            return Err(());
        }

        // Check if the reservation station is available
        if let Some(available) = self
            .available_reservation_stations
            .get_mut(&op.functional_unit())
        {
            if *available == 0 {
                return Err(());
            }
        } else {
            return Err(());
        }

        if let Some(addr) = op.addr() {
            if self.addresses_loaded.contains(&addr) {
                return Err(());
            }
            if self.addresses_stored.contains(&addr) {
                return Err(());
            }
        }

        // Get the reservation station for the op
        debug!("Adding {} to the reservation station", op);
        self.available_reservation_stations
            .entry(op.functional_unit())
            .and_modify(|e| *e -= 1);

        // Add the register mapping
        if let Some(dst) = op.dst() {
            debug!("Adding {} to the register mapping", dst);
            let dst_reg = dst.as_reg();
            self.register_mapping.insert(dst_reg, self.head as u64);
        }
        self.entries[self.head] = Some((self.issue_count, op, Stage::Issue));
        self.head = self.head.wrapping_add(1) % self.size;
        self.entries_used += 1;
        self.issue_count += 1;
        Ok(())
    }

    pub fn get_all_in_stage(&self, stage: Stage) -> Vec<(usize, RiscVOp)> {
        let mut result = Vec::new();
        for i in self.head..self.head + self.size {
            if let Some((_, op, s)) = &self.entries[i % self.size] {
                if s == &stage {
                    result.push((i % self.size, *op));
                }
            }
        }
        result
    }

    pub fn get_all_in_ex(&self) -> Vec<(usize, RiscVOp)> {
        let mut result = Vec::new();

        for i in self.head..self.head + self.size {
            if let Some((_, op, s)) = &self.entries[i % self.size] {
                if let Stage::Execute(_) = s {
                    result.push((i % self.size, *op));
                }
            }
        }
        result
    }

    pub fn write_to_cdb(&mut self, i: usize) -> bool {
        if let Some((_, _op, s)) = &self.entries[i] {
            if s == &Stage::WriteBack {
                // Confirm all the operations before this one are committed
                let mut all_committed = true;
                for j in self.tail..self.tail + self.size {
                    if i == j {
                        break;
                    }
                    if let Some((_, _, s)) = &self.entries[j % self.size] {
                        if s != &Stage::Commit {
                            all_committed = false;
                            break;
                        }
                    }
                }
                if all_committed {
                    self.entries[i].as_mut().unwrap().2 = Stage::Commit;
                } else {
                    self.entries[i].as_mut().unwrap().2 = Stage::WaitingToCommit;
                }
                // Free up the reservation station
                return true;
            }
        }
        false
    }

    pub fn get_finished_instructions(&self) -> usize {
        self.entries_committed
    }

    /// Is instruction i earlier than instruction j?
    pub fn is_earlier_than(&self, i: usize, j: usize) -> bool {
        if i == j {
            return false;
        }
        if i < self.tail && j < self.tail {
            return i < j;
        }
        if i >= self.tail && j >= self.tail {
            return i < j;
        }
        if i < self.tail && j >= self.tail {
            return true;
        }
        if i >= self.tail && j < self.tail {
            return false;
        }
        false
    }

    pub fn tick(&mut self, config: &Config) {
        let mut already_committed = false;

        // Check the commit stage
        // self.available_reservation_stations.entry(op.functional_unit()).and_modify(|e| *e += 1);
        self.get_all_in_stage(Stage::Commit)
            .iter()
            .for_each(|(i, op)| {
                if let Some(addr) = op.addr() {
                    if op.is_load() {
                        self.addresses_loaded.remove(&addr);
                    } else {
                        self.addresses_stored.remove(&addr);
                    }
                }
                self.entries_committed += 1;
                self.entries[*i] = None;
                self.tail = self.tail.wrapping_add(1) % self.size;
                self.entries_used -= 1;
            });

        // Check the commit stage
        self.get_all_in_stage(Stage::WaitingToCommit)
            .iter()
            .for_each(|(i, _op)| {
                // Check if all the instructions before this one are committed
                let mut all_committed = true;
                for j in self.tail..self.tail + self.size {
                    if *i == j {
                        break;
                    }
                    if let Some((_, _, s)) = &self.entries[j % self.size] {
                        if s != &Stage::Commit {
                            all_committed = false;
                            break;
                        }
                    }
                }
                if all_committed && !already_committed {
                    // if let Some(addr) = op.addr() {
                    //     self.addresses_in_use.remove(&addr);
                    // }
                    self.entries[*i].as_mut().unwrap().2 = Stage::Commit;
                    already_committed = true;
                }
            });

        // Check the WB stage
        let mut removed_registers = Vec::new();
        let mut wrote_to_cdb = false;

        self.get_all_in_stage(Stage::WriteBack)
            .iter()
            .for_each(|(i, op)| {
                if wrote_to_cdb {
                    return;
                }
                // Check if all the instructions before this one are committed
                let mut all_committed = true;
                for j in self.tail..self.tail + self.size {
                    if *i == j {
                        break;
                    }
                    if let Some((_, _, s)) = &self.entries[j % self.size] {
                        if s != &Stage::Commit {
                            all_committed = false;
                            break;
                        }
                    }
                }
                if all_committed && !already_committed {
                    // if let Some(addr) = op.addr() {
                    //     self.addresses_in_use.remove(&addr);
                    // }
                    self.entries[*i].as_mut().unwrap().2 = Stage::Commit;
                    already_committed = true;
                } else {
                    self.entries[*i].as_mut().unwrap().2 = Stage::WaitingToCommit;
                }

                if let Some(dst) = op.dst() {
                    debug!("Removing {} from the register mapping", dst);
                    let dst_reg = dst.as_reg();
                    self.register_mapping.remove(&dst_reg);
                    removed_registers.push(dst_reg);
                }
                wrote_to_cdb = true;
            });

        // Check the MEM stage
        // Get the first issued instruction thats in the MEM stage.
        // If it's a load, check if the address is ready.
        // If it is, then write the result to the CDB.
        let mut already_accessed = false;
        self.get_all_in_stage(Stage::MemAccess)
            .iter()
            .for_each(|(i, op)| {
                if let Some(addr) = op.addr() {
                    if self.addresses_stored.contains(&addr) {
                        return;
                    }
                    if op.is_load() {
                        self.addresses_loaded.insert(addr);
                    } else {
                        self.addresses_stored.insert(addr);
                    }
                }
                if !already_accessed {
                    self.entries[*i].as_mut().unwrap().2 = Stage::WriteBack;
                    already_accessed = true;
                }
            });

        self.get_all_in_stage(Stage::WriteBack)
            .iter()
            .for_each(|(_i, op)| {
                // Only write to the CDB if we haven't already
                if wrote_to_cdb {
                    return;
                }

                // Write the result to the CDB
                if op.addr().is_none() {
                    if let Some(dst) = op.dst() {
                        debug!("Removing {} from the register mapping", dst);
                        let dst_reg = dst.as_reg();
                        self.register_mapping.remove(&dst_reg);
                        removed_registers.push(dst_reg);
                    }
                }
            });

        // Check the EX stage
        // Go through and decrement the cycles left to execute.
        // If it's 0, then move it to the MEM stage.
        let _wrote_back = false;
        self.get_all_in_ex().iter().for_each(|(i, op)| {
            if let Stage::Execute(cycles) = &mut self.entries[*i].as_mut().unwrap().2 {
                if *cycles > 0 {
                    *cycles -= 1;
                }
                if *cycles <= 0 {
                    // if *cycles <= 0 && !wrote_back {
                    // wrote_back = true;
                    if op.accesses_memory() {
                        self.entries[*i].as_mut().unwrap().2 = Stage::MemAccess;
                        trace!("Freeing up reservation station for {}", op);
                        self.available_reservation_stations
                            .entry(op.functional_unit())
                            .and_modify(|e| *e += 1);
                    } else if op.writes_back() {
                        if !wrote_to_cdb {
                            self.entries[*i].as_mut().unwrap().2 = Stage::WriteBack;
                            trace!("Freeing up reservation station for {}", op);
                            self.available_reservation_stations
                                .entry(op.functional_unit())
                                .and_modify(|e| *e += 1);
                        }
                    } else {
                        // Confirm all the operations before this one are committed
                        let mut all_committed = true;
                        for j in self.tail..self.tail + self.size {
                            if *i == j {
                                break;
                            }
                            if let Some((_, _op, s)) = &self.entries[j % self.size] {
                                if s != &Stage::Commit {
                                    all_committed = false;
                                    break;
                                }
                            }
                        }
                        if all_committed && !already_committed {
                            // if let Some(addr) = op.addr() {
                            //     self.addresses_in_use.remove(&addr);
                            // }
                            self.entries[*i].as_mut().unwrap().2 = Stage::Commit;
                            already_committed = true;
                        } else {
                            self.entries[*i].as_mut().unwrap().2 = Stage::WaitingToCommit;
                        }
                        trace!("Freeing up reservation station for {}", op);
                        self.available_reservation_stations
                            .entry(op.functional_unit())
                            .and_modify(|e| *e += 1);
                    }
                }
            }
        });
        // Check the issue stage
        self.get_all_in_stage(Stage::Issue)
            .iter()
            .for_each(|(i, op)| {
                // Check if any of the source registers are in the register mapping
                if let Some(src1) = op.src1().dep_reg() {
                    // Check if the source register is the destination of this instruction
                    if let Some(dst) = op.dst() {
                        if src1 != dst.as_reg() && self.register_mapping.contains_key(&src1) {
                            return;
                        }
                    }
                }

                if let Some(src2) = op.src2().dep_reg() {
                    // Check if the source register is the destination of this instruction
                    if let Some(dst) = op.dst() {
                        if src2 != dst.as_reg() && self.register_mapping.contains_key(&src2) {
                            return;
                        }
                    }
                }

                // Move the instruction to the EX stage
                self.entries[*i].as_mut().unwrap().2 = if op.is_fp_div() {
                    Stage::Execute(config.fp_div_buffer_latency)
                } else if op.is_fp_mul() {
                    Stage::Execute(config.fp_mul_buffer_latency)
                } else if op.is_fp_add() {
                    Stage::Execute(config.fp_add_buffer_latency)
                } else if op.is_branch() {
                    Stage::Execute(1)
                } else if op.is_alu() {
                    Stage::Execute(1)
                } else {
                    Stage::Execute(1)
                }
            });

        self.get_all_in_stage(Stage::WriteBack)
            .iter()
            .for_each(|(_i, op)| {
                // Only write to the CDB if we haven't already
                if wrote_to_cdb {
                    return;
                }

                // Write the result to the CDB
                if let Some(dst) = op.dst() {
                    debug!("Removing {} from the register mapping", dst);
                    let dst_reg = dst.as_reg();
                    self.register_mapping.remove(&dst_reg);
                    removed_registers.push(dst_reg);
                }
            });
    }
}

impl Display for ReorderBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Reorder Buffer:")?;
        writeln!(f, "  Register mapping:")?;
        for (reg, i) in &self.register_mapping {
            writeln!(f, "    {} -> {}", reg, i)?;
        }

        writeln!(f, "  Available Reservation stations:")?;
        for (fu, i) in &self.available_reservation_stations {
            writeln!(f, "    {:?} -> {}", fu, i)?;
        }

        writeln!(f, "  Addresses stored:")?;
        for addr in &self.addresses_stored {
            writeln!(f, "    {}", addr)?;
        }
        writeln!(f, "  Addresses loaded:")?;
        for addr in &self.addresses_loaded {
            writeln!(f, "    {}", addr)?;
        }

        writeln!(f, "  Head: {}", self.head)?;
        writeln!(f, "  Tail: {}", self.tail)?;
        writeln!(f, "  Entries used: {}", self.entries_used)?;
        writeln!(f, "  Entries committed: {}", self.entries_committed)?;
        writeln!(f, "  Entries:")?;
        for (i, entry) in self.entries.iter().enumerate() {
            if let Some((issued, op, s)) = entry {
                writeln!(f, "    #{}) {} ({:?}) issued on {issued}", i, op, s)?;
            } else {
                writeln!(f, "    #{}) None", i)?;
            }
        }
        Ok(())
    }
}
