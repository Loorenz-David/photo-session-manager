//! The sidecar protocol client over the supervisor (10E MC-31): writes commands, parses events,
//! and correlates each command with its outcome.

use super::{
    protocol::{Command, SidecarEvent, Stamped, parse_line},
    supervisor::{Supervisor, SupervisorEvent},
};
use std::{collections::HashMap, io, time::Instant};
use uuid::Uuid;

/// The `cmd_id` written on the wire.
pub type CmdId = String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandOutcome {
    Done,
    Rejected { reason: String },
    Error { error: String },
}

pub struct SidecarClient {
    supervisor: Supervisor,
    /// Commands sent and not yet `cmd.done`, by wire id.
    pending: HashMap<CmdId, Command>,
    /// Ids whose `cmd.received` arrived without a `cmd.done` yet, oldest first.
    open: Vec<CmdId>,
    outcomes: HashMap<CmdId, CommandOutcome>,
}

impl SidecarClient {
    pub fn new(supervisor: Supervisor) -> Self {
        Self {
            supervisor,
            pending: HashMap::new(),
            open: Vec::new(),
            outcomes: HashMap::new(),
        }
    }
    pub fn supervisor(&self) -> &Supervisor {
        &self.supervisor
    }
    pub fn supervisor_mut(&mut self) -> &mut Supervisor {
        &mut self.supervisor
    }
    /// Writes `command` as one stdin line. A command that carries a `cmd_id` and has none gets a
    /// fresh one; the returned id is the one written on the wire (empty for `credentials`/`quit`).
    pub fn send(&mut self, command: Command) -> io::Result<CmdId> {
        let command = match command.cmd_id() {
            Some("") => {
                let id = Uuid::new_v4().simple().to_string()[..8].to_owned();
                command.with_cmd_id(&id)
            }
            _ => command,
        };
        let id = command.cmd_id().unwrap_or_default().to_owned();
        self.supervisor.send_line(&command.encode())?;
        if !id.is_empty() {
            self.outcomes.remove(&id);
            self.pending.insert(id.clone(), command);
        }
        Ok(id)
    }
    /// Advances the supervisor and returns the events parsed from the sidecar's stdout, in order.
    /// Every other supervisor event goes to `side`; an unparsable line goes there once as
    /// `Stderr("unparsable sidecar line: …")` and is skipped.
    pub fn poll(&mut self, now: Instant, side: &mut dyn FnMut(SupervisorEvent)) -> Vec<Stamped> {
        let mut events = Vec::new();
        for event in self.supervisor.tick(now) {
            match event {
                SupervisorEvent::Line(line) => match parse_line(&line) {
                    Ok(stamped) => {
                        self.correlate(&stamped.event);
                        events.push(stamped);
                    }
                    Err(_) => side(SupervisorEvent::Stderr(format!(
                        "unparsable sidecar line: {line}"
                    ))),
                },
                other => side(other),
            }
        }
        events
    }
    /// The outcome of the command sent with `cmd_id`, once known.
    pub fn outcome(&self, cmd_id: &str) -> Option<CommandOutcome> {
        self.outcomes.get(cmd_id).cloned()
    }
    /// One command at a time on the real sidecar (`main.cpp:470–500`): `cmd.received {cmd_id}`,
    /// then `cmd.rejected`/`cmd.error` without an id, then `cmd.done {cmd_id}`. A rejection or
    /// error attaches to the most recent received id without a `cmd.done`; `cmd.done` yields
    /// `Done` only when nothing attached. Absent or empty ids are never tracked.
    fn correlate(&mut self, event: &SidecarEvent) {
        match event {
            SidecarEvent::CmdReceived {
                cmd_id: Some(id), ..
            } => {
                self.open.retain(|o| o != id);
                self.open.push(id.clone());
            }
            SidecarEvent::CmdRejected { reason, .. } => {
                if let Some(id) = self.open.last() {
                    self.outcomes
                        .entry(id.clone())
                        .or_insert(CommandOutcome::Rejected {
                            reason: reason.clone(),
                        });
                }
            }
            SidecarEvent::CmdError { error, .. } => {
                if let Some(id) = self.open.last() {
                    self.outcomes
                        .entry(id.clone())
                        .or_insert(CommandOutcome::Error {
                            error: error.clone(),
                        });
                }
            }
            SidecarEvent::CmdDone {
                cmd_id: Some(id), ..
            } => {
                self.open.retain(|o| o != id);
                self.pending.remove(id);
                self.outcomes
                    .entry(id.clone())
                    .or_insert(CommandOutcome::Done);
            }
            _ => {}
        }
    }
}
