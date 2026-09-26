use crate::{Pump, Step, os};
use nebulax_terminal::{OutputEvent, ResizeOutcome, Size, Terminal};
use std::fs::File;
use std::io;
use std::process::{Child, Command, ExitStatus};

/// One PTY, direct child and engine, mutated only by their owner. No worker thread.
/// This experiment does not provide shell job-tree supervision.
pub struct Session {
    master: Option<File>,
    child: Child,
    status: Option<ExitStatus>,
    pump: Pump,
}
impl Session {
    pub fn spawn(command: Command, terminal: Terminal) -> io::Result<Self> {
        os::validate_size(terminal.size())?;
        let (master, child) = os::spawn(command, terminal.size())?;
        Ok(Self {
            master: Some(master),
            child,
            status: None,
            pump: Pump::new(terminal),
        })
    }
    pub fn pump(&self) -> &Pump {
        &self.pump
    }
    pub fn can_accept_input(&self) -> bool {
        self.master.is_some() && self.status.is_none() && self.pump.can_accept_input()
    }
    pub fn queue_input(
        &mut self,
        input: crate::input::Input,
    ) -> Result<(), crate::input::InputError> {
        if self.master.is_none() || self.status.is_some() {
            return Err(crate::input::InputError::Closed);
        }
        self.pump.queue_input(input)
    }
    pub fn take_changed(&mut self) -> bool {
        self.pump.take_changed()
    }
    pub fn child_id(&self) -> u32 {
        self.child.id()
    }
    pub fn child_status(&self) -> Option<ExitStatus> {
        self.status
    }
    pub fn is_complete(&self) -> bool {
        self.status.is_some() && self.pump.is_finished()
    }
    /// Child exit does not imply EOF; EOF does not imply child exit.
    pub fn tick(&mut self, accept_effect: impl FnMut(&OutputEvent) -> bool) -> io::Result<Step> {
        self.status = self.child.try_wait()?;
        let master = self.master.as_mut().ok_or(io::ErrorKind::NotConnected)?;
        self.pump.step(master, accept_effect)
    }
    /// Prepare bounded state first; publish only after the OS accepts geometry.
    /// This initial experiment trades a full clone for atomic failure semantics.
    pub fn resize(&mut self, size: Size) -> io::Result<ResizeOutcome> {
        os::validate_size(size)?;
        let master = self.master.as_ref().ok_or(io::ErrorKind::NotConnected)?;
        let mut prepared = self.pump.terminal.clone();
        let outcome = prepared
            .resize(size)
            .map_err(|_| io::ErrorKind::InvalidInput)?;
        os::resize(master, size)?;
        self.pump.changed |= self.pump.terminal.size() != size;
        self.pump.terminal = prepared;
        Ok(outcome)
    }
    /// Explicit cancellation: close PTY, kill/reap the direct child. Outstanding
    /// input/output is abandoned by cancellation, not reported as a clean drain.
    /// wait() can block: execute shutdown on the future session worker, not UI.
    pub fn shutdown(&mut self) -> io::Result<ExitStatus> {
        self.master.take();
        if let Some(status) = self.status {
            return Ok(status);
        }
        if let Some(status) = self.child.try_wait()? {
            self.status = Some(status);
            return Ok(status);
        }
        self.child.kill()?;
        let status = self.child.wait()?;
        self.status = Some(status);
        Ok(status)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        // Child::drop alone does not reap. Explicit shutdown is preferable so
        // errors can be reported; this fallback prevents a normal abandoned child.
        let _ = self.shutdown();
    }
}
