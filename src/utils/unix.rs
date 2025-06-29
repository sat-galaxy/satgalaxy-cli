pub fn limit_time(max_cpu_time:u64)->anyhow::Result<()> {
    if max_cpu_time == 0 {
        return Ok(());
    }
    let (rlim_cur, rlim_max) = rlimit::getrlimit(rlimit::Resource::CPU)?;
    if rlim_cur < max_cpu_time {
        return Err(anyhow::anyhow!(
            "Current CPU time limit ({}) is less than the requested limit ({})",
            rlim_cur,
            max_cpu_time
        ));
    }

    rlimit::setrlimit(rlimit::Resource::CPU, max_cpu_time, rlim_max)?;
    Ok(())
}

pub fn limit_memory(max_memory:u64) -> anyhow::Result<()> {
    if max_memory == 0 {
        return Ok(());
    }
    let (rlim_cur, rlim_max) = rlimit::getrlimit(rlimit::Resource::AS)?;
    if rlim_cur < max_memory {
        return Err(anyhow::anyhow!(
            "Current memory limit ({}) is less than the requested limit ({})",
            rlim_cur,
            max_memory
        ));
    }
    rlimit::setrlimit(rlimit::Resource::AS, max_memory, rlim_max)?;
    Ok(())
}