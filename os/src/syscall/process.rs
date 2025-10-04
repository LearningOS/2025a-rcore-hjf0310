//! Process management syscalls
use alloc::vec::Vec;

use crate::{mm::{self,address::VPNRange, MapPermission}, task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next}, timer::get_time};
use crate::config::PAGE_SIZE;
use crate::mm::address::{VirtPageNum,VirtAddr};
use crate::task::{TASK_MANAGER};
use crate::syscall;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let  inner=TASK_MANAGER.inner.exclusive_access();
    let cur =inner.current_task;
    let to=inner.tasks[cur].memory_set.token();
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
    //let realtime=t1|t2;
    let mut x=mm::page_table::translated_byte_buffer(to, _ts as *const u8, 128);
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
return 0;
    
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let inner=TASK_MANAGER.inner.exclusive_access();
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
            panic!("ss");
         }

      };

    }
    else if _trace_request==0{
       if _id>1<<39-1{
        return -1;
       }
       let virpage:VirtPageNum=VirtAddr::from(_id).floor();
       let cur=inner.current_task;
       let pentry=inner.tasks[cur].memory_set.translate(virpage);
       match pentry {
       None=>{return -1;}
       _=>{}
       }
       
       if !pentry.unwrap().is_valid()||!pentry.unwrap().readable() {
        return -1;
       }
       let s=mm::page_table::translated_byte_buffer(inner.tasks[cur].memory_set.page_table.token(), _id as *const u8, 1);
       let s1=s[0][0];
       return  s1 as isize;
    }
    else if _trace_request==1{
      // let _s1=unsafe{(_id as *mut u8).write_volatile(_data as u8);};
       if _id>1<<39-1{
        return -1;
       }
       let virpage:VirtPageNum=VirtAddr::from(_id).floor();
       let cur=inner.current_task;
       let pentry=inner.tasks[cur].memory_set.translate(virpage);
       match pentry {
       None=>{return -1;}
       _=>{}
       }
       if !pentry.unwrap().is_valid()||!pentry.unwrap().writable() {
        return -1;
       }
       let mut s=mm::page_table::translated_byte_buffer(inner.tasks[cur].memory_set.page_table.token(), _id as *const u8, 1);
       if let Some(first_slice) = s.get_mut(0) {
     // 修改切片中的值
         if let Some(o)=first_slice.get_mut(0){
              *o=_data as u8;
         }
}      return 0;
    }

    -1
}

pub fn sys_mmap(_start: usize, _len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
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
    let mut inner=TASK_MANAGER.inner.exclusive_access();
    let cur=inner.current_task;
    let vstr:VirtPageNum=strp.floor();
    let endv:VirtPageNum=endp.ceil();
    let r=VPNRange::new(vstr, endv);
    for i in r{
        if let Some(_a)=inner.tasks[cur].memory_set.page_table.find_pte(i.into()){
            if _a.is_valid()
            {
            return -1;
            }
        }
    }
    inner.tasks[cur].memory_set.insert_framed_area(strp, endp, flags);
    


    0

}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
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
    let mut inner=TASK_MANAGER.inner.exclusive_access();
    let cur=inner.current_task;
    if inner.tasks[cur].memory_set.unmap(strp.into(),endp.into())==0{
            return 0;
    }
    -1
    
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
