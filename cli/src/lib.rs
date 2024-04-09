//! # About Cali Cli
//!
//! Cali is a rust framework for GRPC microservices. The Cali Cli crate is part of a series of
//! three crates:
//! - cali_core contains most of the actual logic behind cali. It contains middleware, helpers,
//!   logging utilities and utilities used in parsing proto files for codegen.
//! - cali_cli - you are here - is the command line utility that facilitates codegen for new projects and controller
//!   and store generation.
//! - cali_derive includes procedural macros that generate the main entry point,
//!   connects middleware and controllers and does some of the magic.
//!
//! ## Getting started
//!
//! Cali needs the `protoc` protocol buffers compiler + protocol buffer resource files to build tonic.
//!
//! On Ubuntu:
//! ```bash
//! sudo apt install -y protobuf-compiler libprotobuf-dev
//! ```
//! 
//! Then you want to go ahead and grab cali's cli tool:
//! ```bash
//! cargo install cali_cli
//! ```
//!
//! Finally, create a new project with:
//! ```
//! cali new <your project name>
//! ```
//!
//! This generates a cargo workspace with the following structure:
//! ```
//! .
//! ├── interface
//! │   └── grpc
//! │       ├── models
//! │       └── services
//! ├── store
//! │   └── src
//! │       └── repositories
//! └── web
//!     ├── config
//!     └── src
//!         ├── controllers
//!         ├── entry
//!         └── protos
//! ```
//! The web package is your entry point, and the interfaces directory specifies your GRPC services and models. Type 
//! out a simple service definition under `/interfaces/grpc/services` and run:
//! 
//! ```
//! cali generate controllers
//! ```
//! 
//! You should see your rust controllers generated in the controllers directory. From here you get to choose:
//! 
//! 1. Write your code directly in the controller (for simple endpoints)
//! 2. Create some modules directly in the web crate to handle your logic.
//! 3. Add a new cargo library and have web depend on it.
//!
pub mod scaffold;

pub static CORE_VERSION: &str = "0.3.0";
pub static DERIVE_VERSION: &str = "0.3.0";
