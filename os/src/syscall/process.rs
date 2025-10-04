//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// TODO: implement the syscall
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let inner=task::TASK_MANAGER.inner.exclusive_access();
    if _trace_request==2{
        let cur=inner.current_task;
        
      match _id {
         93=>{
            return inner.tasks[cur].sys_num.sysexit;
         },
         169=>{
            return inner.tasks[cur].sys_num.sysgettime;
         },
         410=>{
            return inner.tasks[cur].sys_num.systrace;
         },
         syscall::SYSCALL_WRITE=>{
            return inner.tasks[cur].sys_num.syswrite;
         },
         syscall::SYSCALL_YIELD=>{
            return inner.tasks[cur].sys_num.sysyield;
         },
         _=>{
            panic!("fys");
         }
      };

    }
    else if _trace_request==0{
       let s1=unsafe{(_id as *const u8).read_volatile()};
       return  s1 as isize;
    }
    else if _trace_request==1{
       let _s1=unsafe{(_id as *mut u8).write_volatile(_data as u8);};
       return 1;
    }

    -1
}