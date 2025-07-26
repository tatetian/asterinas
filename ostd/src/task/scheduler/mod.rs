// SPDX-License-Identifier: MPL-2.0

//! Task scheduling.
//!
//! # Scheduler Injection
//!
//! The task scheduler of an OS is a complex beast,
//! and the most suitable scheduling algorithm often depends on the target usage scenario.
//! To offer flexibility and avoid code bloat,
//! OSTD does not include a one-size-fits-all task scheduler.
//! Instead, it allows the client to implement a custom scheduler (in safe Rust, of course)
//! and register it with OSTD.
//! This feature is known as **scheduler injection**.
//!
//! The client kernel performs scheduler injection via the [`inject_scheduler`] API.
//! This API should be called as early as possible during kernel initialization,
//! before any [`Task`]-related APIs are used.
//! This requirement is reasonable since `Task`s depend on the scheduler.
//!
//! # Scheduler Abstraction
//!
//! The `inject_scheduler` API accepts an object implementing the [`Scheduler`] trait,
//! which abstracts over any SMP-aware task scheduler.
//! Whenever an OSTD client spawns a new task (via [`crate::task::TaskOptions`])
//! or wakes a sleeping task (e.g., via [`crate::sync::Waker`]),
//! OSTD internally forwards the corresponding `Arc<Task>`
//! to the scheduler by invoking the [`Scheduler::enqueue`] method.
//! This allows the injected scheduler to manage all runnable tasks.
//!
//! Each enqueued task is dispatched to one of the per-CPU local runqueues,
//! which manage all runnable tasks on a specific CPU.
//! A local runqueue is abstracted by the [`LocalRunQueue`] trait.
//! OSTD accesses the local runqueue of the current CPU
//! via [`Scheduler::local_rq_with`] or [`Scheduler::mut_local_rq_with`],
//! which return immutable and mutable references to `dyn LocalRunQueue`, respectively.
//!
//! The [`LocalRunQueue`] trait enables OSTD to inspect and manipulate local runqueues.
//! For instance, OSTD invokes the [`LocalRunQueue::pick_next`] method
//! to let the scheduler select the next task to run.
//! OSTD then performs a context switch to that task,
//! which becomes the _current_ running task, accessible via [`LocalRunQueue::current`].
//! When the current task is about to sleep (e.g., via [`crate::sync::Waiter`]),
//! OSTD removes it from the local runqueue using [`LocalRunQueue::dequeue_current`].
//!
//! The interfaces of `Scheduler` and `LocalRunQueue` are simple
//! yet (perhaps surprisingly) powerful enough to support
//! even complex and advanced task scheduler implementations.
//! Scheduler implementations are free to employ any load-balancing strategy
//! to dispatch enqueued tasks across local runqueues,
//! and each local runqueue is free to choose any prioritization strategy
//! for selecting the next task to run.
//! Based on OSTD's scheduling abstractions,
//! the Asterinas kernel has successfully supported multiple Linux scheduling classes,
//! including both real-time and normal policies.
//!
//! # Safety Impact
//!
//! While OSTD delegates scheduling decisions to the injected task scheduler,
//! it verifies these decisions to avoid undefined behavior.
//! In particular, it enforces the following safety invariant:
//!
//! > A task must not be scheduled to run on more than one CPU at a time.
//!
//! Violating this invariant—e.g., running the same task on two CPUs concurrently—
//! can have catastrophic consequences,
//! as the task's stack and internal state may be corrupted by concurrent modifications.

mod fifo_scheduler;
pub mod info;

use spin::Once;

use super::{preempt::cpu_local, processor, Task};
use crate::{
    cpu::{CpuId, CpuSet, PinCurrentCpu},
    prelude::*,
    task::disable_preempt,
    timer,
};

/// Injects a custom implementation of task scheduler into OSTD.
///
/// This function can only be called once and must be called during the initialization phase of kernel,
/// before any [`Task`]-related APIs are invoked.
pub fn inject_scheduler(scheduler: &'static dyn Scheduler<Task>) {
    SCHEDULER.call_once(|| scheduler);

    timer::register_callback(|| {
        SCHEDULER.get().unwrap().mut_local_rq_with(&mut |local_rq| {
            if local_rq.update_current(UpdateFlags::Tick) {
                cpu_local::set_need_preempt();
            }
        })
    });
}

