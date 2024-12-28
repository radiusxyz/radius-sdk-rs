//!The getrlimit() and setrlimit() system calls get and set resource
//! limits.  Each resource has an associated soft and hard limit, as
//! defined by the rlimit structure.
//!
//! The soft limit is the value that the kernel enforces for the
//! corresponding resource.  The hard limit acts as a ceiling for the
//! soft limit: an unprivileged process may set only its soft limit
//! to a value in the range from 0 up to the hard limit, and
//! (irreversibly) lower its hard limit.  A privileged process (under
//! Linux: one with the CAP_SYS_RESOURCE capability in the initial
//! user namespace) may make arbitrary changes to either limit value.
//!
//! The value RLIM_INFINITY denotes no limit on a resource (both in
//! the structure returned by getrlimit() and in the structure passed
//! to setrlimit()).
use std::mem::MaybeUninit;

/// The [resource](https://www.man7.org/linux/man-pages/man2/getrlimit.2.html)
/// argument must be one of:
///
/// **RLIMIT\_AS**
///
/// This is the maximum size of the process's virtual memory (address space).
/// The limit is specified in bytes, and is rounded down to the system page
/// size. This limit affects calls to
/// [brk(2)](https://www.man7.org/linux/man-pages/man2/brk.2.html),
/// [mmap(2)](https://www.man7.org/linux/man-pages/man2/mmap.2.html),
/// and [mremap(2)](https://www.man7.org/linux/man-pages/man2/mremap.2.html),
/// which fail with the error **ENOMEM** upon exceeding this limit. In addition,
/// automatic stack expansion fails (and generates a **SIGSEGV** that kills the
/// process if no alternate stack has been made available via
/// [sigaltstack(2)](https://www.man7.org/linux/man-pages/man2/sigaltstack.2.html)).
/// Since the value is a _long_, on machines with a 32-bit _long_ either this
/// limit is at most 2 GiB, or this resource is unlimited.
///
/// **RLIMIT\_CORE**
///
/// This is the maximum size of a _core_ file
/// (see [core(5)](https://www.man7.org/linux/man-pages/man5/core.5.html))
/// in bytes that the process may dump. When 0 no core dump files are created.
/// When nonzero, larger dumps are truncated to this size.
///
/// **RLIMIT\_CPU**
///
/// This is a limit, in seconds, on the amount of CPU time that
/// the process can consume. When the process reaches the soft limit, it is sent
/// a **SIGXCPU** signal. The default action for this signal is to terminate the
/// process. However, the signal can be caught, and the handler can return
/// control to the main program. If the process continues to consume CPU time,
/// it will be sent **SIGXCPU** once per second until the hard limit is reached,
/// at which time it is sent **SIGKILL**. (This latter point describes Linux
/// behavior. Implementations vary in how they treat processes which continue to
/// consume CPU time after reaching the soft limit. Portable applications that
/// need to catch this signal should perform an orderly termination upon first
/// receipt of **SIGXCPU**.)
///
/// **RLIMIT\_DATA**
///
/// This is the maximum size of the process's data segment (initialized data,
/// uninitialized data, and heap). The limit is specified in bytes, and is
/// rounded down to the system page size. This limit affects calls to
/// [brk(2)](https://www.man7.org/linux/man-pages/man2/brk.2.html),
/// [sbrk(2)](https://www.man7.org/linux/man-pages/man2/sbrk.2.html),
/// and (since Linux 4.7)
/// [mmap(2)](https://www.man7.org/linux/man-pages/man2/mmap.2.html),
/// which fail with the error **ENOMEM** upon encountering the soft limit of
/// this resource.
///
/// **RLIMIT\_FSIZE**
/// This is the maximum size in bytes of files that the process may create.
/// Attempts to extend a file beyond this limit result in delivery of a
/// **SIGXFSZ** signal. By default, this signal terminates a process, but a
/// process can catch this signal instead, in which case the relevant system
/// call (e.g., [write(2)](https://www.man7.org/linux/man-pages/man2/write.2.html),
/// [truncate(2)](https://www.man7.org/linux/man-pages/man2/truncate.2.html))
/// fails with the error **EFBIG**.
///
/// **RLIMIT\_MEMLOCK**
///
/// This is the maximum number of bytes of memory that may be locked into RAM.
/// This limit is in effect rounded down to the nearest multiple of the system
/// page size. This limit affects
/// [mlock(2)](https://www.man7.org/linux/man-pages/man2/mlock.2.html),
/// [mlockall(2)](https://www.man7.org/linux/man-pages/man2/mlockall.2.html), and the
/// [mmap(2)](https://www.man7.org/linux/man-pages/man2/mmap.2.html)
/// **MAP\_LOCKED** operation. Since Linux 2.6.9, it also affects the
/// [shmctl(2)](https://www.man7.org/linux/man-pages/man2/shmctl.2.html)
/// **SHM\_LOCK** operation, where it sets a maximum on the total bytes in
/// shared memory segments (see [shmget(2)](https://www.man7.org/linux/man-pages/man2/shmget.2.html))
/// that may be locked by the real user ID of the calling process. The
/// [shmctl(2)](https://www.man7.org/linux/man-pages/man2/shmctl.2.html)
/// **SHM\_LOCK** locks are accounted for separately from the per-process memory
/// locks established by [mlock(2)](https://www.man7.org/linux/man-pages/man2/mlock.2.html),
/// [mlockall(2)](https://www.man7.org/linux/man-pages/man2/mlockall.2.html), and
/// [mmap(2)](https://www.man7.org/linux/man-pages/man2/mmap.2.html)
/// **MAP\_LOCKED**; a process can lock bytes up to this limit in each of these
/// two categories. Before Linux 2.6.9, this limit controlled the amount of
/// memory that could be locked by a privileged process. Since Linux 2.6.9, no
/// limits are placed on the amount of memory that a privileged process may
/// lock, and this limit instead governs the amount of memory that an
/// unprivileged process may lock.
///
/// **RLIMIT\_NOFILE**
///
/// This specifies a value one greater than the maximum file descriptor number
/// that can be opened by this process. Attempts
/// ([open(2)](https://www.man7.org/linux/man-pages/man2/open.2.html),
/// [pipe(2)](https://www.man7.org/linux/man-pages/man2/pipe.2.html),
/// [dup(2)](https://www.man7.org/linux/man-pages/man2/dup.2.html), etc.)
/// to exceed this limit yield the error **EMFILE**. (Historically, this limit
/// was named **RLIMIT\_OFILE** on BSD.) Since Linux 4.5, this limit also
/// defines the maximum number of file descriptors that an unprivileged process
/// (one without the **CAP\_SYS\_RESOURCE** capability) may have "in flight" to
/// other processes, by being passed across UNIX domain sockets. This limit
/// applies to the
/// [sendmsg(2)](https://www.man7.org/linux/man-pages/man2/sendmsg.2.html)
/// system call. For further details, see
/// [unix(7)](https://www.man7.org/linux/man-pages/man7/unix.7.html).
///
/// **RLIMIT\_NPROC**
///
/// This is a limit on the number of extant process (or, more precisely on
/// Linux, threads) for the real user ID of the calling process. So long as the
/// current number of processes belonging to this process's real user ID is
/// greater than or equal to this limit, [fork(2)](https://www.man7.org/linux/man-pages/man2/fork.2.html)
/// fails with the error **EAGAIN**. The **RLIMIT\_NPROC** limit is not enforced
/// for processes that have either the **CAP\_SYS\_ADMIN** or the
/// **CAP\_SYS\_RESOURCE** capability, or run with real user ID 0.
///
/// **RLIMIT\_RSS**
///
/// This is a limit (in bytes) on the process's resident set (the number of
/// virtual pages resident in RAM). This limit has effect only in Linux 2.4.x, x
/// < 30, and there affects only calls to
/// [madvise(2)](https://www.man7.org/linux/man-pages/man2/madvise.2.html)
/// specifying **MADV\_WILLNEED**.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum ResourceType {
    RLIMIT_AS,
    RLIMIT_CORE,
    RLIMIT_CPU,
    RLIMIT_DATA,
    RLIMIT_FSIZE,
    RLIMIT_MEMLOCK,
    RLIMIT_NOFILE,
    RLIMIT_NPROC,
    RLIMIT_RSS,
}

