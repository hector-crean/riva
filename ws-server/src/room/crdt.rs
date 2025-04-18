use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use ts_rs::TS;

// Last-Write-Wins Element Set CRDT
#[derive(Clone, Debug, TS, Serialize, Deserialize, Default)]
#[ts(export)]
pub struct LWWElementSet<T: Clone + Eq + std::hash::Hash + TS> {
    additions: HashMap<T, u64>,
    removals: HashMap<T, u64>,
}

impl<T: Clone + Eq + std::hash::Hash + Serialize + for<'de> Deserialize<'de> + TS> LWWElementSet<T> {
    #[must_use] pub fn new() -> Self {
        Self {
            additions: HashMap::new(),
            removals: HashMap::new(),
        }
    }

    pub fn add(&mut self, element: T, timestamp: u64) {
        self.additions.insert(element, timestamp);
    }

    pub fn remove(&mut self, element: T, timestamp: u64) {
        self.removals.insert(element, timestamp);
    }

    pub fn contains(&self, element: &T) -> bool {
        match (self.additions.get(element), self.removals.get(element)) {
            (Some(add_time), Some(remove_time)) => add_time >= remove_time,
            (Some(_), None) => true,
            _ => false,
        }
    }

    #[must_use] pub fn elements(&self) -> HashSet<T> {
        self.additions
            .iter()
            .filter_map(|(element, add_time)| {
                match self.removals.get(element) {
                    Some(remove_time) if remove_time >= add_time => None,
                    _ => Some(element.clone()),
                }
            })
            .collect()
    }

    pub fn merge(&mut self, other: &Self) {
        for (element, timestamp) in &other.additions {
            match self.additions.get(element) {
                Some(current) if current >= timestamp => {}
                _ => {
                    self.additions.insert(element.clone(), *timestamp);
                }
            }
        }

        for (element, timestamp) in &other.removals {
            match self.removals.get(element) {
                Some(current) if current >= timestamp => {}
                _ => {
                    self.removals.insert(element.clone(), *timestamp);
                }
            }
        }
    }
}

// Grow-only Counter CRDT
#[derive(Clone, Debug, TS, Serialize, Deserialize, Default)]
#[ts(export)]
pub struct GCounter {
    counters: HashMap<String, u64>,
}

impl GCounter {
    #[must_use] pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    pub fn increment(&mut self, client_id: &str, amount: u64) {
        let counter = self.counters.entry(client_id.to_string()).or_insert(0);
        *counter += amount;
    }

    #[must_use] pub fn value(&self) -> u64 {
        self.counters.values().sum()
    }

    pub fn merge(&mut self, other: &Self) {
        for (client_id, &count) in &other.counters {
            let entry = self.counters.entry(client_id.clone()).or_insert(0);
            *entry = std::cmp::max(*entry, count);
        }
    }
} 