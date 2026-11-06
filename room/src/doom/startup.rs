//! Informational output for one synchronous engine initialization.
//!
//! This state belongs to the same static image as the Doom world. Hosts using
//! compiler-isolated images activate that image before entering Room; the
//! standalone frontend owns the only world on its event-loop thread.

use std::{marker::PhantomData, rc::Rc};

/// Controls informational initialization output. Errors and warnings are never
/// routed through this option, and normal output resumes when startup returns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StartupLog {
    #[default]
    Normal,
    Quiet,
}

static mut CURRENT: StartupLog = StartupLog::Normal;

struct Scope {
    previous: StartupLog,
    // The active static image must not change workers before restoration.
    _worker: PhantomData<Rc<()>>,
}

impl Drop for Scope {
    fn drop(&mut self) {
        // The caller keeps exclusive ownership of this static image until
        // scoped() returns, including when the initialization unwinds.
        unsafe { std::ptr::addr_of_mut!(CURRENT).write(self.previous) };
    }
}

/// Runs initialization with its selected informational output mode.
///
/// # Safety
/// The caller must exclusively own the active Doom static image throughout
/// this call. `run` must neither move to another worker nor deactivate it.
pub(crate) unsafe fn scoped<R>(log: StartupLog, run: impl FnOnce() -> R) -> R {
    let previous = unsafe { std::ptr::addr_of!(CURRENT).read() };
    let _scope = Scope {
        previous,
        _worker: PhantomData,
    };
    unsafe { std::ptr::addr_of_mut!(CURRENT).write(log) };
    run()
}

/// Emits an informational message only when the active startup permits it.
///
/// # Safety
/// The caller must own the active Doom image, as for other Room globals.
/// The closure must contain output only, without simulation side effects.
pub(crate) unsafe fn info(write: impl FnOnce()) {
    if unsafe { std::ptr::addr_of!(CURRENT).read() } == StartupLog::Normal {
        write();
    }
}

#[cfg(test)]
mod tests;
