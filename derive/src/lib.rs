extern crate proc_macro;
use std::path::Path;

use convert_case::{Case, Casing};
use flair_core::protos::parser::get_proto_data;
use proc_macro::{Delimiter, Group, TokenStream, TokenTree};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;

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
        };
    gen.into()
}

#[proc_macro]
pub fn controller(input: TokenStream) -> TokenStream {
    let controller_struct_name = Ident::new(&format!("{}", input)[..], Span::call_site());

    let gen = quote! {
        use flair_derive::endpoint;

        #[derive(Clone)]
        pub struct #controller_struct_name {
            pub(crate) server_ctx: std::sync::Arc<flair_core::ServerContext>,
        }

        impl #controller_struct_name {
            pub fn new(server_ctx: std::sync::Arc<flair_core::ServerContext>) -> Self {
                #controller_struct_name { server_ctx }
            }
        }
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn endpoint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut items: Vec<TokenTree> = item.into_iter().collect();
    let t = if let Some(t) = items.pop() {
        match t {
            TokenTree::Group(g) => TokenTree::Group(Group::new(
                g.delimiter(),
                g.stream()
                    .into_iter()
                    .map(|tt| match tt {
                        TokenTree::Group(gd) => match gd.delimiter() {
                            Delimiter::Parenthesis => {
                                TokenTree::Group(Group::new(
                                    Delimiter::Parenthesis,
                                    gd.stream()
                                        .into_iter()
                                        .map(|ttt| match ttt {
                                            TokenTree::Group(gdd) => match gdd.delimiter() {
                                                Delimiter::Brace => {
                                                    TokenTree::Group(Group::new(
                                                        Delimiter::Brace,
                                                        gdd.stream().into_iter().map(|ft| match ft {
                                                            TokenTree::Group(fg) => match fg.delimiter() {
                                                                Delimiter::Brace => {

                                                                    if &format!("{}", fg.stream())[..] != "return __ret ;" {
                                                                        let inner : TokenStream2 = fg.stream().into();
                                                                        let wrapper = quote! {

                                                                            flair_core::SERVER_CONTEXT
                                                                                .scope(self.server_ctx.clone(), async move {
                                                                                    #inner
                                                                                })
                                                                                .await
                                                                        };

                                                                        TokenTree::Group(Group::new(fg.delimiter(), wrapper.into()))
                                                                    } else {
                                                                        TokenTree::Group(fg)
                                                                    }
                                                                    },
                                                                fd => {
                                                                    TokenTree::Group(Group::new(fd, fg.stream()))
                                                                }
                                                            },
                                                            _ => ft,
                                                        }).collect(),
                                                    ))
                                                }
                                                dd => {
                                                    TokenTree::Group(Group::new(dd, gdd.stream()))
                                                }
                                            },
                                            _ => ttt,
                                        })
                                        .collect(),
                                ))
                            }
                            d => TokenTree::Group(Group::new(d, gd.stream())),
                        },
                        _ => tt,
                    })
                    .collect(),
            )),
            _ => t,
        }
    } else {
        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new()))
    };
    let body = TokenStream::from(t);
    let body2: TokenStream2 = body.into();

    let mut func_sig = TokenStream::new();
    func_sig.extend(items.into_iter());
    let func_sig2: TokenStream2 = func_sig.into();

    let gen = quote! {
        #func_sig2 #body2

    };
    gen.into()
}

#[proc_macro]
pub fn setup_server(input: TokenStream) -> TokenStream {
    let app_name: String;
    let version: String;

    let mut params_stream = input.into_iter();

    if let Some(proc_macro::TokenTree::Literal(val)) = params_stream.next() {
        let temp = format!("{}", val);
        params_stream.next(); // Skip the comma
        app_name = temp[1..temp.len() - 1].to_string();
    } else {
        panic!("Please add an application name")
    }

    if let Some(proc_macro::TokenTree::Literal(val)) = params_stream.next() {
        let temp = format!("{}", val);
        version = temp[1..temp.len() - 1].to_string();
    } else {
        panic!("Please add a version")
    }

    let path = Path::new("../interface/grpc/services");
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
                let #controller_var_name = #web_crate::controllers::#controller_snake_name::#controller_name::new(server_ctx.clone());
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

    let gen = quote! {
    // Setup logging
    flair_core::logging::util::setup();

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
                .default_value("config/dev.yml")
                .takes_value(true),
        )
        .get_matches();


    // Setup Config File
    log::info!("Loading config...");
        let config_file = std::fs::File::open(matches.value_of("config").expect("No value set for config path"))
            .expect("Could not open config file at config/dev.yml");
        let config = std::sync::Arc::new({
        let deserializer = serde_yaml::Deserializer::from_reader(config_file);
        let config: Config = serde_ignored::deserialize(deserializer, |path| {
            log::warn!("Unused config field: {}", path);
        })
        .expect("Could not deserialize config");
        // Edit config here if you want to
        config
    });

    log::info!("Connecting to DB...");
    let db_pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(config.database.num_connections)
        .test_before_acquire(true)
        .connect(&config.database.url)
        .await?;


    let server_ctx = std::sync::Arc::new(flair_core::ServerContext { db_pool });

    #(#controllers)*

    let (host, port) = flair_core::helpers::split_host_and_port(&config.bind_address);
    let addr = format!("{}:{}", host, port);
    let server = tonic::transport::Server::builder()
        #(#services)*;

    log::info!("Starting server...");
    let mut interrupt_signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    let closer = async move {
        let _ = interrupt_signal.recv().await;
        log::info!("Good bye!");
    };


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
    gen.into()
}
