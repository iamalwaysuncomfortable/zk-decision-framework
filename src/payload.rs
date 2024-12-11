use snarkvm::prelude::{Identifier, Network, Plaintext, ProgramID};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound(serialize = "N: Serialize", deserialize = "N: for<'a> Deserialize<'a>"))]
pub struct EventPayLoad<N: Network> {
    // The event type.
    event_type: String,
    // User specified context.
    context: String,
    // Program executed.
    program: ProgramID<N>,
    // Block height execution was found at.
    block_height: u32,
    // Function executed.
    function_id: Identifier<N>,
    // Transaction ID execution was found at.
    transaction: N::TransactionID,
    // Transition ID execution was found at.
    transition: N::TransitionID,
    // Inputs triggered.
    inputs: Option<IndexMap<u32, Plaintext<N>>>,
    // Outputs triggered.
    outputs: Option<IndexMap<u32, Plaintext<N>>>,
}

impl<N: Network> EventPayLoad<N> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_type: String,
        context: String,
        program: ProgramID<N>,
        block_height: u32,
        function_id: Identifier<N>,
        transaction: N::TransactionID,
        transition: N::TransitionID,
        inputs: Option<IndexMap<u32, Plaintext<N>>>,
        outputs: Option<IndexMap<u32, Plaintext<N>>>,
    ) -> EventPayLoad<N> {
        EventPayLoad {
            event_type,
            context,
            program,
            block_height,
            function_id,
            transaction,
            transition,
            inputs,
            outputs,
        }
    }
}
