// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Real-time communication between the control thread and the audio thread (06 §2, §4).
//!
//! [`realtime`] splits a [`Graph`] into the two halves of the A02 contract (F-036):
//!
//! - [`Controller`], on a non-real-time thread: sends [`Command`]s through a wait-free
//!   `rtrb` ring (capacity 1024), swaps in new plans as `basedrop::Owned` graphs through a
//!   second ring, reads the latest [`EngineState`] from a `triple_buffer`, and runs the
//!   `basedrop` collector that frees retired graphs.
//! - [`Processor`], moved to the audio thread: drains at most
//!   [`MAX_COMMANDS_PER_BLOCK`] commands per block, applies a plan swap at the block
//!   boundary with a crossfade, renders, and publishes its state.
//!
//! **Overflow policy.** If the command ring is full, the controller keeps the command in a
//! local queue and coalesces it with queued commands of the same kind (the latest seek,
//! speed, loop or level wins); it never blocks the audio thread. Overflows are counted.

use std::time::Duration;

use basedrop::{Collector, Handle, Owned};
use rtrb::{Consumer, Producer, RingBuffer};
use sonarithm_dsp::env::ramp_frames;
use triple_buffer::{Input, Output, triple_buffer};

use crate::graph::{Command, Graph, GraphStats, PAUSE_FADE};

/// Command ring capacity (06 §4).
pub const COMMAND_CAPACITY: usize = 1024;
/// Commands applied per block at most, so a burst cannot stall the audio thread
/// (coding-standards §11.1, unbounded work).
pub const MAX_COMMANDS_PER_BLOCK: usize = 64;
/// Plan swaps waiting at most. A swap takes effect at the next block, so two are plenty.
const SWAP_CAPACITY: usize = 2;

/// The engine state the audio thread publishes after every block (06 §4).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct EngineState {
    /// Transport, voice and level counters of the current graph.
    pub graph: GraphStats,
    /// The backend's playback timestamp for the block, if it gave one: the anchor for the
    /// UI playhead (06 §9).
    pub host_time: Option<Duration>,
    /// Blocks processed.
    pub blocks: u64,
    /// Underruns the backend reported.
    pub xruns: u64,
    /// Plan swaps applied.
    pub swaps: u64,
}

/// The control half: lives on a non-real-time thread and may allocate.
pub struct Controller {
    commands: Producer<Command>,
    swaps: Producer<Owned<Graph>>,
    state: Output<EngineState>,
    collector: Collector,
    handle: Handle,
    /// Commands that did not fit in the ring, coalesced.
    pending: Vec<Command>,
    overflows: u64,
}

/// The audio half: moved to the audio thread. Every method is RT-safe.
pub struct Processor {
    commands: Consumer<Command>,
    swaps: Consumer<Owned<Graph>>,
    state: Input<EngineState>,
    graph: Owned<Graph>,
    /// The graph being replaced, fading out, with the frames of fade left.
    retiring: Option<(Owned<Graph>, u64)>,
    scratch: Vec<f32>,
    published: EngineState,
}

/// Splits `graph` into its control and audio halves.
#[must_use]
pub fn realtime(graph: Graph) -> (Controller, Processor) {
    let collector = Collector::new();
    let handle = collector.handle();
    let (commands_tx, commands_rx) = RingBuffer::new(COMMAND_CAPACITY);
    let (swaps_tx, swaps_rx) = RingBuffer::new(SWAP_CAPACITY);
    let (state_tx, state_rx) = triple_buffer(&EngineState::default());
    let scratch = vec![0.0; 2 * graph.max_block()];
    let processor = Processor {
        commands: commands_rx,
        swaps: swaps_rx,
        state: state_tx,
        graph: Owned::new(&handle, graph),
        retiring: None,
        scratch,
        published: EngineState::default(),
    };
    let controller = Controller {
        commands: commands_tx,
        swaps: swaps_tx,
        state: state_rx,
        collector,
        handle,
        pending: Vec::new(),
        overflows: 0,
    };
    (controller, processor)
}

/// Two commands of the same kind (and target) replace each other in the overflow queue.
fn same_kind(a: &Command, b: &Command) -> bool {
    match (a, b) {
        (Command::SetGain(x, _), Command::SetGain(y, _)) => x == y,
        (Command::Seek(_), Command::Seek(_))
        | (Command::SetSpeed(_), Command::SetSpeed(_))
        | (Command::SetLoop(_), Command::SetLoop(_)) => true,
        _ => false,
    }
}

impl Controller {
    /// Sends a command. Never blocks: when the ring is full, the command waits in a local
    /// queue, replacing an earlier queued command of the same kind, until
    /// [`Controller::flush`] can send it.
    pub fn send(&mut self, command: Command) {
        self.flush();
        if !self.pending.is_empty() {
            self.queue(command);
            return;
        }
        if let Err(rtrb::PushError::Full(command)) = self.commands.push(command) {
            self.queue(command);
        }
    }

