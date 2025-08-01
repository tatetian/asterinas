// SPDX-License-Identifier: MPL-2.0

use core::sync::atomic::AtomicU32;

use ostd::{
    cpu::{
        context::{FpuContext, UserContext},
        CpuSet,
    },
    sync::RwArc,
    task::Task,
};

use super::{thread_table, PosixThread, ThreadLocal};
use crate::{
    fs::{file_table::FileTable, thread_info::ThreadFsInfo},
    prelude::*,
    process::{
        posix_thread::name::ThreadName,
        signal::{sig_mask::AtomicSigMask, sig_queues::SigQueues},
        Credentials, Process,
    },
    sched::{Nice, SchedPolicy},
    thread::{task, Thread, Tid},
    time::{clocks::ProfClock, TimerManager},
};

/// The builder to build a posix thread
pub struct PosixThreadBuilder {
    // The essential part
    tid: Tid,
    user_ctx: Arc<UserContext>,
    process: Weak<Process>,
    credentials: Credentials,

    // Optional part
    thread_name: Option<ThreadName>,
    set_child_tid: Vaddr,
    clear_child_tid: Vaddr,
    file_table: Option<RwArc<FileTable>>,
    fs: Option<Arc<ThreadFsInfo>>,
    sig_mask: AtomicSigMask,
    sig_queues: SigQueues,
    sched_policy: SchedPolicy,
    fpu_context: FpuContext,
}

impl PosixThreadBuilder {
    pub fn new(tid: Tid, user_ctx: Arc<UserContext>, credentials: Credentials) -> Self {
        Self {
            tid,
            user_ctx,
            process: Weak::new(),
            credentials,
            thread_name: None,
            set_child_tid: 0,
            clear_child_tid: 0,
            file_table: None,
            fs: None,
            sig_mask: AtomicSigMask::new_empty(),
            sig_queues: SigQueues::new(),
            sched_policy: SchedPolicy::Fair(Nice::default()),
            fpu_context: FpuContext::new(),
        }
    }

    pub fn process(mut self, process: Weak<Process>) -> Self {
        self.process = process;
        self
    }

    pub fn thread_name(mut self, thread_name: Option<ThreadName>) -> Self {
        self.thread_name = thread_name;
        self
    }

    pub fn set_child_tid(mut self, set_child_tid: Vaddr) -> Self {
        self.set_child_tid = set_child_tid;
        self
    }

    pub fn clear_child_tid(mut self, clear_child_tid: Vaddr) -> Self {
        self.clear_child_tid = clear_child_tid;
        self
    }

    pub fn file_table(mut self, file_table: RwArc<FileTable>) -> Self {
        self.file_table = Some(file_table);
        self
    }

    pub fn fs(mut self, fs: Arc<ThreadFsInfo>) -> Self {
        self.fs = Some(fs);
        self
    }

    pub fn sig_mask(mut self, sig_mask: AtomicSigMask) -> Self {
        self.sig_mask = sig_mask;
        self
    }

    pub fn fpu_context(mut self, fpu_context: FpuContext) -> Self {
        self.fpu_context = fpu_context;
        self
    }

    pub fn build(self) -> Arc<Task> {
        let Self {
            tid,
            user_ctx,
            process,
            credentials,
            thread_name,
            set_child_tid,
            clear_child_tid,
            file_table,
            fs,
            sig_mask,
            sig_queues,
            sched_policy,
            fpu_context,
        } = self;

        let file_table = file_table.unwrap_or_else(|| RwArc::new(FileTable::new_with_stdio()));

        let fs = fs.unwrap_or_else(|| Arc::new(ThreadFsInfo::default()));

        let root_vmar = process
            .upgrade()
            .unwrap()
            .lock_root_vmar()
            .unwrap()
            .dup()
            .unwrap();

        Arc::new_cyclic(|weak_task| {
            let posix_thread = {
                let prof_clock = ProfClock::new();
                let virtual_timer_manager = TimerManager::new(prof_clock.user_clock().clone());
                let prof_timer_manager = TimerManager::new(prof_clock.clone());

                PosixThread {
                    process,
                    tid,
                    name: Mutex::new(thread_name),
                    credentials,
                    file_table: Mutex::new(Some(file_table.clone_ro())),
                    fs,
                    sig_mask,
                    sig_queues,
                    signalled_waker: SpinLock::new(None),
                    prof_clock,
                    virtual_timer_manager,
                    prof_timer_manager,
                    io_priority: AtomicU32::new(0),
                }
            };

            let cpu_affinity = CpuSet::new_full();
            let thread = Arc::new(Thread::new(
                weak_task.clone(),
                posix_thread,
                cpu_affinity,
                sched_policy,
            ));

            let thread_local = ThreadLocal::new(
                set_child_tid,
                clear_child_tid,
                root_vmar,
                file_table,
                fpu_context,
            );

            thread_table::add_thread(tid, thread.clone());
            task::create_new_user_task(user_ctx, thread, thread_local)
        })
    }
}
