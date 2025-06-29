#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

use sysinfo::{Pid, ProcessesToUpdate};

pub fn get_memory()->Option<u64>{
      // 获取当前进程 ID
    let pid = Pid::from_u32(std::process::id());
    let mut sys = sysinfo::System::new();

    // 刷新进程信息
    sys.refresh_processes(ProcessesToUpdate::All,true);

    // 查询当前进程的内存使用
    if let Some(process) = sys.process(pid) {
        // 内存使用量（单位：字节）
        let memory_usage_bytes = process.memory();
        return  Some(memory_usage_bytes);
    } else {
       return None;
    }

}
