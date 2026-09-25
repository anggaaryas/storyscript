//! Bounded semantic transcript, independent of the legacy TUI history.
use std::collections::VecDeque;

use crate::contract::{
    HARD_LIMITS, HistoryEntry, HistoryPage, LimitError, MediaEffect, PlayerLimits, SemanticEvent,
};

const MAX_PAGE_ENTRIES: usize = 256;

/// Uses UTF-8 payload bytes plus a fixed 64-byte per-entry accounting overhead.
/// The estimate is deterministic; it never transfers the transcript in a delta.
#[derive(Clone)]
pub struct HistoryBuffer {
    limits: PlayerLimits,
    entries: VecDeque<(HistoryEntry, usize)>,
    bytes: usize,
    next_sequence: u64,
    omitted_count: u64,
}

impl HistoryBuffer {
    pub fn new(limits: PlayerLimits) -> Result<Self, LimitError> {
        Ok(Self {
            limits: limits.lowered()?,
            entries: VecDeque::new(),
            bytes: 0,
            next_sequence: 0,
            omitted_count: 0,
        })
    }

    pub fn default_limits() -> Self {
        Self::new(HARD_LIMITS).expect("hard limits are valid")
    }

    /// Invalid oversized entries do not alter the buffer. Valid entries larger
    /// than the *lowered history cap* are omitted, like entries evicted later.
    pub fn append(
        &mut self,
        event: SemanticEvent,
        effects: Vec<MediaEffect>,
        scene: String,
    ) -> Result<u64, &'static str> {
        let event_bytes = event_bytes(&event);
        if event_bytes > self.limits.rendered_bytes
            || scene.len() > self.limits.rendered_bytes
            || effects
                .iter()
                .any(|effect| effect_bytes(effect) > self.limits.rendered_bytes)
        {
            return Err("event exceeds rendered byte limit");
        }
        let sequence = self.next_sequence;
        let next = sequence.checked_add(1).ok_or("history sequence overflow")?;
        let size = effects.iter().fold(
            64usize
                .saturating_add(event_bytes)
                .saturating_add(scene.len()),
            |total, effect| total.saturating_add(effect_bytes(effect)),
        );
        self.next_sequence = next;
        if size <= self.limits.history_bytes {
            self.bytes += size;
            self.entries.push_back((
                HistoryEntry {
                    sequence,
                    event,
                    effects,
                    scene,
                },
                size,
            ));
        } else {
            // A rolling transcript must be a suffix; never retain an older
            // entry while dropping a newer, unretainable one.
            self.omitted_count += self.entries.len() as u64;
            self.entries.clear();
            self.bytes = 0;
            self.omitted_count += 1;
        }
        while self.entries.len() > self.limits.history_entries
            || self.bytes > self.limits.history_bytes
        {
            let (_, discarded) = self.entries.pop_front().expect("eviction has an entry");
            self.bytes -= discarded;
            self.omitted_count += 1;
        }
        Ok(sequence)
    }

    pub fn next_sequence(&self) -> u64 {
        self.next_sequence
    }
    pub fn first_retained_sequence(&self) -> u64 {
        self.entries
            .front()
            .map_or(self.next_sequence, |(entry, _)| entry.sequence)
    }
    pub fn omitted_history_count(&self) -> u64 {
        self.omitted_count
    }
    pub fn retained_bytes(&self) -> usize {
        self.bytes
    }

    pub(crate) fn retained_entries(&self) -> Vec<HistoryEntry> {
        self.entries
            .iter()
            .map(|(entry, _)| entry.clone())
            .collect()
    }

    pub(crate) fn restore(
        limits: PlayerLimits,
        entries: Vec<HistoryEntry>,
        next_sequence: u64,
        omitted_count: u64,
    ) -> Result<Self, &'static str> {
        let limits = limits.lowered().map_err(|_| "invalid resource limits")?;
        let mut restored = Self {
            limits,
            entries: VecDeque::new(),
            bytes: 0,
            next_sequence,
            omitted_count,
        };
        let mut previous = None;
        for entry in entries {
            if previous.is_some_and(|sequence| sequence >= entry.sequence)
                || entry.sequence >= next_sequence
            {
                return Err("invalid history sequence");
            }
            let size = entry_size(&entry, limits)?;
            restored.bytes = restored
                .bytes
                .checked_add(size)
                .ok_or("history size overflow")?;
            restored.entries.push_back((entry.clone(), size));
            previous = Some(entry.sequence);
        }
        if restored.entries.len() > limits.history_entries || restored.bytes > limits.history_bytes
        {
            return Err("history exceeds configured limits");
        }
        if restored.first_retained_sequence() < omitted_count {
            return Err("invalid omitted history count");
        }
        Ok(restored)
    }

    pub fn page(&self, from_sequence: u64, maximum: usize) -> HistoryPage {
        let start = from_sequence.max(self.first_retained_sequence());
        let entries: Vec<_> = self
            .entries
            .iter()
            .filter(|(entry, _)| entry.sequence >= start)
            .take(maximum.min(MAX_PAGE_ENTRIES))
            .map(|(entry, _)| entry.clone())
            .collect();
        let next_sequence = entries
            .last()
            .map_or(start.min(self.next_sequence), |entry| entry.sequence + 1);
        HistoryPage {
            entries,
            next_sequence,
            first_retained_sequence: self.first_retained_sequence(),
            omitted_history_count: self.omitted_count,
        }
    }
}

fn entry_size(entry: &HistoryEntry, limits: PlayerLimits) -> Result<usize, &'static str> {
    let event_size = event_bytes(&entry.event);
    if event_size > limits.rendered_bytes
        || entry.scene.len() > limits.rendered_bytes
        || entry
            .effects
            .iter()
            .any(|effect| effect_bytes(effect) > limits.rendered_bytes)
    {
        return Err("event exceeds rendered byte limit");
    }
    Ok(entry.effects.iter().fold(
        64usize
            .saturating_add(event_size)
            .saturating_add(entry.scene.len()),
        |total, effect| total.saturating_add(effect_bytes(effect)),
    ))
}

fn effect_bytes(effect: &MediaEffect) -> usize {
    match effect {
        MediaEffect::Background(path) | MediaEffect::Bgm(path) | MediaEffect::Sfx(path) => {
            path.len() + 8
        }
        MediaEffect::BgmStop => 8,
    }
}

fn event_bytes(event: &SemanticEvent) -> usize {
    match event {
        SemanticEvent::SceneTransition(scene) | SemanticEvent::Narration(scene) => {
            scene.len().saturating_add(8)
        }
        SemanticEvent::Dialogue {
            actor_id,
            actor_name,
            emotion,
            position,
            portrait_path,
            text,
        } => [
            actor_id.len(),
            actor_name.len(),
            text.len(),
            emotion.as_ref().map_or(0, String::len),
            position.as_ref().map_or(0, String::len),
            portrait_path.as_ref().map_or(0, String::len),
            24,
        ]
        .into_iter()
        .fold(0usize, usize::saturating_add),
        SemanticEvent::Choices(options) => options.iter().fold(8usize, |total, choice| {
            total
                .saturating_add(choice.text.len())
                .saturating_add(choice.target_scene.len())
                .saturating_add(8)
        }),
        SemanticEvent::Media(effect) => effect_bytes(effect).saturating_add(8),
        SemanticEvent::End => 8,
        SemanticEvent::Error(error) => [
            error.code.len(),
            error.scene.len(),
            error.message.len(),
            error.resource.as_ref().map_or(0, String::len),
            48,
        ]
        .into_iter()
        .fold(0usize, usize::saturating_add),
    }
}
