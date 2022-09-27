extern crate proc_macro;
use std::{fs::File, path::Path};

use convert_case::{Case, Casing};
use flair_core::{
    protos::parser::get_proto_data,
    scaffolding::controller::{
        generate_controller_files_contents, generate_controller_mod_file_contents,
    },
};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::io::prelude::*;

#[proc_macro]
pub fn autogen_controllers(_item: TokenStream) -> TokenStream {
    let path = Path::new("../interface/services");
    let proto_data = get_proto_data(&path).expect("Should have worked");
    let file_with_contents = generate_controller_files_contents(&proto_data);
    let mod_contents = generate_controller_mod_file_contents(&proto_data);

    for (file_name, file_contents) in file_with_contents.iter() {
        // TODO check if file exists and if it does doen't auto gen contents
        let mut file = File::create(file_name).expect("Could not create controller file");
        file.write_all(file_contents.as_bytes())
            .expect("Could not write to controller file");
    }
    let mut mod_file =
        File::create("src/controllers/mod.rs").expect("Could not create controller file");

    mod_file
        .write_all(&mod_contents)
        .expect("Could not write body");
    let string_proto_data = format!("{:#?}", proto_data);
    let gen = quote! {
        println!("{}", #string_proto_data);
    };
    gen.into()
}

#[proc_macro]
pub fn setup_server(input: TokenStream) -> TokenStream {
    let mut app_name: String = "Flair App".to_string();
    let mut version: String = "0.1.0".to_string();

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

    let path = Path::new("../interface/services");
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

    let gen = quote! {
    // Setup logging
    flair_core::logging::util::setup();

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
        let config_file = std::fs::File::open(matches.value_of("config").unwrap()).unwrap();
    let config = std::sync::Arc::new({
        let deserializer = serde_yaml::Deserializer::from_reader(config_file);
        let config: Config = serde_ignored::deserialize(deserializer, |path| {
            log::warn!("Unused config field: {}", path);
        })
        .unwrap();
        // Edit config here if you want to
        config
    });

    let db_pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(config.database.num_connections)
        .test_before_acquire(true)
        .connect(&config.database.url)
        .await?;

    #(#controllers)*

    let (host, port) = flair_core::helpers::split_host_and_port(&config.bind_address);
    let addr = format!("{}:{}", host, port);
    let server = tonic::transport::Server::builder()
        #(#services)*;

    server
        .serve_with_shutdown(
            std::net::SocketAddr::from_str(&addr[..]).unwrap(),
            async move {
                // Add closers for other processes
            },
        )
        .await?;
    };
    gen.into()
}
