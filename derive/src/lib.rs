extern crate proc_macro;
use std::{fs::File, path::Path};

use flair_core::{
    protos::parser::get_proto_data,
    scaffolding::controller::{
        generate_controller_files_contents, generate_controller_mod_file_contents,
    },
};
use proc_macro::TokenStream;
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
