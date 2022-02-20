//! The key-value database with an expiration date.
//!
use async_std::{channel, prelude::*, stream, sync::RwLock, task};
use futures::{future::join_all, select, FutureExt};
use once_cell::sync::Lazy;
use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use std::time::{Duration, Instant};

/// The data base singleton.
pub(crate) static DB: Lazy<RwLock<DBManager>> = Lazy::new(|| {
    RwLock::new(DBManager {
        entries: HashMap::new(),
        expirations: BTreeMap::new(),
        expiration_id: 1,
        task_handles: Vec::new(),
    })
});

/// Key-value entries.
pub(crate) struct BDEntry {
    /// Value
    pub(crate) value: Vec<u8>,
    /// Expiration date
    pub(crate) expiration: Option<Instant>,
}
/// Database manager
pub(crate) struct DBManager {
    /// Key-value entries.
    entries: HashMap<Vec<u8>, BDEntry>,
    /// Map of entries with expiration dates.    
    expirations: BTreeMap<(Instant, u64), Vec<u8>>,
    /// ID to make the key unique.
    expiration_id: u64,
    /// Worker task handles.
    task_handles: Vec<task::JoinHandle<()>>,
}
/// Redis command option.
#[derive(PartialEq)]
pub(crate) enum SetCondition {
    NX,
    XX,
    GT,
    LT,
    NONE,
}

/// Helper function. Prepare to start the database.
pub(crate) async fn open(shutdown_event: channel::Receiver<crate::Void>) {
    DB.write().await.open(shutdown_event);
}

/// Helper function. Clean up the database.
pub(crate) async fn close() {
    DB.write().await.close().await;
}

/// Register expiration date to expirations map
/// If you make it a function, you'll get a borrowing error.
macro_rules! register_expiration {
    ($db:expr, $key:expr, $expiration:expr) => {
        if let Some(expiration) = $expiration {
            $db.expirations
                .insert((expiration, $db.expiration_id), $key);
                $db.expiration_id = $db.expiration_id + 1;
        }
    };
}

impl DBManager {
    
