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

use snarkvm::prelude::Network;

mod helpers;
pub use helpers::*;

mod routes;

use crate::monitor::Monitor;
use anyhow::Result;
use axum::{
    body::Body,
    extract::{ConnectInfo, DefaultBodyLimit, State},
    http::{header::CONTENT_TYPE, Method, Request, StatusCode},
    middleware,
    middleware::Next,
    response::Response,
    routing::post,
    Json,
};
use axum_extra::response::ErasedJson;
use parking_lot::Mutex;
use snarkvm::prelude::store::ConsensusStorage;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, task::JoinHandle};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{debug, info};

/// An additional Axum server for addon REST endpoints.
#[derive(Clone)]
pub struct MonitorRestService<N: Network, C: ConsensusStorage<N>> {
    /// The indexer service.
    monitor: Arc<Mutex<Monitor<N, C>>>,
    /// The server handles.
    handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl<N: Network, C: ConsensusStorage<N>> MonitorRestService<N, C> {
    /// Initializes a new instance of the server.
    pub async fn start(monitor: Monitor<N, C>, rest_ip: SocketAddr, rest_rps: u32) -> Result<Self> {
        // Initialize the server.
        let mut server = Self {
            monitor: Arc::new(Mutex::new(monitor)),
            handles: Default::default(),
        };
        // Spawn the server.
        server.spawn_server(rest_ip, rest_rps).await;
        // Return the server.
        Ok(server)
    }
}

impl<N: Network, C: ConsensusStorage<N>> MonitorRestService<N, C> {
    /// Returns the ledger.
    pub fn monitor(&self) -> Arc<Mutex<Monitor<N, C>>> {
        self.monitor.clone()
    }

    /// Returns the handles.
    pub fn handles(&self) -> &Arc<Mutex<Vec<JoinHandle<()>>>> {
        &self.handles
    }
}

impl<N: Network, C: ConsensusStorage<N>> MonitorRestService<N, C> {
    async fn spawn_server(&mut self, rest_ip: SocketAddr, rest_rps: u32) {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([CONTENT_TYPE]);

        // Log the REST rate limit per IP.
        debug!("REST rate limit per IP - {rest_rps} RPS");

        // Prepare the rate limiting setup.
        let governor_config = Box::new(
            GovernorConfigBuilder::default()
                .per_second(1)
                .burst_size(rest_rps)
                .error_handler(|error| Response::new(error.to_string().into()))
                .finish()
                .expect("Couldn't set up rate limiting for the REST server!"),
        );

        // Get the network being used.
        let network = match N::ID {
            snarkvm::console::network::MainnetV0::ID => "mainnet",
            snarkvm::console::network::TestnetV0::ID => "testnet",
            snarkvm::console::network::CanaryV0::ID => "canary",
            unknown_id => {
                eprintln!("Unknown network ID ({unknown_id})");
                return;
            }
        };

        let router = {
            let routes = axum::Router::new()
                // POST - job control.
                .route(
                    &format!("/{network}/subscribe"),
                    post(Self::start_subscription),
                )
                .route(&format!("/{network}/events"), post(Self::get_events));

            routes
                // Pass in `Rest` to make things convenient.
                .with_state(self.clone())
                // Enable tower-http tracing.
                .layer(TraceLayer::new_for_http())
                // Custom logging.
                .layer(middleware::from_fn(log_middleware))
                // Enable CORS.
                .layer(cors)
                // Cap body size at 512KiB.
                .layer(DefaultBodyLimit::max(512 * 1024))
                .layer(GovernorLayer {
                    // We can leak this because it is created only once and it persists.
                    config: Box::leak(governor_config),
                })
        };

        let rest_listener = TcpListener::bind(rest_ip).await.unwrap();
        self.handles.lock().push(tokio::spawn(async move {
            axum::serve(
                rest_listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .expect("couldn't start rest server");
        }))
    }
}

async fn log_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    info!(
        "Received '{} {}' from '{addr}'",
        request.method(),
        request.uri()
    );

    Ok(next.run(request).await)
}

/// Formats an ID into a truncated identifier (for logging purposes).
pub fn fmt_id(id: impl ToString) -> String {
    let id = id.to_string();
    let mut formatted_id = id.chars().take(16).collect::<String>();
    if id.chars().count() > 16 {
        formatted_id.push_str("..");
    }
    formatted_id
}
