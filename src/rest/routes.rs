// Copyright 2024 Aleo Network Foundation
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:

// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

use crate::{EventManifests, Subscription, SubscriptionID};
use serde_json::json;

impl<N: Network, C: ConsensusStorage<N>> MonitorRestService<N, C> {
    /// POST /<network>/subscribe
    pub(crate) async fn start_subscription(
        State(rest): State<Self>,
        Json(manifest): Json<EventManifests<N>>,
    ) -> Result<ErasedJson, RestError> {
        // Start a new job.
        debug!("POST /subscribe");
        let subscription = Subscription::new(manifest)?;
        let subscription_id = *subscription.id();
        rest.monitor.lock().add(subscription);
        Ok(ErasedJson::pretty(
            json!({"status": "event monitor started", "subscription_id": subscription_id}),
        ))
    }

    /// POST /<network>/events
    pub(crate) async fn get_events(
        State(rest): State<Self>,
        Json(id): Json<SubscriptionID<N>>,
    ) -> Result<ErasedJson, RestError> {
        let (id, events) = rest.monitor.lock().drain(id);
        Ok(ErasedJson::pretty(
            json!({"subscription": id, "events": events}),
        ))
    }
}
