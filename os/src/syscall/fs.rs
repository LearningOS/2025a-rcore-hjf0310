//! File and filesystem-related syscalls
use crate::fs::{open_file, OSInode, OpenFlags, Stat};
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};
use alloc::sync::Arc;
use crate::fs::inode::ROOT_INODE;
use  crate::fs::StatMode;
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let task=current_task().unwrap();
    let inner=task.inner_exclusive_access();
    let _s=Arc::clone(inner.fd_table[_fd].as_ref().unwrap());
    let osno=_s.get_info().unwrap();
    let iid_t=osno.inner.exclusive_access();
    let iid=ROOT_INODE.fs.lock().get_inode_id(Arc::clone(&iid_t.inode));
    let link_num=ROOT_INODE.fs.lock().get_link_num(iid);
    
    let tokn=inner.memory_set.token();
    let v=translated_refmut(tokn, _st);
    v.dev=0;
    v.ino=iid as u64;
    v.mode=StatMode::FILE;
    v.nlink=link_num+1;
    0
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let cur=current_task().unwrap();
    let token=current_user_token();
    let source_str=translated_str(token, _old_name);
    let dest_str=translated_str(token, _new_name);
    let flag=OpenFlags::RDWR;
    if let Some(a)=open_file(&source_str, flag){
         let inner1=a.inner.exclusive_access();
         let n=&inner1.inode;
         let ind=ROOT_INODE.create_link(&dest_str, n.clone());
         let mut inner=cur.inner_exclusive_access();
         let fd=inner.alloc_fd();
         let u=OSInode::new(true, true, ind);
         inner.fd_table[fd]=Some(Arc::new(u));
         let id=ROOT_INODE.fs.lock().get_inode_id(Arc::clone(&inner1.inode));
         ROOT_INODE.fs.lock().add_nlink(id);
   }

  0
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let cur=current_task().unwrap();
    let  cur_inner=cur.inner_exclusive_access();
    let token=cur_inner.get_user_token();
    let s_name=translated_str(token,_name);
    let f=OpenFlags::RDWR;
    if let Some(a)=open_file(&s_name, f){
        let eq2=Arc::clone(&(a.inner.exclusive_access().inode));
        let id2=ROOT_INODE.fs.lock().get_inode_id(eq2);
        ROOT_INODE.fs.lock().sub_nlink(id2);
        ROOT_INODE.un_link(&s_name);
    }

    
    0
}
