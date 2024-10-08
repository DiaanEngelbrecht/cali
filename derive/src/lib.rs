//! # About Cali Derive
//!
//! Cali is a rust framework for GRPC microservices. The Cali Derive crate is part of a series of
//! three crates:
//! - cali_core contains most of the actual logic behind cali. It contains middleware, helpers,
//!   logging utilities and utilities used in parsing proto files for codegen.
//! - cali_cli is the command line utility that facilitates codegen for new projects and controller
//!   and store generation.
//! - cali_derive - you are here - includes procedural macros that generate the main entry point,
//!   connects middleware and controllers and does some of the magic.
//!
//! Ideally you shouldn't need to use this crate directly, but these docs should serve as a jump
//! off point to understand what each macro does. You are always encouraged to expand these macros
//! to take a peek under the hood. I try to keep the generated code legible so that you can copy it
//! out and use it if you ever need an escape hatch.

extern crate proc_macro;
use std::path::Path;

use cali_core::protos::parser::get_proto_data;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// This is typically used inside a project's build.rs file. It takes no arguments and serves as a
/// convenience that automatically generates the necessary tonic code to generate rust code from
/// proto files.
#[proc_macro]
pub fn autogen_protos(_item: TokenStream) -> TokenStream {
    let gen = quote! {
        let service_files: Vec<String> = std::fs::read_dir("../interface/grpc/services/")
            .expect("Could not read contents of interface file")
            .filter(|entry| entry.is_ok())
            .map(|entry| entry.unwrap().path().to_str().unwrap().to_string())
            .collect();

        let out_path = std::path::Path::new("src/protos");
        if !out_path.exists() {
            let _ = std::fs::create_dir(out_path)
                .expect(&format!("Unable to create protos folder {:?}", out_path));
        }

        if service_files.len() > 0 {
            tonic_build::configure()
                .build_server(true)
                .out_dir(out_path.to_str().unwrap())
                .compile(service_files.as_slice(), &["../interface/grpc/".to_string()])
                .unwrap();
        }

        // build the protos mod.rs
        let path = std::path::Path::new("../interface/grpc/services");
        let proto_data = cali_core::protos::parser::get_proto_data(&path).expect("Should have worked");
        let mut mod_contents = "".to_string();
        proto_data.services.iter().for_each(|service| {
            let import_line = format!("pub mod {};\n", service.name.to_case(Case::Snake));
            mod_contents.push_str(&import_line);
        });
        let mod_path = std::path::Path::new("src/protos/mod.rs");
        std::fs::write(mod_path, mod_contents).expect("Could not write main file");
    };
    gen.into()
}

/// This procedural macro is typically used at the top of a controller file to instantiate a struct
/// for which the proto service's generated rust trait will be implemented. Usually this is
/// generated from cali_cli.
#[proc_macro]
pub fn controller(input: TokenStream) -> TokenStream {
    let controller_struct_name = Ident::new(&format!("{}", input)[..], Span::call_site());

    let gen = quote! {
        #[derive(Clone)]
        pub struct #controller_struct_name {}

        impl #controller_struct_name {
            pub fn new() -> Self {
                #controller_struct_name {}
            }
        }
    };
    gen.into()
}

/// Snare is Cali's very simple ORM/convenience around sqlx. You can Ensnare your structs that map
/// to database table's by including it in your derive directives.
/// ```rust
/// #[derive(Clone, FromRow, Ensnare)]
/// pub struct Account {
///     pub id: i64,
///     pub name: String,
///     pub email: String,
///     pub password_hash: String
/// }
/// ```
///
/// From here you should now have trap/1 available on your structs. Trap returns a wrapping type
/// `Snare<T>` that allows you to generate insert statements with easy:
/// ```rust
/// let result = account.trap("accounts").insert().execute(conn).await?;
/// ```
#[proc_macro_derive(Ensnare)]
pub fn derive_ensnare(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let struct_fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(named_fields),
            ..
        }) => named_fields
            .named
            .iter()
            .filter_map(|f| f.ident.clone())
            .collect::<Vec<Ident>>(),
        _ => {
            panic!("Can only Ensnare struct types");
        }
    };

    let bind_points = struct_fields
        .iter()
        .map(|_| "?".to_string())
        .collect::<Vec<String>>()
        .join(",");

    let fields = struct_fields
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let bindings: Vec<TokenStream2> = struct_fields
        .iter()
        .map(|f| quote!(bind(self.#f.clone())))
        .collect();

    let expanded = quote! {
    impl cali_core::store::snare::Ensnarable for #struct_name {
                fn insert_parts(&self) -> (String, String) {
                    (#fields.to_string(), #bind_points.to_string())
                }

                fn capture<'a>(
                    &'a self,
                    query: sqlx::query::Query<
                        'a,
                        sqlx::MySql,
                        <sqlx::MySql as sqlx::Database>::Arguments<'_>,
                    >,
                ) -> sqlx::query::Query<
                    'a,
                    sqlx::MySql,
                    <sqlx::MySql as sqlx::Database>::Arguments<'_>,
                > {
                    query.#(#bindings).*
                }
            }

            impl #struct_name {
                pub fn trap(self, table_name: &str) -> cali_core::store::snare::Snare<#struct_name> {
                    cali_core::store::snare::Snare {
                        query: "".to_string(),
                        table_name: table_name.to_string(),
                        data: self,
                    }
                }
            }
        };

    TokenStream::from(expanded)
}

