use snarkvm::prelude::{const_assert, hrp2, AleoID, Field};

pub const DOMAIN_SEPARATOR: &str = "event-sub";

/// Type alias for a transfer job ID.
pub type SubscriptionID<N> = AleoID<Field<N>, { hrp2!("es") }>;
