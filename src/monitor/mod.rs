use crate::{EventPayLoad, Subscription, SubscriptionID};
use indexmap::IndexMap;
use parking_lot::Mutex;
use snarkvm::ledger::store::ConsensusStorage;
use snarkvm::ledger::Ledger;
use snarkvm::prelude::{Itertools, Network};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::info;

#[derive(Clone)]
pub struct Monitor<N: Network, C: ConsensusStorage<N>> {
    ledger: Ledger<N, C>,
    latest_block: Arc<AtomicU32>,
    subscriptions: Arc<Mutex<Vec<Subscription<N>>>>,
    #[allow(clippy::type_complexity)]
    matching_events: Arc<Mutex<IndexMap<SubscriptionID<N>, Vec<EventPayLoad<N>>>>>,
    join_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl<N: Network, C: ConsensusStorage<N>> Monitor<N, C> {
    /// Create a new monitor object.
    pub fn new(ledger: Ledger<N, C>) -> Self {
        let latest_block = ledger.latest_height();
        Self {
            ledger,
            latest_block: Arc::new(AtomicU32::new(latest_block)),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            matching_events: Arc::new(Mutex::new(IndexMap::new())),
            join_handles: Arc::new(Mutex::new(Default::default())),
        }
    }

    /// Get the ledger.
    pub fn ledger(&self) -> &Ledger<N, C> {
        &self.ledger
    }

    /// Add subscription.
    pub fn add(&mut self, subscription: Subscription<N>) {
        info!("Adding subscription {subscription:?}");
        self.matching_events
            .lock()
            .insert(*subscription.id(), vec![]);
        self.subscriptions.lock().push(subscription)
    }

    /// Drain subscriptions.
    pub fn drain(&mut self, id: SubscriptionID<N>) -> (SubscriptionID<N>, Vec<EventPayLoad<N>>) {
        info!("Getting all events for subscription {id}");
        match self.matching_events.lock().get_full_mut(&id) {
            Some((_, id, events)) => {
                let num_events = events.len();
                info!("{num_events} events found");
                (*id, events.drain(0..num_events).collect_vec())
            }
            None => (id, vec![]),
        }
    }

    /// Start the monitor.
    pub async fn start_monitor(&self) {
        let self_ = self.clone();
        let task = tokio::task::spawn(async move {
            loop {
                let latest_ledger_height = self_.ledger.latest_height();
                let latest_tracked_block = self_.latest_block.load(Ordering::Relaxed);
                info!("Latest ledger height {latest_ledger_height} latest tracked block {latest_tracked_block}");
                if self_.ledger.latest_height() > self_.latest_block.load(Ordering::Relaxed) {
                    for height in (latest_tracked_block + 1)..(latest_ledger_height + 1) {
                        info!("Getting events for height {height}");
                        for subscription in self_.subscriptions.lock().iter() {
                            let subscription_id = subscription.id();
                            info!("Getting events for subscription id {subscription_id}");
                            let transactions = self_.ledger.get_transactions(height).unwrap();
                            for transaction in transactions.iter() {
                                info!("Transaction {transaction:?} being searched");
                                for transition in transaction.transitions() {
                                    for event in subscription.events().iter() {
                                        let program_id = &event.program;
                                        let function_name = &event.function;
                                        let transition_program = transition.program_id();
                                        let transition_function = transition.function_name();
                                        info!("Search program: {program_id}-Transition program: {transition_program}\nSearch function:{function_name}-Transition function:{transition_function}");
                                        if transition.program_id() == program_id
                                            && transition.function_name() == function_name
                                        {
                                            let payload = EventPayLoad::new(
                                                event.name.clone(),
                                                event.description.clone(),
                                                *program_id,
                                                height,
                                                *function_name,
                                                transaction.id(),
                                                *transition.id(),
                                                None,
                                                None,
                                            );
                                            let subscription_id = subscription.id();
                                            if let Some(payloads) = self_
                                                .matching_events
                                                .lock()
                                                .get_mut(subscription_id)
                                            {
                                                info!("Adding event {payload:?} to subscription {subscription_id}");
                                                payloads.push(payload);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        self_.latest_block.store(height, Ordering::Relaxed);
                    }
                }
                sleep(Duration::from_millis(200)).await
            }
        });
        self.join_handles.lock().push(task);
    }
}
