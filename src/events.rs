use crate::action::ChainAction;
use snarkvm::prelude::{Identifier, Network, Plaintext, ProgramID};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound(serialize = "N: Serialize", deserialize = "N: for<'a> Deserialize<'a>"))]
pub struct EventManifest<N: Network> {
    pub name: String,
    pub description: String,
    pub function: Identifier<N>,
    pub program_id: ProgramID<N>,
    pub inputs: Option<IndexMap<usize, Plaintext<N>>>,
    pub outputs: Option<IndexMap<usize, Plaintext<N>>>,
    pub actions: Vec<ChainAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound(serialize = "N: Serialize", deserialize = "N: for<'a> Deserialize<'a>"))]
pub struct EventManifests<N: Network> {
    manifests: Vec<EventManifest<N>>,
}

impl<N: Network> EventManifests<N> {
    pub fn manifests(&self) -> &Vec<EventManifest<N>> {
        &self.manifests
    }
}