static SCHEDULER: Once<&'static dyn Scheduler<Task>> = Once::new();

/// A SMP-aware task scheduler.
pub trait Scheduler<T = Task>: Sync + Send {
    /// Enqueues a runnable task.
    ///
    /// The scheduler implementators can perform load-balancing or some time accounting work here.
    ///
    /// The newly-enqueued task may have a higher priority than the currently running one on a CPU
    /// and thus should preempt the latter.
    /// In this case, this method returns the ID of that CPU.
    fn enqueue(&self, runnable: Arc<T>, flags: EnqueueFlags) -> Option<CpuId>;

    /// Gets an immutable access to the local runqueue of the current CPU.
    fn local_rq_with(&self, f: &mut dyn FnMut(&dyn LocalRunQueue<T>));

    /// Gets a mutable access to the local runqueue of the current CPU.
    fn mut_local_rq_with(&self, f: &mut dyn FnMut(&mut dyn LocalRunQueue<T>));
}

/// A per-CPU, local runqueue.
///
/// This abstraction allows OSTD to inspect and manipulate local runqueues.
///
/// Conceptually, a local runqueue maintains:
/// 1. A priority queue of runnable tasks.
/// 2. The current running task.
/// The definition of "priority" is left to the concrete implementation.
///
/// # Interactions with OSTD
///
/// ## Overview
///
/// It is crucial for implementers of `LocalRunQueue`
/// to understand how OSTD interacts with local runqueues.
///
/// A local runqueue is consulted by OSTD in response to one of three scheduling events:
/// - **Yielding**, triggered by [`Task::yield_now`], where the current task voluntarily gives up CPU time.
/// - **Sleeping**, triggered by [`crate::sync::Waiter::wait`]
///   or any synchronization primitive built upon it (e.g., [`crate::sync::WaitQueue`], [`crate::sync::Mutex`]),
///   which blocks the current task until a wake-up event occurs.
/// - **Ticking**, triggered periodically by the system timer
///   (see [`crate::arch::timer::TIMER_FREQ`]),
///   providing an opportunity to do time accounting and consider preemption.
///
/// OSTD handles all three types of events using a three-step workflow:
/// 1. Acquire exclusive access to the local runqueue using [`Scheduler::mut_local_rq_with`].
/// 2. Call [`LocalRunQueue::update_current`] to update the current task's state
///    and determine whether it should be replaced with another runnable task.
///    If so, proceed to step 3.
/// 3. Select the next task to run using [`LocalRunQueue::pick_next`].
///
/// ## When to Pick the Next Task?
///
/// As shown above,
/// OSTD guarantees that `pick_next` is only called
/// when the current task should and can be replaced.
/// This avoids unnecessary invocations and improves efficiency.
///
/// But under what conditions should the current task be replaced?
/// Two criteria must be met:
/// 1. There exists at least one other runnable task in the runqueue.
/// 2. That task should preempt the current one, if present.
///
/// Some implications of these rules:
/// - If the runqueue is empty, `update_current` must return `false`—there's nothing to run.
/// - If the runqueue is non-empty but the current task is absent,
///   `update_current` should return `true`—anything is better than nothing.
/// - If the runqueue is non-empty and the flag is `UpdateFlags::WAIT`,
///   `update_current` should also return `true`,
///   because the current task is about to block.
/// - In other cases, the return value depends on the scheduler's prioritization policy.
///   For instance, a real-time task may only be preempted by a higher-priority task
///   or if it explicitly yields.
///   A normal task under Linux's CFS may be preempted by a task with smaller vruntime,
///   but never by the idle task.
///
/// ## Intern Working
///
/// Below are simplified examples of how OSTD interacts with the local runqueue
/// for the three types of scheduling events.
///
/// ### Yielding
///
/// ```rust
/// fn yield(scheduler: &'static dyn Scheduler) {
///     let next_task_op = scheduler.mut_local_rq_with(|local_rq| {
///         let need_preempt = local_rq.update_current(UpdateFlags::Yield);
///         if !need_preempt {
///             return None;
///         }
///         local_rq.pick_next().map(|next| next.clone())
///     });
///     let Some(next_task) = next_task_op else {
///         return;
///     };
///     switch_to(next_task);
/// }
/// ```
///
/// ### Sleeping
///
/// ```rust
/// fn sleep<F: Fn() -> bool>(scheduler: &'static dyn Scheduler, is_woken: F) {
///     let mut is_first_attempt = true;
///     let mut next_task_opt = None;
///     while scheduler.mut_local_rq_with(|local_rq| {
///         if is_first_attempt {
///             if is_woken() {
///                 return false; // exit loop
///             }
///             is_first_attempt = false;
///
///             let should_pick_next = local_rq.update_current(UpdateFlags::Wait);
///             let _current = local_rq.dequeue_current();
///             if !should_pick_next {
///                 return true; // continue loop
///             }
///
///             next_task_opt = local_rq.pick_next().map(|next| next.clone());
///             next_task_opt.is_none() // continue if no next task
///         } else {
///             next_task_opt = local_rq.pick_next().map(|next| next.clone());
///             next_task_opt.is_none() // continue if no next task
///         }
///     }) {}
///     let Some(next_task) = next_task_opt else {
///         return;
///     };
///     switch_to(next_task);
/// }
/// ```
///
/// ### Ticking
///
/// ```rust
/// fn on_tick(scheduler: &'static dyn Scheduler) {
///     scheduler.mut_local_rq_with(|local_rq| {
///         let should_and_can_preempt = local_rq.update_current(UpdateFlags::Tick);
///         if should_and_can_preempt {
///             cpu_local::set_need_preempt();
///         }
///     });
/// }
///
/// fn might_preempt(scheduler: &'static dyn Scheduler) {
///     if !cpu_local::should_preempt() {
///         return;
///     }
///     let next_task = scheduler
///         .mut_local_rq_with(|local_rq| local_rq.pick_next().unwrap().clone());
///     switch_to(next_task);
/// }
/// ```
pub trait LocalRunQueue<T = Task> {
    /// Gets the current runnable task.
    fn current(&self) -> Option<&Arc<T>>;

