use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChainAction {
    Notify,
}
