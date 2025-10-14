//! Process management syscalls
//!
use alloc::sync::Arc;
use crate::timer::get_time;
use alloc::vec::Vec;
use crate::mm::{page_table,VirtAddr,VirtPageNum,MapPermission,address::VPNRange};
use crate::config::PAGE_SIZE;
use crate::{
    fs::{open_file, OpenFlags},
    mm::{translated_refmut, translated_str},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next,
    },
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    //trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    //trace!("kernel: sys_waitpid");
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let time=get_time();
    let t1=time/1_000_000;
    let t2=time%1_000_000;
    let mut v=Vec::new();
    for i in 0..8{
       v.push((t1>>i*8) as u8);
    }
    for i in 0..8{
       v.push((t2>>i*8) as u8);
    }
    if let Some(inner)=current_task(){
        let to=inner.inner_exclusive_access().memory_set.token();
        let mut x=page_table::translated_byte_buffer(to, _ts as *const u8, 128);
        let mut u=0;
        for i in x.iter_mut(){
          for j in i.iter_mut(){
              if u==16{
                return 0;
             }
              *j=v[u];
              u+=1;
            }
        }
    }
     
    
    0
}


/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, port: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let strp = VirtAddr::from(_start);
    if !strp.aligned(){
        return -1;
    }
    if port&0x7==0||(port>>3)!=0{
        return -1;
    }

    let real_len: usize=match _len/PAGE_SIZE
    {
        0=>0,
        _=>{
            if _len%PAGE_SIZE==0 {
                _len/PAGE_SIZE
            }
            else {
                _len/PAGE_SIZE+1
            }
        }
    };
   // let start_page:VirtPageNum=strp.into();
    let endp=VirtAddr::from(_start+real_len*PAGE_SIZE);
    let mut flags = MapPermission::from_bits_truncate(port as u8);
     if port & 0b0000_0001 != 0 {
            flags |= MapPermission::R;
        }

        if port & 0b0000_0010 != 0 {
            flags |= MapPermission::W;
        }

        if port & 0b0000_0100 != 0 {
            flags |= MapPermission::X;
        }
        flags |= MapPermission::U;
    let inner=current_task().unwrap();
    let mut inner1=inner.inner_exclusive_access();
    let vstr:VirtPageNum=strp.floor();
    let endv:VirtPageNum=endp.ceil();
    let r=VPNRange::new(vstr, endv);
    for i in r{
        if let Some(_a)=inner1.memory_set.page_table.find_pte(i.into()){
            if _a.is_valid()
            {
            return -1;
            }
        }
    }
    inner1.memory_set.insert_framed_area(strp, endp, flags);
    0

}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let strp = VirtAddr::from(_start);
    if !strp.aligned(){
        return -1;
    }
    let real_len: usize=match _len/PAGE_SIZE
    {
        0=>0,
        _=>{
            if _len%PAGE_SIZE==0 {
                _len/PAGE_SIZE
            }
            else {
                _len/PAGE_SIZE+1
            }
        }
    };
    let endp=VirtAddr::from(_start+real_len*PAGE_SIZE);
    let  inner=current_task().unwrap();
    let  mut inner1=inner.inner_exclusive_access();
    if inner1.memory_set.unmap(strp.into(),endp.into())==0{
            return 0;
    }
    -1
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token=current_user_token();
   let name=translated_str(token,_path);
    if let Some(app_inode) = open_file(name.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let tcb=current_task().unwrap().spaw(all_data.as_slice());
        let  trap_x=tcb.inner_exclusive_access().get_trap_cx();
        trap_x.x[10]=0;
        let npid=tcb.getpid();
        add_task(tcb);
        npid as isize    
    } 
    else {
        -1}
   
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    if _prio<2 {
        return  -1;
    }
    let now=current_task();
    now.unwrap().setpri(_prio)
    //_prio
}
