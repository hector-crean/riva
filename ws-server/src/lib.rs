pub mod database;
pub mod error;
pub mod file_storage;
pub mod handlers;
pub mod message;
pub mod message_broker;
pub mod presentation;
pub mod request_client;
pub mod room;

use database::{Database, surrealdb::SurrealDatabase};
use message_broker::{MessageBroker, socket_io::SocketIoMessageBroker};
use presentation::Presentation;

use crate::room::RoomLike;
use axum::routing::{delete, get, post, put};
use error::ServerError;
use file_storage::{FileStorage, s3::S3Bucket};
use message::{ClientMessage, ClientMessageType};
use room::{room_id::RoomId, room_manager::RoomManager};
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef, State},
    socket::DisconnectReason,
};
use std::sync::Arc;
use std::{collections::HashMap, future::Future, net::SocketAddr};
use surrealdb::{Surreal, engine::remote::ws::Ws, opt::auth::Root};
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

// Define the server state to be shared across handlers

#[derive(Clone)]
pub struct ApplicationConfig {
    pub aws_key: String,
    pub aws_key_secret: String,
    pub s3_region: String,
    pub aws_bucket: String,
    pub surreal_url: String,
    // pub surreal_username: String,
    // pub surreal_password: String,
}

pub trait AppState: Clone + Send + Sync + 'static {
    /// Database for the application
    type D: Database;
    /// File storage for the application
    type F: FileStorage;
    /// Configuration for the application
    type C;
    /// Request handler for the application
    // type R: for<'a> HttpClient<'a>;
    type Broker: MessageBroker;
    type Room: RoomLike;

    fn new(config: Self::C) -> impl Future<Output = Self> + Send;

    fn database(&self) -> &Self::D;

    fn file_storage(&self) -> &Self::F;

    fn request_client(&self) -> &reqwest::Client;

    fn room_manager(&self) -> &RoomManager<Self::Broker, Self::Room>;

    fn run(&self, port: u16) -> impl Future<Output = Result<(), ServerError>> + Send;
}

#[derive(Clone)]
pub struct Application {
    room_manager: RoomManager<SocketIoMessageBroker, Presentation>,
    db: SurrealDatabase,
    fs: S3Bucket,
    request_client: reqwest::Client,
}

impl AppState for Application {
    type D = SurrealDatabase;
    type F = S3Bucket;
    type C = ApplicationConfig;
    type Broker = SocketIoMessageBroker;
    type Room = Presentation;

    async fn new(config: Self::C) -> Self {
        let aws_config = aws_sdk_s3::config::Builder::new()
            .region(aws_sdk_s3::config::Region::new(config.s3_region.clone()))
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                config.aws_key,
                config.aws_key_secret,
                None,
                None,
                "loaded-from-custom-env",
            ))
            .build();

        let fs = S3Bucket::new(aws_config, &config.s3_region, &config.aws_bucket);

        info!("Connecting to SurrealDB at {}", config.surreal_url);
        let client = match Surreal::new::<Ws>(config.surreal_url.clone()).await {
            Ok(client) => client,
            Err(e) => {
                error!(
                    "Failed to connect to SurrealDB at {}: {}",
                    config.surreal_url, e
                );
                panic!(
                    "Database connection failed. Please check your SurrealDB URL and ensure the server is running."
                );
            }
        };

        match client
            .signin(Root {
                username: "root",
                password: "root",
            })
            .await
        {
            Ok(_) => info!("Successfully authenticated with SurrealDB"),
            Err(e) => {
                error!("Failed to authenticate with SurrealDB: {}", e);
                panic!("Database authentication failed. Please check your credentials.");
            }
        }

        match client.use_ns("riva").use_db("v1").await {
            Ok(()) => info!("Successfully connected to namespace and database"),
            Err(e) => {
                error!("Failed to use namespace and database: {}", e);
                panic!("Failed to select namespace and database.");
            }
        }

        let db = SurrealDatabase::new(client);

        let request_client = reqwest::Client::new();

        let broker = SocketIoMessageBroker::new();

        let room_manager = RoomManager::<SocketIoMessageBroker, Presentation>::new(broker);

        Self {
            room_manager,
            db,
            fs,
            request_client,
        }
    }

    fn database(&self) -> &Self::D {
        &self.db
    }
    fn file_storage(&self) -> &Self::F {
        &self.fs
    }
    fn request_client(&self) -> &reqwest::Client {
        &self.request_client
    }

    fn room_manager(&self) -> &RoomManager<SocketIoMessageBroker, Presentation> {
        &self.room_manager
    }

    async fn run(&self, port: u16) -> Result<(), ServerError> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        info!(address = %addr, "Starting WebSocket server");

        // Configure CORS with explicit headers instead of Any
        let cors = tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allow_headers([
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::ORIGIN,
            ]);
        debug!("CORS configuration set up");

        // Create a shared state that will be used by both SocketIO and route handlers
        let shared_state = self.clone();

        let (socket_io_layer, io) = SocketIo::builder()
            .with_state(shared_state.clone())
            .build_layer();
        debug!("SocketIO layer created");

        // Register the on_connect handler for the root namespace
        // io.ns("/", Self::on_connect);
        debug!("Root namespace handler registered");

        let app = axum::Router::new()
            .nest(
                "/rooms",
                axum::Router::new()
                    .route("/", get(handlers::room::get_rooms::<Self>))
                    .route("/", post(handlers::room::create_room::<Self>))
                    .route("/{room_id}", get(handlers::room::get_room::<Self>))
                    .route("/{room_id}", put(handlers::room::update_room::<Self>))
                    .route("/{room_id}", delete(handlers::room::delete_room::<Self>))
                    .route(
                        "/{room_id}/upsert",
                        post(handlers::room::upsert_room::<Self>),
                    ), // .route(
                       //     "/{room_id}/broadcast-event",
                       //     post(handlers::room::broadcast_event),
                       // ),
            )
            .with_state(shared_state) // Use the same shared state for route handlers
            .layer(axum::Extension(io.clone())) // Add the IO instance as an extension
            .layer(socket_io_layer)
            .layer(cors);
        debug!("Axum router configured");

        info!(port = port, "Binding TCP listener");
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => {
                info!(address = %addr, "TCP listener bound successfully");
                l
            }
            Err(e) => {
                error!(address = %addr, error = %e, "Failed to bind TCP listener");
                return Err(e.into());
            }
        };

        info!(address = %addr, "Starting server");
        if let Err(e) = axum::serve(listener, app).await {
            error!(error = %e, "Server error");
            return Err(e.into());
        }

        Ok(())
    }
}
