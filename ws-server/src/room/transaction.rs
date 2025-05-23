use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use ts_rs::TS;
use uuid::Uuid;

use crate::message::ClientMessage;

use super::presence::PresenceLike;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct Transaction<Presence: PresenceLike + TS> {
    pub id: String,
    pub client_id: String,
    pub timestamp: u64,
    pub msg: ClientMessage<Presence>,
}

impl<Presence: PresenceLike + TS> Transaction<Presence> {
    #[must_use]
    pub fn new(client_id: String, msg: ClientMessage<Presence>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            msg,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TransactionManager<Presence: PresenceLike + TS> {
    history: VecDeque<Transaction<Presence>>,
    undone: VecDeque<Transaction<Presence>>,
    max_history: usize,
}

impl<Presence: PresenceLike + TS> TransactionManager<Presence> {
    #[must_use]
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            undone: VecDeque::new(),
            max_history,
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction<Presence>) {
        // Clear the undo stack when a new transaction is added
        self.undone.clear();

        // Add the transaction to history
        self.history.push_back(transaction);

        // Maintain history size limit
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    pub fn undo(&mut self) -> Option<Transaction<Presence>> {
        if let Some(transaction) = self.history.pop_back() {
            self.undone.push_back(transaction.clone());
            Some(transaction)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<Transaction<Presence>> {
        if let Some(transaction) = self.undone.pop_back() {
            self.history.push_back(transaction.clone());
            Some(transaction)
        } else {
            None
        }
    }

    #[must_use]
    pub fn get_history(&self) -> &VecDeque<Transaction<Presence>> {
        &self.history
    }
}
