//! Receive path: splits S→C chunks with `d2-proto`'s size rule and
//! dispatches each message by id. Unowned ids and discarded bytes are
//! recorded, never interpreted. A message whose id has a unit handler is
//! appended to the addressed unit's queue when that unit is in the model,
//! else dropped; the update pass applies it.
//!
use std::collections::BTreeMap;

use d2_proto::transport::{split_server_buffer, SplitError};

use super::dispatch::{Dispatch, Handle, HandlerError, Message};
use super::output::{move_freed, Output, Outputs};
use super::world::{addressed_unit, ClientWorld, ModelInputs};

/// Bytes a split discarded: everything after the first id the size rule
/// refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Discarded {
    /// The first discarded byte: the id the size rule refused.
    pub first: u8,
    pub bytes: usize,
}

/// A handler's refusal: the message's effect is dropped.
#[derive(Debug, PartialEq, Eq)]
pub struct Rejected {
    pub id: u8,
    pub error: HandlerError,
}

/// Everything the receive path did not apply to the model.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ReceiveLog {
    /// Messages applied by a handler.
    pub handled: u64,
    /// Messages without a handler, counted per id.
    pub unowned: BTreeMap<u8, u64>,
    /// Unit-handler messages appended to a unit's queue.
    pub queued: u64,
    /// Unit-handler messages whose unit is not in the model, per id
    /// (dropped).
    pub dropped: BTreeMap<u8, u64>,
    /// Queued messages applied by the update pass.
    pub drained: u64,
    pub discarded: Vec<Discarded>,
    pub rejected: Vec<Rejected>,
}

/// What one chunk did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChunkReport {
    pub messages: usize,
    pub handled: usize,
    pub queued: usize,
    pub dropped: usize,
    pub unowned: usize,
    pub rejected: usize,
    pub discarded_bytes: usize,
}

/// Splits `chunk` and dispatches its messages in order. A chunk the split
/// cannot take (a message too large or running past the chunk) is refused
/// whole: nothing in it is dispatched. The handlers' outputs are appended
/// to `outputs` in message order; a rejected message's outputs are dropped
/// with its effect. On a mod wire, mod messages go to `mods.payloads` and
/// count as handled.
pub fn receive_chunk(
    world: &mut ClientWorld,
    inputs: &ModelInputs,
    dispatch: &Dispatch,
    log: &mut ReceiveLog,
    outputs: &mut Vec<Output>,
    chunk: &[u8],
) -> Result<ChunkReport, SplitError> {
    let split = split_server_buffer(chunk)?;
    let mut report = ChunkReport {
        messages: split.messages.len(),
        ..ChunkReport::default()
    };
    for bytes in split.messages {
        let sink = Outputs::default();
        let msg = Message {
            id: bytes[0],
            bytes,
            unit: addressed_unit(bytes),
            inputs,
            out: &sink,
        };
        match dispatch.get(msg.id).map(|e| e.handle) {
            None => {
                *log.unowned.entry(msg.id).or_default() += 1;
                report.unowned += 1;
            }
            Some(Handle::Unit(_)) => match msg.unit.and_then(|k| world.units.get_mut(&k)) {
                Some(unit) => {
                    unit.queue.push(bytes.to_vec());
                    log.queued += 1;
                    report.queued += 1;
                }
                None => {
                    *log.dropped.entry(msg.id).or_default() += 1;
                    report.dropped += 1;
                }
            },
            Some(Handle::General(handle)) => match handle(world, &msg) {
                Ok(()) => {
                    log.handled += 1;
                    report.handled += 1;
                    outputs.extend(sink.take());
                    move_freed(world, outputs);
                }
                Err(error) => {
                    log.rejected.push(Rejected { id: msg.id, error });
                    report.rejected += 1;
                    // A unit the handler freed before failing is gone
                    // from the model: its free is still applied.
                    move_freed(world, outputs);
                }
            },
        }
    }
    if let Some(&first) = split.discarded.first() {
        log.discarded.push(Discarded {
            first,
            bytes: split.discarded.len(),
        });
        report.discarded_bytes = split.discarded.len();
    }
    Ok(report)
}
