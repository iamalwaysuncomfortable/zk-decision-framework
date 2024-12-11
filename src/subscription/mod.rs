use snarkvm::prelude::{Field, Network, Uniform};

use anyhow::Result;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

pub mod id;
use crate::events::EventManifest;
use crate::EventManifests;
pub use id::{SubscriptionID, DOMAIN_SEPARATOR};

/// An object representing an individual job.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound(serialize = "N: Serialize", deserialize = "N: for<'a> Deserialize<'a>"))]
pub struct Subscription<N: Network> {
    id: SubscriptionID<N>,
    events: EventManifests<N>,
}

impl<N: Network> Subscription<N> {
    /// Create a new ETL job.
    pub fn new(events: EventManifests<N>) -> Result<Self> {
        // Create a nonce and domains separator for the etl job.
        let nonce = N::hash_psd4(&[Field::<N>::rand(&mut thread_rng())])?;
        let domain_separator = Field::new_domain_separator(DOMAIN_SEPARATOR);

        // Create a unique ID for the transfer job.
        let id = SubscriptionID::from(N::hash_psd4(&[nonce, domain_separator])?);

        Ok(Self { id, events })
    }
}

impl<N: Network> Subscription<N> {
    /// Get the ID of the job.
    pub fn id(&self) -> &SubscriptionID<N> {
        &self.id
    }

    /// Get the type of the job.
    pub fn events(&self) -> &Vec<EventManifest<N>> {
        self.events.manifests()
    }
}

impl<N: Network> Display for Subscription<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{\"id:\" {}, \":\" {:?}}}", self.id, self.events)
    }
}