    /// Prepare to start the database.
    pub(self) fn open(&mut self, shutdown_event: channel::Receiver<crate::Void>) {
        let task_handle = task::spawn(Self::run(shutdown_event));
        self.task_handles.push(task_handle);
    }
    /// Clean up the database.
    pub(self) async fn close(&mut self) {
        // Wait for all worker tasks to end.
        join_all(&mut self.task_handles).await;
    }
    /// run worker.
    async fn run(mut shutdown_event: channel::Receiver<crate::Void>) {
        // The expiration entrys are checked every five seconds.
        let mut interval = stream::interval(Duration::from_secs(5));

        loop {
            select! {
                // Remove the expired entries.
                _ = interval.next().fuse() =>{
                    if DB.read().await.check_expired() {
                        DB.write().await.remove_expired();
                    }
                },
                // Wait for a shutdown.
                void = shutdown_event.next().fuse() => match void {
                    Some(void) => match void {},
                    None => break,
                },
            }
        }
    }
    /// Get the entry.
    pub(crate) fn get(&self, key: &Vec<u8>) -> Option<&BDEntry> {
        let entry = self.entries.get(key);

        if Self::expierd_opt(entry) {
            None
        } else {
            entry
        }
    }
    /// Get the value.
    pub(crate) fn get_value(&self, key: &Vec<u8>) -> Option<Vec<u8>> {
        let entry = self.entries.get(key);

        if Self::expierd_opt(entry) {
            None
        }
        else {
            match entry {
                Some(entry) => Some(entry.value.clone()),
                None => None,
            }
        }
    }
    /// Set value with options
    pub(crate) fn set(
        &mut self,
        key: Vec<u8>,
        value: Vec<u8>,
        expiration: Option<Instant>,
        set_condition: SetCondition,
        keep_ttl: bool,
        get_value: bool,
    ) -> Option<Vec<u8>> {
        match self.entries.entry(key) {
            Entry::Occupied(mut entry) => {
                let expierd = Self::expierd(entry.get());

                let old_value;
                if get_value && !expierd {
                    old_value = Some(entry.get().value.clone());
                } else {
                    old_value = None;
                }
                if set_condition != SetCondition::NX {
                    if keep_ttl && !expierd {
                        entry.get_mut().value = value;
                    } else {
                        //　Register expiration date.
                        register_expiration!(self, entry.key().clone(), expiration);
                        *entry.get_mut() = BDEntry { value, expiration };
                    }
                }
                return old_value;
            }
            Entry::Vacant(entry) => {
                if set_condition != SetCondition::XX {
                    //　Register expiration date.
                    register_expiration!(self, entry.key().clone(), expiration);
                    entry.insert(BDEntry { value, expiration });
                }
                None
            }
        }
    }
    /// Delete the entry.
    pub(crate) fn del(&mut self, key: Vec<u8>) -> bool {
        match self.entries.entry(key) {
            Entry::Occupied(entry) => {
                let mut deleted = false;
                if Self::expierd(entry.get()) {
                    deleted = false;
                }
                entry.remove();
                deleted
            }
            Entry::Vacant(_) => false,
        }
    }
    ///Make the expiration date indefinite.
    pub(crate) fn persist(&mut self, key: Vec<u8>) -> bool {
        match self.entries.entry(key) {
            Entry::Occupied(mut entry) => {
                if Self::expierd(entry.get()) {
                    false
                } else {
                    entry.get_mut().expiration = None;
                    true
                }
            }
            Entry::Vacant(_) => false,
        }
    }
    /// Append the value to the entry.
    pub(crate) fn append(&mut self, key: Vec<u8>, mut value: Vec<u8>) -> usize {
        match self.entries.entry(key) {
            Entry::Occupied(mut entry) => {
                let expierd = Self::expierd(entry.get());
                if expierd {
                    *entry.get_mut() = BDEntry {
                        value,
                        expiration: None,
                    };
                } else {
                    entry.get_mut().value.append(&mut value);
                }
                entry.get().value.len()
            }
            Entry::Vacant(entry) => {
                let len = value.len();
                entry.insert(BDEntry {
                    value,
                    expiration: None,
                });
                len
            }
        }
    }
    /// Get the value with options.
    pub(crate) fn getex(
        &mut self,
        key: Vec<u8>,
        expiration: Option<Instant>,
        persist: bool,
    ) -> Option<Vec<u8>> {
        match self.entries.entry(key) {
            Entry::Occupied(mut entry) => {
                let expierd = Self::expierd(entry.get());
                if expierd {
                    None
                } else {
                    if persist {
                        entry.get_mut().expiration = None;
                    } else {
                        // Register expiration date.
                        register_expiration!(self, entry.key().clone(), expiration);
                        if let Some(_) = expiration {
                            entry.get_mut().expiration = expiration;
                        }
                    }
                    Some(entry.get().value.clone())
                }
            }
            Entry::Vacant(_) => None,
        }
    }
    /// Set the expiration date for the entry.
    pub(crate) fn expire(
        &mut self,
        key: Vec<u8>,
        expiration: Option<Instant>,
        set_condition: SetCondition,
    ) -> bool {
        match self.entries.entry(key) {
            Entry::Occupied(mut entry) => {
                let expierd = Self::expierd(entry.get());
                if expierd {
                    false
                } else {
                    match set_condition {
                        SetCondition::NX => {
                            match entry.get().expiration {
                                Some(_) => false,
                                None => {
                                    // Register expiration date.
                                    register_expiration!(self, entry.key().clone(), expiration);
                                    entry.get_mut().expiration = expiration;
                                    true
                                }
                            }
                        }
                        SetCondition::XX => {
                            match entry.get().expiration {
                                Some(_) => {
                                    // Register expiration date.
                                    register_expiration!(self, entry.key().clone(), expiration);
                                    entry.get_mut().expiration = expiration;
                                    true
                                }
                                None => false,
                            }
                        }
                        SetCondition::GT => {
                            if expiration > entry.get().expiration {
                                // Register expiration date.
                                register_expiration!(self, entry.key().clone(), expiration);
                                entry.get_mut().expiration = expiration;
                                true
                            } else {
                                false
                            }
                        }
                        SetCondition::LT => {
                            if expiration < entry.get().expiration {
                                // Register expiration date.
                                register_expiration!(self, entry.key().clone(), expiration);
                                entry.get_mut().expiration = expiration;
                                true
                            } else {
                                false
                            }
                        }
                        SetCondition::NONE => {
                            // Register expiration date.
                            register_expiration!(self, entry.key().clone(), expiration);
                            entry.get_mut().expiration = expiration;
                            true
                        }
                    }
                }
            }
            Entry::Vacant(_) => false,
        }
    }
    /// Expired or not.
    fn expierd_opt(entry: Option<&BDEntry>) -> bool {
        if let Some(entry) = entry {
            return Self::expierd(entry);
        }
        false
    }
    /// Expired or not.
    fn expierd(entry: &BDEntry) -> bool {
        if let Some(expiration) = entry.expiration {
            expiration < Instant::now()
        } else {
            false
        }
    }
    /// Check if there are any expired entries.
    fn check_expired(&self) -> bool {
        if let Some((&(when, _), _)) = self.expirations.iter().next() {
            when < Instant::now()
        } else {
            false
        }
    }
    /// Remove the expired entries.
    fn remove_expired(&mut self) {
        while let Some((&(when, id), key)) = self.expirations.iter().next() {
            if when > Instant::now() {
                break;
            }
            match self.entries.entry(key.clone()) {
                Entry::Occupied(entry) => {
                    // Because the expiration date may have been updated.
                    if Self::expierd(entry.get()) {
                        entry.remove();
                    }
                }
                Entry::Vacant(_) => {}
            }
            self.expirations.remove(&(when, id));
        }
    }
}