    /// Updates the current runnable task's scheduling statistics and potentially its
    /// position in the runqueue.
    ///
    /// The return value of this method indicates whether an invocation of `pick_next` should be followed
    /// to find a next task to replace the current one.
    fn update_current(&mut self, flags: UpdateFlags) -> bool;

    /// Picks the next runnable task.
    ///
    /// This method returns the chosen next runnable task,
    /// which will replace the current one.
    /// If no runnable task, then this method returns `None`.
    /// 
    /// Note that OSTD invokes this method only if a previous call to `update_current` returns true.
    fn pick_next(&mut self) -> Option<&Arc<T>>;

    /// Removes the current runnable task from runqueue.
    ///
    /// This method returns the current runnable task.
    /// If there is no current runnable task, this method returns `None`.
    fn dequeue_current(&mut self) -> Option<Arc<T>>;
}

/// Possible triggers of an `enqueue` action.
#[derive(PartialEq, Copy, Clone)]
pub enum EnqueueFlags {
    /// Spawn a new task.
    Spawn,
    /// Wake a sleeping task.
    Wake,
}

/// Possible triggers of an `update_current` action.
#[derive(PartialEq, Copy, Clone)]
pub enum UpdateFlags {
    /// Timer interrupt.
    Tick,
    /// Task waiting.
    Wait,
    /// Task yielding.
    Yield,
}

/// Preempts the current task.
#[track_caller]
pub(crate) fn might_preempt() {
    if !cpu_local::should_preempt() {
        return;
    }
    yield_now();
}

/// Blocks the current task unless `has_woken()` returns `true`.
///
/// Note that this method may return due to spurious wake events. It's the caller's responsibility
/// to detect them (if necessary).
#[track_caller]
pub(crate) fn park_current<F>(has_woken: F)
where
    F: Fn() -> bool,
{
    let mut current = None;
    let mut is_first_try = true;

    reschedule(|local_rq: &mut dyn LocalRunQueue| {
        if is_first_try {
            if has_woken() {
                return ReschedAction::DoNothing;
            }

            // Note the race conditions: the current task may be woken after the above `has_woken`
            // check, but before the below `dequeue_current` action, we need to make sure that the
            // wakeup event isn't lost.
            //
            // Currently, for the FIFO scheduler, `Scheduler::enqueue` will try to lock `local_rq`
            // when the above race condition occurs, so it will wait until we finish calling the
            // `dequeue_current` method and nothing bad will happen. This may need to be revisited
            // after more complex schedulers are introduced.

            local_rq.update_current(UpdateFlags::Wait);
            current = local_rq.dequeue_current();
        }

        if let Some(next_task) = local_rq.pick_next() {
            if Arc::ptr_eq(current.as_ref().unwrap(), next_task) {
                return ReschedAction::DoNothing;
            }
            return ReschedAction::SwitchTo(next_task.clone());
        }

        is_first_try = false;
        ReschedAction::Retry
    });
}

