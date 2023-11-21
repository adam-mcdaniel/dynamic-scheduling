use std::fmt::{self, Display, Formatter};
use log::*;

pub struct Config {
    pub eff_addr_buffer_entries: u64,
    pub fp_add_buffer_entries: u64,
    pub fp_mul_buffer_entries: u64,
    pub int_buffer_entries: u64,
    pub reorder_buffer_entries: u64,

    pub fp_add_buffer_latency: u64,
    pub fp_sub_buffer_latency: u64,
    pub fp_mul_buffer_latency: u64,
    pub fp_div_buffer_latency: u64,
}

impl Config {
    pub fn parse(filename: &str) -> Result<Self, String> {
        // buffers
        // 
        // eff addr: 2
        // fp adds: 3
        // fp muls: 3
        // ints: 2
        // reorder: 5
        //
        // latencies
        //
        // fp_add: 2
        // fp_sub: 2
        // fp_mul: 5
        // fp_div: 10

        // Read the file
        let contents = std::fs::read_to_string(filename).map_err(|e| {
            format!("Failed to read {}: {}", filename, e)
        })?;
        info!("Contents of {}: {}", filename, contents);

        let mut eff_addr_buffer_entries = 0;
        let mut fp_add_buffer_entries = 0;
        let mut fp_mul_buffer_entries = 0;
        let mut int_buffer_entries = 0;
        let mut reorder_buffer_entries = 0;
        let mut fp_add_buffer_latency = 0;
        let mut fp_sub_buffer_latency = 0;
        let mut fp_mul_buffer_latency = 0;
        let mut fp_div_buffer_latency = 0;

        let mut valid_count = 0;
        for line in contents.lines() {
            let mut parts = line.split(":");
            if parts.clone().count() != 2 {
                debug!("Skipping line: {}", line);
                continue;
            } else {
                valid_count += 1;
            }
            let name = parts.next().unwrap().trim();
            let value = parts.next().unwrap().trim().parse::<u64>().unwrap();
            debug!("{}: {}", name, value);
            match name {
                "eff addr" => eff_addr_buffer_entries = value,
                "fp adds" => fp_add_buffer_entries = value,
                "fp muls" => fp_mul_buffer_entries = value,
                "ints" => int_buffer_entries = value,
                "reorder" => reorder_buffer_entries = value,
                "fp_add" => fp_add_buffer_latency = value,
                "fp_sub" => fp_sub_buffer_latency = value,
                "fp_mul" => fp_mul_buffer_latency = value,
                "fp_div" => fp_div_buffer_latency = value,
                _ => return Err(format!("Unknown config parameter: {}", name)),
            }
        }
        if valid_count != 9 {
            return Err(format!("Expected 9 config parameters, found {}", valid_count));
        }
        

        let result = Self {
            eff_addr_buffer_entries,
            fp_add_buffer_entries,
            fp_mul_buffer_entries,
            int_buffer_entries,
            reorder_buffer_entries,
        
            fp_add_buffer_latency,
            fp_sub_buffer_latency,
            fp_mul_buffer_latency,
            fp_div_buffer_latency,
        };
        Ok(result)
    }
}


impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Configuration
        // -------------
        // buffers:
        //    eff addr: 2
        //     fp adds: 3
        //     fp muls: 3
        //        ints: 2
        //     reorder: 5
        
        // latencies:
        //    fp add: 2
        //    fp sub: 2
        //    fp mul: 5
        //    fp div: 10

        writeln!(f, "Configuration")?;
        writeln!(f, "-------------")?;
        writeln!(f, "buffers:")?;
        writeln!(f, "   eff addr: {}", self.eff_addr_buffer_entries)?;
        writeln!(f, "    fp adds: {}", self.fp_add_buffer_entries)?;
        writeln!(f, "    fp muls: {}", self.fp_mul_buffer_entries)?;
        writeln!(f, "       ints: {}", self.int_buffer_entries)?;
        writeln!(f, "    reorder: {}", self.reorder_buffer_entries)?;
        writeln!(f)?;
        writeln!(f, "latencies:")?;
        writeln!(f, "   fp add: {}", self.fp_add_buffer_latency)?;
        writeln!(f, "   fp sub: {}", self.fp_sub_buffer_latency)?;
        writeln!(f, "   fp mul: {}", self.fp_mul_buffer_latency)?;
        writeln!(f, "   fp div: {}", self.fp_div_buffer_latency)?;
        writeln!(f)
    }
}