impl ResourceType {
    fn into_u32(&self) -> u32 {
        match self {
            ResourceType::RLIMIT_AS => libc::RLIMIT_AS,
            ResourceType::RLIMIT_CORE => libc::RLIMIT_CORE,
            ResourceType::RLIMIT_CPU => libc::RLIMIT_CPU,
            ResourceType::RLIMIT_DATA => libc::RLIMIT_DATA,
            ResourceType::RLIMIT_FSIZE => libc::RLIMIT_FSIZE,
            ResourceType::RLIMIT_MEMLOCK => libc::RLIMIT_MEMLOCK,
            ResourceType::RLIMIT_NOFILE => libc::RLIMIT_NOFILE,
            ResourceType::RLIMIT_NPROC => libc::RLIMIT_NPROC,
            ResourceType::RLIMIT_RSS => libc::RLIMIT_RSS,
        }
    }
}

/// The soft limit is the value that the kernel enforces for the
/// corresponding resource. The hard limit acts as a ceiling for the soft limit:
/// an unprivileged process may set only its soft limit to a value in the range
/// from 0 up to the hard limit, and (irreversibly) lower its hard limit. A
/// privileged process (under Linux: one with the **CAP\_SYS\_RESOURCE**
/// capability in the initial user namespace) may make arbitrary changes to
/// either limit value. The value **RLIM\_INFINITY** denotes no limit on a
/// resource (both in the structure returned by **getrlimit**() and in the
/// structure passed to **setrlimit**()).
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ResourceLimit {
    pub soft_limit: u64,
    pub hard_limit: u64,
}