/// Unblocks a target task.
pub(crate) fn unpark_target(runnable: Arc<Task>) {
    let preempt_cpu = SCHEDULER
        .get()
        .unwrap()
        .enqueue(runnable, EnqueueFlags::Wake);
    if let Some(preempt_cpu_id) = preempt_cpu {
        set_need_preempt(preempt_cpu_id);
    }
}

/// Enqueues a newly built task.
///
/// Note that the new task is not guaranteed to run at once.
#[track_caller]
pub(super) fn run_new_task(runnable: Arc<Task>) {
    // FIXME: remove this check for `SCHEDULER`.
    // Currently OSTD cannot know whether its user has injected a scheduler.
    if !SCHEDULER.is_completed() {
        fifo_scheduler::init();
    }

    let preempt_cpu = SCHEDULER
        .get()
        .unwrap()
        .enqueue(runnable, EnqueueFlags::Spawn);
    if let Some(preempt_cpu_id) = preempt_cpu {
        set_need_preempt(preempt_cpu_id);
    }

    might_preempt();
}

fn set_need_preempt(cpu_id: CpuId) {
    let preempt_guard = disable_preempt();

    if preempt_guard.current_cpu() == cpu_id {
        cpu_local::set_need_preempt();
    } else {
        crate::smp::inter_processor_call(&CpuSet::from(cpu_id), || {
            cpu_local::set_need_preempt();
        });
    }
}

/// Dequeues the current task from its runqueue.
///
/// This should only be called if the current is to exit.
#[track_caller]
pub(super) fn exit_current() -> ! {
    reschedule(|local_rq: &mut dyn LocalRunQueue| {
        let _ = local_rq.dequeue_current();
        if let Some(next_task) = local_rq.pick_next() {
            ReschedAction::SwitchTo(next_task.clone())
        } else {
            ReschedAction::Retry
        }
    });

    unreachable!()
}

/// Yields execution.
#[track_caller]
pub(super) fn yield_now() {
    reschedule(|local_rq| {
        local_rq.update_current(UpdateFlags::Yield);
        if let Some(next_task) = local_rq.pick_next() {
            ReschedAction::SwitchTo(next_task.clone())
        } else {
            ReschedAction::DoNothing
        }
    })
}

/// Do rescheduling by acting on the scheduling decision (`ReschedAction`) made by a
/// user-given closure.
///
/// The closure makes the scheduling decision by taking the local runqueue has its input.
#[track_caller]
fn reschedule<F>(mut f: F)
where
    F: FnMut(&mut dyn LocalRunQueue) -> ReschedAction,
{
    // Even if the decision below is `DoNothing`, we should clear this flag. Meanwhile, to avoid
    // race conditions, we should do this before making the decision.
    cpu_local::clear_need_preempt();

    let next_task = loop {
        let mut action = ReschedAction::DoNothing;
        SCHEDULER.get().unwrap().mut_local_rq_with(&mut |rq| {
            action = f(rq);
        });

        match action {
            ReschedAction::DoNothing => {
                return;
            }
            ReschedAction::Retry => {
                continue;
            }
            ReschedAction::SwitchTo(next_task) => {
                break next_task;
            }
        };
    };

    // `switch_to_task` will spin if it finds that the next task is still running on some CPU core,
    // which guarantees soundness regardless of the scheduler implementation.
    //
    // FIXME: The scheduler decision and context switching are not atomic, which can lead to some
    // strange behavior even if the scheduler is implemented correctly. See "Problem 2" at
    // <https://github.com/asterinas/asterinas/issues/1633> for details.
    processor::switch_to_task(next_task);
}

/// Possible actions of a rescheduling.
enum ReschedAction {
    /// Keep running current task and do nothing.
    DoNothing,
    /// Loop until finding a task to swap out the current.
    Retry,
    /// Switch to target task.
    SwitchTo(Arc<Task>),
}
