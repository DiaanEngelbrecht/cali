extern crate proc_macro;
use std::{fs::File, path::Path};

use flair_core::protos::parser::get_proto_data;
use proc_macro::TokenStream;
use quote::quote;
use std::io::prelude::*;

#[proc_macro]
pub fn print_test(_item: TokenStream) -> TokenStream {
    let path = Path::new("../interface/services");
    let proto_data = get_proto_data(&path).expect("Should have worked");

    let file_name = format!("src/controllers/{}.rs", service.name.to_case(Case::Snake));
    let mut file = File::create(file_name).expect("Could not create controller file");

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