impl ResourceLimit {
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut Self {
        self as *mut Self
    }
}

/// # Examples
///
/// ```rust
/// use radius_sdk::util::{self, ResourceType};
///
/// // Get the number of maximum file descriptor that can be opened by the current process.
/// let rlimit = util::get_resource_limit(ResourceType::RLIMIT_NOFILE).unwrap();
/// println!("{:?}", rlimit);
/// ```
pub fn get_resource_limit(resource_type: ResourceType) -> Result<ResourceLimit, std::io::Error> {
    let mut rlimit = MaybeUninit::<ResourceLimit>::uninit();
    let code = unsafe {
        libc::getrlimit(
            resource_type.into_u32(),
            rlimit.as_mut_ptr() as *mut libc::rlimit,
        )
    };
    if code.is_negative() {
        return Err(std::io::Error::from_raw_os_error(-code));
    }

    Ok(unsafe { rlimit.assume_init() })
}

/// # Examples
///
/// ```rust
/// use radius_sdk::util::{self, ResourceType};
///
/// // Set the number of file descriptor that can be opened by the current process.
/// let descriptor_count: u64 = 4096;
/// util::set_resource_limit(ResourceType::RLIMIT_NOFILE, descriptor_count).unwrap();
/// ```
pub fn set_resource_limit(resource_type: ResourceType, limit: u64) -> Result<(), std::io::Error> {
    let mut rlimit = get_resource_limit(resource_type)?;
    rlimit.soft_limit = limit;

    let code = unsafe {
        libc::setrlimit(
            resource_type.into_u32(),
            rlimit.as_mut_ptr() as *mut libc::rlimit,
        )
    };
    if code.is_negative() {
        return Err(std::io::Error::from_raw_os_error(-code));
    }

    Ok(())
}
