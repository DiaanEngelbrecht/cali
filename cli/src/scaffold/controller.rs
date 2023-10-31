use convert_case::{Case, Casing};
use cali_core::protos::parser::get_proto_data;
use cali_core::protos::parser::ProtoData;
use cali_core::protos::parser::ProtoService;
use proc_macro2::{Ident, LineColumn};
use std::{fs::File, io::Write, path::Path};
use syn::ImplItemFn;
use syn::ItemMod;
use syn::UseTree;
use syn::{ItemImpl, ItemUse};

pub fn sync_protos_with_controllers() {
    let path = Path::new("./interface/grpc/services");
    let proto_data = get_proto_data(&path).expect("Should have worked");
    let file_with_contents = generate_controller_files_contents(&proto_data);
    generate_controller_mod_file_contents(&proto_data);

    for (file_name, file_contents) in file_with_contents.iter() {
        let mut file = File::create(file_name).expect("Could not create controller file");
        file.write_all(file_contents.as_bytes())
            .expect("Could not write to controller file");
    }
}

use std::{collections::HashSet, fs};

fn generate_controller_files_contents(proto_data: &ProtoData) -> Vec<(String, String)> {
    let mut file_with_contents = Vec::new();
    for service in proto_data.services.iter() {
        // Compose the filename
        let file_name = format!(
            "web/src/controllers/{}.rs",
            service.name.to_case(Case::Snake)
        );

        let file_exists = Path::new(&file_name).try_exists().unwrap_or(false);
        if !file_exists {
            // If new file, generate a new file
            let file_contents = generate_new_controller_file(service);

            file_with_contents.push((file_name, file_contents));
        } else {
            // If existing file, mutate import statement
            // and inject new function
            generate_existing_controller_file(&file_name, service)
        };
    }

    file_with_contents
}

fn imported_under_path(tree_root: &UseTree, path: Vec<String>) -> Vec<&Ident> {
    if path.len() == 0 {
        return vec![];
    }

    // Step through the path and if the paths match we continue
    // otherwise abort early
    let mut path_pointer = tree_root;
    let mut path_iter = path.iter();
    loop {
        let path_segment = path_iter.next();
        if path_segment.is_none() {
            break;
        }
        // We need to be able to traverse to the path
        // before we can start looking through groups or names
        match path_pointer {
            UseTree::Path(p) => {
                if p.ident == path_segment.unwrap() {
                    path_pointer = &p.tree;
                } else {
                    return vec![];
                }
            }
            _ => return vec![],
        }
    }

    // Now that we have the base, we want to include all paths
    // under the base
    match path_pointer {
        UseTree::Name(n) => vec![&n.ident],
        UseTree::Group(g) => {
            let mut under_group = vec![];
            g.items.iter().for_each(|i| match i {
                UseTree::Name(n) => under_group.push(&n.ident),
                UseTree::Rename(_) => {
                    unimplemented!(
                        "Need to still support aliases in the generate controllers command"
                    )
                }
                _ => (),
            });

            return under_group;
        }
        _ => return vec![],
    }
}

