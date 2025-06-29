pub fn limit_time(max_cpu_time:u32) {
    if max_cpu_time==0 {
        return;
    }
    println!("c WARNING: CPU limit not supported  on windows");
}

pub fn limit_memory(max_memory:u32) {
    if max_memory==0 {
        return;
    }
    println!("c WARNING: Memory limit not supported  on windows");
}