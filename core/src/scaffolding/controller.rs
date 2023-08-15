use std::{fs, path::Path};

use crate::protos::parser::{ProtoData, ProtoService};
use convert_case::{Case, Casing};
use proc_macro2::Ident;
use syn::{ItemUse, UsePath, UseTree};

pub fn generate_controller_files_contents(proto_data: &ProtoData) -> Vec<(String, String)> {
    let mut file_with_contents = Vec::new();
    for service in proto_data.services.iter() {
        // Compose the filename
        let file_name = format!(
            "web/src/controllers/{}.rs",
            service.name.to_case(Case::Snake)
        );

        let file_exists = Path::new(&file_name).try_exists().unwrap_or(false);
        let file_contents = if !file_exists {
            // If new file, generate a new file
            generate_new_controller_file(service)
        } else {
            // If existing file, mutate import statement
            //
            // and inject new function
            generate_existing_controller_file(&file_name, service)
        };

        file_with_contents.push((file_name, file_contents))
    }

    file_with_contents
}

fn generate_existing_controller_file(file_name: &str, service: &ProtoService) -> String {
    let contents = fs::read_to_string(file_name).expect("Should have been able to read the file");
    let stream: proc_macro2::TokenStream = contents.parse().unwrap();
    let file = syn::parse2::<syn::File>(stream).unwrap();
    let mut use_trees: Vec<&UseTree> = Vec::new();

    file.items.iter().for_each(|item| match item {
        syn::Item::Use(ItemUse {
            tree: root_use_tree,
            ..
        }) => {
            // all_import_names_at([ident<crate>, ident<protos>, ident<service>])
            // UseName after path directly or in group
            // build index of all names and build diff
            // if group, import_site at end of group span
            // if solo, import_site as new line after line
            // insert diff at import_site
            use_trees.push(root_use_tree);
        }
        _ => (),
    });

    // let import_statement = format!(
    //     "use crate::protos::{}::{{{}}};",
    //     service.name.to_case(Case::Snake),
    //     rpc_imports.join(", ")
    // );

    let mut rpc_imports = Vec::new();
    for rpc in service.rpcs.iter() {
        if rpc_imports
            .iter()
            .find(|v| v == &&rpc.request_name)
            .is_none()
        {
            rpc_imports.push(rpc.request_name.clone());
            rpc_imports.push(rpc.response_name.clone());
        }
    }

    todo!()
}

fn generate_new_controller_file(service: &ProtoService) -> String {
    let mut controller_body = "".to_string();
    let mut rpc_imports = Vec::new();
    for rpc in service.rpcs.iter() {
        if rpc_imports
            .iter()
            .find(|v| v == &&rpc.request_name)
            .is_none()
        {
            rpc_imports.push(rpc.request_name.clone())
        }

        if rpc_imports
            .iter()
            .find(|v| v == &&rpc.response_name)
            .is_none()
        {
            rpc_imports.push(rpc.response_name.clone())
        }

        controller_body = format!(
            "{}

#[endpoint]
async fn {}(
        &self,
        request: Request<{}>,
    ) -> Result<Response<{}>, Status> {{
        todo!()
    }}
",
            controller_body,
            rpc.name.to_case(Case::Snake),
            rpc.request_name,
            rpc.response_name
        );
    }

    let import_statement = format!(
        "use crate::protos::{}::{{{}}};",
        service.name.to_case(Case::Snake),
        rpc_imports.join(", ")
    );

    format!(
        "use tonic::async_trait;
use tonic::{{Status, Response, Request}};
{import_statement}
use crate::protos::{snake_name}::{snake_name}_server::{name};


flair_derive::controller!({name}Controller);
#[async_trait]
impl {name} for {name}Controller {{
{controller_body}
}}",
        import_statement = import_statement,
        snake_name = service.name.to_case(Case::Snake),
        name = service.name,
        controller_body = controller_body
    )
}

pub fn generate_controller_mod_file_contents(proto_data: &ProtoData) -> Vec<u8> {
    let mut mods = Vec::new();
    for service in proto_data.services.iter() {
        mods.push(service.name.to_case(Case::Snake));
    }
    let mut mod_contents: Vec<u8> = Vec::new();
    for mod_ in mods.iter() {
        mod_contents.extend(format!("pub mod {};\n", mod_).as_bytes().iter())
    }

    mod_contents
}