fn generate_existing_controller_file(file_name: &str, service: &ProtoService) -> () {
    let contents = fs::read_to_string(file_name).expect("Should have been able to read the file");
    let stream: proc_macro2::TokenStream = contents.parse().unwrap();
    let file = syn::parse2::<syn::File>(stream).unwrap();
    let mut use_trees: Vec<&UseTree> = Vec::new();
    let mut functions: HashSet<String> = HashSet::new();
    let mut last_func_loc = None;

    file.items.iter().for_each(|item| match item {
        syn::Item::Use(ItemUse {
            tree: root_use_tree,
            ..
        }) => {
            use_trees.push(root_use_tree);
        }
        syn::Item::Impl(ItemImpl { items, .. }) => {
            items.iter().for_each(|impl_item| match impl_item {
                syn::ImplItem::Fn(ImplItemFn { sig, block, .. }) => {
                    functions.insert(sig.ident.to_string());
                    last_func_loc = Some(block.brace_token.span.close().end())
                }
                _ => (),
            });
        }
        _ => (),
    });

    let mut rpc_imports = HashSet::new();
    let mut rpc_functions = Vec::new();
    for rpc in service.rpcs.iter() {
        rpc_imports.insert(rpc.request_name.clone());
        rpc_imports.insert(rpc.response_name.clone());

        if functions.get(&rpc.name.to_case(Case::Snake)).is_none() {
            rpc_functions.push(rpc);
        }
    }

    let mut controller_body = "".to_string();
    rpc_functions.iter().for_each(|rpc| {
        controller_body = format!(
            "{}

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
    });

    insert_at_loc_in_file(file_name, last_func_loc.unwrap(), controller_body)
        .expect("Coudn't update controller file with imports");

    let mut import_site_loc = None;

    // Get rid of all already imported paths
    for tree_root in use_trees {
        let imports = imported_under_path(
            tree_root,
            vec![
                "crate".to_string(),
                "protos".to_string(),
                service.name.to_case(Case::Snake),
            ],
        );

        if imports.len() != 1 && imports.first().is_some() {
            import_site_loc = Some(imports.first().unwrap().span().start());
        }

        imports.iter().for_each(|ident| {
            rpc_imports.remove(&ident.to_string());
        });
    }

    if import_site_loc.is_none() {
        panic!("Could not find a group under crate::protos::{} to import new request & response objects into",service.name.to_case(Case::Snake));
    }

    let mut import_line = rpc_imports.into_iter().collect::<Vec<String>>().join(", ");
    if !import_line.is_empty() {
        import_line.push_str(", ");
    }

    insert_at_loc_in_file(file_name, import_site_loc.unwrap(), import_line)
        .expect("Coudn't update controller file with imports");
}

/// Take the conents, and put it at the given location, in a given file_name, and write out
fn insert_at_loc_in_file(
    file_name: &str,
    location: LineColumn,
    contents: String,
) -> Result<(), std::io::Error> {
    let local_contents =
        fs::read_to_string(file_name).expect("Should have been able to read the file");

    let new_file = local_contents
        .split('\n')
        .enumerate()
        .map(|(num, line)| {
            if num == location.line - 1 {
                format!(
                    "{}{}{}",
                    line[..location.column].to_string(),
                    contents,
                    line[location.column..].to_string()
                )
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    fs::File::create(file_name)?.write_all(new_file.as_bytes())
}

fn generate_new_controller_file(service: &ProtoService) -> String {
    let mut controller_body = "".to_string();
    // TODO: much better if this is a HashSet
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

        // TODO: get rid of string format favour of a proper quote!
        controller_body = format!(
            "{}

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


cali_derive::controller!({name}Controller);
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

fn generate_controller_mod_file_contents(proto_data: &ProtoData) {
    let file_name = "./web/src/controllers/mod.rs";

    let file_exists = Path::new(&file_name).try_exists().unwrap_or(false);
    let mut mods = Vec::new();
    let mut location = LineColumn { line: 1, column: 0 };
    if file_exists {
        let contents =
            fs::read_to_string(file_name).expect("Should have been able to read the file");
        let stream: proc_macro2::TokenStream = contents.parse().unwrap();
        let file = syn::parse2::<syn::File>(stream).unwrap();
        let mut module_imports: Vec<&ItemMod> = Vec::new();

        file.items.iter().for_each(|item| match item {
            syn::Item::Mod(mod_import) => {
                module_imports.push(mod_import);
            }
            _ => (),
        });

        for service in proto_data.services.iter() {
            let i = service.name.to_case(Case::Snake);
            match module_imports
                .iter()
                .find(|tree_root| tree_root.ident.to_string() == i)
            {
                Some(item) => {
                    location = LineColumn {
                        line: item.ident.span().end().line,
                        column: item.ident.span().end().column + 1, // Increment for semicolon
                    }
                }
                None => mods.push(i),
            }
        }
    } else {
        for service in proto_data.services.iter() {
            mods.push(service.name.to_case(Case::Snake));
        }
    }

    let mut mod_contents: Vec<u8> = Vec::new();
    if !(location.line == 1 && location.column == 0) {
        mod_contents.extend(format!("\n").as_bytes().iter())
    }
    let mods_string = mods
        .iter()
        .map(|mod_| format!("pub mod {};", mod_))
        .collect::<Vec<String>>()
        .join("\n");
    mod_contents.extend(mods_string.as_bytes().iter());
    if file_exists {
        insert_at_loc_in_file(
            file_name,
            location,
            String::from_utf8(mod_contents).unwrap(),
        )
        .expect("Coudn't update controller mod file.");
    } else {
        let mut mod_file = File::create("./web/src/controllers/mod.rs")
            .expect("Could not create controller mod file");

        mod_file
            .write_all(&mod_contents)
            .expect("Could not write body");
    }
}
