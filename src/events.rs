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
    pub program: ProgramID<N>,
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
    pub fn new(manifests: Vec<EventManifest<N>>) -> EventManifests<N> {
        Self { manifests }
    }

    pub fn manifests(&self) -> &Vec<EventManifest<N>> {
        &self.manifests
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::prelude::MainnetV0;
    use std::str::FromStr;

    #[test]
    fn test_item() {
        let manifest = EventManifest {
            name: "transferPublics".to_string(),
            description: "Find transfer publics".to_string(),
            function: Identifier::from_str("transfer_public").unwrap(),
            program: ProgramID::<MainnetV0>::from_str("credits.aleo").unwrap(),
            inputs: None,
            outputs: None,
            actions: vec![ChainAction::Notify],
        };
        let json_manifest = serde_json::to_string(&manifest).unwrap();

        println!("{json_manifest:?}");
        let manifests = EventManifests::new(vec![manifest]);
        let manifests = serde_json::to_string(&manifests).unwrap();
        println!("{manifests:?}");
    }
}