/// This is the main magic macro of cali, usually found in the entry/main.rs of the web crate.
/// It takes three arguments. The first is a string literal that contains your application name,
/// the second is also a string literal that contains your application version, and the last
/// argument is your own server config. 
///
/// The server config makes use of the builder pattern to enable opt in features of the framework.
/// At this stage, there are three opt in features:
/// - Enable database using `.enable_database()`
/// - Add custom Tower middleware using `.add_middleware(setup_fn: impl FnOnce(Server<CaliBaseTowerStack>) -> Server<ResultTowerStack> + 'static)`
/// - Create your own globally available context using the `.add_global_context(your_context: T)`
///   where T is the server context type defined in your application code. This isn't optional, but
///   can be left to a empty struct with very little consequence.
#[proc_macro]
pub fn setup_server(input: TokenStream) -> TokenStream {
    let app_name: String;
    let version: String;
    let server_config: Ident;

    let input = proc_macro2::TokenStream::from(input);
    let mut params_stream = input.into_iter();

    if let Some(proc_macro2::TokenTree::Literal(val)) = params_stream.next() {
        let temp = format!("{}", val);
        params_stream.next(); // Skip the comma
        app_name = temp[1..temp.len() - 1].to_string();
    } else {
        panic!("Please add an application name")
    }

    if let Some(proc_macro2::TokenTree::Literal(val)) = params_stream.next() {
        let temp = format!("{}", val);
        version = temp[1..temp.len() - 1].to_string();
        params_stream.next(); // Skip the comma
    } else {
        panic!("Please add a version")
    }

    if let Some(proc_macro2::TokenTree::Ident(val)) = params_stream.next() {
        server_config = val;
    } else {
        panic!("Cali config has to be provided")
    }

    let path = Path::new("./interface/grpc/services");
    let proto_data = get_proto_data(&path).expect("Should have worked");

    let web_crate = Ident::new(&format!("{}_web", app_name)[..], Span::call_site());

    let controllers: Vec<proc_macro2::TokenStream> = proto_data
        .services
        .iter()
        .map(|service| {
            let controller_var_name = Ident::new(
                &format!("{}_controller", service.name.to_case(Case::Snake))[..],
                Span::call_site(),
            );

            let controller_snake_name = Ident::new(
                &format!("{}", service.name.to_case(Case::Snake))[..],
                Span::call_site(),
            );

            let controller_name = Ident::new(
                &format!(
                    "{}Controller",
                    service.name.to_case(Case::UpperCamel)
                )[..],
                Span::call_site(),
            );

            quote! {
                let #controller_var_name = #web_crate::controllers::#controller_snake_name::#controller_name::new();
            }
        })
        .collect();

    let services: Vec<proc_macro2::TokenStream> = proto_data
        .services
        .iter()
        .map(|service| {
            let controller_var_name = Ident::new(
                &format!("{}_controller", service.name.to_case(Case::Snake))[..],
                Span::call_site(),
            );
            let service_name = Ident::new(
                &format!("{}Server", service.name.to_case(Case::UpperCamel))[..],
                Span::call_site(),
            );

            let controller_snake_name = Ident::new(
                &format!("{}", service.name.to_case(Case::Snake))[..],
                Span::call_site(),
            );
            let server_snake_name = Ident::new(
                &format!("{}_server", service.name.to_case(Case::Snake))[..],
                Span::call_site(),
            );

            quote! {
                .add_service(#web_crate::protos::#controller_snake_name::#server_snake_name::#service_name::new(#controller_var_name))
            }
        })
        .collect();

    let mut body = quote! {
        // Setup tokio_console if setup

        if (#server_config.tokio_console) {
            console_subscriber::init();
        }

        // Setup logging
        cali_core::logging::util::setup();

        log::info!("Getting ready...");
        // Configure CLI App
        let matches = clap::App::new(#app_name)
            .version(#version)
            .arg(
                clap::Arg::with_name("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Sets a custom config file")
                    .default_value("./web/config/dev.yml")
                    .takes_value(true),
            )
            .get_matches();


        // Setup Config File
        log::info!("Loading config...");
        let config_file = std::fs::File::open(matches.value_of("config")
                            .expect("No value set for config path"))
                            .expect("Could not open config file at web/config/dev.yml");

        let config: std::sync::Arc<Config> = std::sync::Arc::new({
            let deserializer = serde_yaml::Deserializer::from_reader(config_file);
            let config: Config = serde_ignored::deserialize(deserializer, |path| {
                log::warn!("Unused config field: {}", path);
            })
            .expect("Could not deserialize config");
            // Edit config here if you want to
            config
        });
        log::info!("Config loaded!");

        let db_pool = if (#server_config.database) {
            log::info!("Connecting to DB...");
            let pool = sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(config.database.num_connections)
                .test_before_acquire(true)
                .connect(&config.database.url)
                .await?;
            log::info!("Connected!");
            Some(pool)
        } else {
            None
        };

        let server_ctx : std::sync::Arc<cali_core::ServerContext> = std::sync::Arc::new(cali_core::ServerContext { db_pool });

        let context_layer = cali_core::middleware::server_context::ServerContextLayer {
            config: config.clone(),
            extendable_context: #server_config.global_context.clone(),
            internal_context: server_ctx.clone()
        };


        #(#controllers)*

        let (host, port) = cali_core::helpers::split_host_and_port(&config.bind_address);
        let addr = format!("{}:{}", host, port);

        let server = if let Some(middleware_fn) = #server_config.middleware_setup {
            (middleware_fn)(tonic::transport::Server::builder()
                .layer(context_layer))
        } else {
            tonic::transport::Server::builder()
                .layer(context_layer)
        }#(#services)*;;


    };

    let server_segment = quote! {
        let mut interrupt_signal = tokio::signal::ctrl_c();
        let closer = async move {
            let _ = interrupt_signal.await;
            log::info!("Goodbye!");
        };

        log::info!("GRPC server started, waiting for requests...");
        server
            .serve_with_shutdown(
                std::net::SocketAddr::from_str(&addr[..]).unwrap(),
                async move {
                    // Add closers for other processes
                    let _ = closer.await;
                },
            )
            .await?;

    };

    let no_server_segment = quote! {
        log::info!("No GRPC services have been defined, terminating server.");
    };

    if services.len() > 0 {
        body.extend(server_segment);
    } else {
        body.extend(no_server_segment);
    }

    body.into()
}

/// Usually found in tests/common/mod.rs, this defines run/2 which takes a reference to the server
/// config file, and the test function. This creates a new test db, runs the required migrations
/// and injects the required global context. This makes use of your previously configured cali
/// context.
#[proc_macro]
pub fn test_runner(_input: TokenStream) -> TokenStream {
    // Rather let this return a wrapping type called test context under cali core?
    // That way I can implement the drop trait on that type and clean up test databases that way?
    let test_setup_body = quote! {
         pub async fn run(config_file: &str, test: impl std::future::Future<Output = ()>) -> () {
        cali_core::logging::util::setup();

        let config_file = std::fs::File::open(config_file).expect("Could not open config file");

        let config = {
            let deserializer = serde_yaml::Deserializer::from_reader(config_file);
            let config: Config =
                serde_ignored::deserialize(deserializer, |_| {}).expect("Could not deserialize config");
            config
        };

        let db_url = url::Url::parse(&config.clone().database.url).expect("Unable to parse DB url");

        // Create the database
        let pool = sqlx::MySqlPool::connect(&db_url[..url::Position::BeforePath])
            .await
            .unwrap();

        let db_name = db_url
            .path_segments()
            .expect("No database specified")
            .next()
            .expect("No database specified");

        // Delete the existing database
        let drop_query = format!("DROP DATABASE IF EXISTS {}", db_name);
        sqlx::query(&drop_query).execute(&pool).await.unwrap();

        // Recreate it
        let create_query = format!("CREATE DATABASE IF NOT EXISTS {}", db_name);
        sqlx::query(&create_query).execute(&pool).await.unwrap();

        // Run all migrations
        let pool = sqlx::MySqlPool::connect(&db_url.to_string()).await.unwrap();

        sqlx::migrate!("../store/migrations")
            .run(&pool)
            .await
            .expect("Expected to be able to run migrations");

        let db_pool = Some(sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(1)
            .test_before_acquire(true)
            .connect(&config.clone().database.url)
            .await
            .expect("Couldn't connect to test database"));

        let mut context: std::collections::HashMap<std::any::TypeId, cali_core::MapKey> =
            std::collections::HashMap::new();

        context.insert(
            std::any::TypeId::of::<cali_core::ServerContext>(),
            std::sync::Arc::new(cali_core::ServerContext { db_pool }),
        );


        context.insert(
            std::any::TypeId::of::<Config>(),
            std::sync::Arc::new(config),
        );

        cali_core::SERVER_CONTEXT
            .scope(std::sync::Arc::new(context), test)
            .await
    }
            };

    test_setup_body.into()
}
