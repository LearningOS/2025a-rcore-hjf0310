//! Types related to task management

use super::TaskContext;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    ///
    pub sys_num: TaskSyscall,
    ///addr
    pub sys_addr:u8,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}

///
#[derive(Copy, Clone)]
pub struct TaskSyscall{
    ///
    pub sysgettime:isize,
    ///
    pub systrace:isize,
    ///
    pub syswrite:isize,
    ///
    pub sysyield:isize,
    ///
    pub sysexit:isize,
}