    fn queue(&mut self, command: Command) {
        self.overflows += 1;
        if let Some(slot) = self
            .pending
            .iter_mut()
            .find(|queued| same_kind(queued, &command))
        {
            // Keep the queue position, take the newest value (latest seek wins).
            *slot = command;
        } else {
            self.pending.push(command);
        }
    }

    /// Sends queued commands, in order, while the ring has room. Returns how many are
    /// still waiting. Call it on every control tick.
    pub fn flush(&mut self) -> usize {
        let sendable = self.commands.slots().min(self.pending.len());
        for command in self.pending.drain(..sendable) {
            // Cannot fail: `slots()` reported room for each.
            let _ = self.commands.push(command);
        }
        self.pending.len()
    }

    /// Commands that did not fit in the ring when sent.
    #[must_use]
    pub fn overflows(&self) -> u64 {
        self.overflows
    }

    /// Hands a new graph to the audio thread, which swaps it in at the next block with a
    /// crossfade and keeps the transport state.
    ///
    /// # Errors
    ///
    /// Returns the graph, boxed, if two swaps are already waiting; try again after the
    /// audio thread has taken them.
    pub fn swap(&mut self, graph: Graph) -> Result<(), Box<Graph>> {
        if self.swaps.is_full() {
            return Err(Box::new(graph));
        }
        // Only this producer fills the ring and the consumer only frees slots, so the push
        // cannot fail after `is_full` returned false.
        let _ = self.swaps.push(Owned::new(&self.handle, graph));
        Ok(())
    }

    /// The latest engine state the audio thread published.
    pub fn state(&mut self) -> EngineState {
        *self.state.read()
    }

    /// Frees graphs the audio thread has retired. Call it on every control tick.
    pub fn collect(&mut self) {
        self.collector.collect();
    }

    /// Allocations the collector still tracks: the live graphs plus retired ones not yet
    /// collected.
    #[must_use]
    pub fn live_allocations(&self) -> usize {
        self.collector.alloc_count()
    }
}

impl Processor {
    /// The sample rate of the current plan.
    #[must_use]
    pub fn sample_rate(&self) -> sonarithm_core::SampleRate {
        self.graph.plan().sample_rate
    }

    /// Counts `count` underruns reported by the backend.
    ///
    /// # Real-time safety
    ///
    /// RT-safe.
    pub fn report_xruns(&mut self, count: u64) {
        self.published.xruns = self.published.xruns.saturating_add(count);
    }

    /// Applies pending commands and plan swaps, renders the next `out.len() / 2` frames of
    /// interleaved stereo, and publishes the engine state.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: no allocation or deallocation (retired graphs go to the collector), no
    /// locks, no I/O; at most [`MAX_COMMANDS_PER_BLOCK`] commands per block.
    pub fn process(&mut self, out: &mut [f32], host_time: Option<Duration>) {
        for _ in 0..MAX_COMMANDS_PER_BLOCK {
            let Ok(command) = self.commands.pop() else {
                break;
            };
            self.graph.apply(command);
        }
        if let Ok(mut incoming) = self.swaps.pop() {
            incoming.adopt(&self.graph);
            let mut outgoing = core::mem::replace(&mut self.graph, incoming);
            outgoing.apply(Command::StopAll);
            let fade = u64::from(ramp_frames(PAUSE_FADE, outgoing.plan().sample_rate));
            // Dropping a still-retiring graph only queues it for the collector.
            self.retiring = Some((outgoing, fade + 1));
            self.published.swaps += 1;
        }
        self.graph.process(out);
        self.mix_retiring(out);
        self.published.graph = self.graph.stats();
        self.published.host_time = host_time;
        self.published.blocks += 1;
        self.state.write(self.published);
    }

    /// Adds the fading-out graph of a plan swap, then retires it once its fade is over.
    fn mix_retiring(&mut self, out: &mut [f32]) {
        let Some((graph, remaining)) = &mut self.retiring else {
            return;
        };
        for chunk in out.chunks_mut(self.scratch.len().max(2)) {
            let Some(scratch) = self.scratch.get_mut(..chunk.len()) else {
                break;
            };
            graph.process(scratch);
            for (sample, extra) in chunk.iter_mut().zip(scratch.iter()) {
                *sample += extra;
            }
        }
        let frames = (out.len() / 2) as u64;
        *remaining = remaining.saturating_sub(frames);
        if *remaining == 0 {
            self.retiring = None;
        }
    }

    /// Mutable access to the graph, for tests and offline drivers that apply commands
    /// directly.
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    /// Applies a command directly, bypassing the ring (offline drivers).
    ///
    /// # Real-time safety
    ///
    /// RT-safe.
    pub fn apply(&mut self, command: Command) {
        self.graph.apply(command);
    }
}

#[cfg(test)]
mod tests;
