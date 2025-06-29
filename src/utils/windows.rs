pub fn limit_time(max_cpu_time:u64)->anyhow::Result<()> {
    if max_cpu_time==0 {
        return Ok(());
    }
    anyhow::anyhow!("CPU limit not supported on Windows");
}

pub fn limit_memory(max_memory:u64)->anyhow::Result<()> {
    if max_memory==0 {
        return Ok(());
    }
    anyhow::anyhow!("Memory limit not supported on Windows");
}