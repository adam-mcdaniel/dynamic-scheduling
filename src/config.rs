use std::fmt::{self, Display, Formatter};

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