//! Process management syscalls

use crate::task::{change_program_brk, current_user_token, exit_current_and_run_next, get_current_task_syscall_times, suspend_current_and_run_next, task_mmap, task_munmap,};
use crate::timer::get_time_us;
use crate::mm::{copyout, PageTable, PhysAddr, VirtAddr /*translated_byte_buffer*/};
use crate::config::PAGE_SIZE;

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
    /***  sys_get_time without pagetable
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    ***/
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let timeval = TimeVal {
        sec : us / 1_000_000,
        usec: us % 1_000_000,
    };  // in kernel space

    copyout(&timeval, _ts);
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// refer to xv6 kernel/vm.c:walkaddr
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    /*** 
    *   如果 trace_request 为 0，则 id 应被视作 *const u8 ，表示读取当前任务 id 地址处一个字节的无符号整数值。此时应忽略 data 参数。返回值为 id 地址处的值。
    *
    *   如果 trace_request 为 1，则 id 应被视作 *const u8 ，表示写入 data （作为 u8，即只考虑最低位的一个字节）到该用户程序 id 地址处。返回值应为0。
    *
    *   如果 trace_request 为 2，表示查询当前任务调用编号为 id 的系统调用的次数，返回值为这个调用次数。本次调用也计入统计 。
    *
    *   在读取（trace_request 为 0）时，如果对应地址用户不可见或不可读，则返回值应为 -1（isize 格式的 -1，而非 u8）。
    *
    *   在写入（trace_request 为 1）时，如果对应地址用户不可见或不可写，则返回值应为 -1（isize 格式的 -1，而非 u8）。
    *
    *   否则，忽略其他参数，返回值为 -1。
    ***/
    trace!("kernel: sys_trace");
    let token = current_user_token();  // satp
    let pgtbl = PageTable::from_token(token);  // user page table
    let vaddr = VirtAddr::from(_id);
    // can't just simply use unwrap here
    let pte = match pgtbl.translate(vaddr.floor()) {
        Some(pte ) => pte,
        None => return -1,
    };  // get user pgtbl's pte

    // I use match here instead of if-else in ch3
    match _trace_request {
        0 => {
            if !pte.is_user() || !pte.readable() {
                -1
            }
            else {
                let physical_page_number: PhysAddr = pte.ppn().into();
                let offset = vaddr.page_offset(); 
                let paddr = physical_page_number.0 | offset; 
                unsafe {*(paddr as *const u8) as isize}
            }
        }
        1 => {
            if !pte.is_user() || !pte.writable() {
                -1
            }
            else {
                let physical_page_number: PhysAddr = pte.ppn().into();
                let offset = vaddr.page_offset();
                let paddr = physical_page_number.0 | offset;
                unsafe {
                    *(paddr as *mut u8) = _data as u8;
                    0
                }
            }

        }
        2 => {
            get_current_task_syscall_times(_id) as isize
        }
        _ => {
            -1
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _prot: usize) -> isize {
    /*** 
        syscall ID：222

        申请长度为 len 字节的物理内存（不要求实际物理内存位置，可以随便找一块），将其映射到 start 开始的虚存，内存页属性为 prot

        参数：
        start 需要映射的虚存起始地址，要求按页对齐

        len 映射字节长度，可以为 0

        prot：第 0 位表示是否可读，第 1 位表示是否可写，第 2 位表示是否可执行。其他位无效且必须为 0

        返回值：执行成功则返回 0，错误返回 -1

        说明：
        为了简单，目标虚存区间要求按页对齐，len 可直接按页向上取整，不考虑分配失败时的页回收。

        可能的错误：
        start 没有按页大小对齐

        prot & !0x7 != 0 (prot 其余位必须为0)

        prot & 0x7 = 0 (这样的内存无意义)

        [start, start + len) 中存在已经被映射的页

        物理内存不足
    ***/
    trace!("kernel: sys_mmap");
    if !VirtAddr::from(_start).aligned() || _prot & !0x7 != 0 || _prot & 0x7 == 0 {
        return -1;
    }

    let len_aligned = (_len + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;
    task_mmap(_start, len_aligned, _prot << 1)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if !VirtAddr::from(_start).aligned() {return -1;}
    let len_aligned = (_len + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;

    task_munmap(_start, len_aligned)
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
