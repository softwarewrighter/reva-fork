//! Host calls, and whether the machine stopped.

use reva_core::Status;
use reva_core::hostcall::{HostAnswer, HostCall, HostReply};

reva_macros::machine_trait! {
    /// Waiting for the host, and stopping.
    pub trait HostCalls {
        /// The host call `Status::HostCall` waits on.
        fn pending_host_call(&self) -> Option<&HostCall> {
            self.pending_call.as_ref()
        }

        /// Answer the pending host call; the next `run` continues.
        fn resume(&mut self, reply: HostReply) {
            self.pending_call = None;
            self.host_reply = Some(HostAnswer::Reply(reply));
        }

        /// Refuse the pending host call: the line stops with `why`, as for a
        /// service the host does not have.
        fn refuse(&mut self, why: String) {
            self.pending_call = None;
            self.host_reply = Some(HostAnswer::Unavailable(why));
        }

        /// Why `run` must not step: halted, restarting, or a host call waits.
        #[inline(always)]
        fn stopped(&self) -> Option<Status> {
            if let Some(code) = self.halted {
                Some(Status::Halted(code))
            } else if self.restart {
                Some(Status::Restart)
            } else if self.pending_call.is_some() {
                Some(Status::HostCall)
            } else {
                None
            }
        }
    }
}
