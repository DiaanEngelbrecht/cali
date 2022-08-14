use crate::protos::parser::ProtoData;
use convert_case::{Case, Casing};

pub fn generate_controller_files_contents(proto_data: &ProtoData) -> Vec<(String, String)> {
    let mut file_with_contents = Vec::new();
    for service in proto_data.services.iter() {
        let file_name = format!("src/controllers/{}.rs", service.name.to_case(Case::Snake));
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

        let file_contents = format!(
            "use tonic::async_trait;
use tonic::{{Status, Response, Request}};
{}
use crate::protos::{}::{}_server::{};

#[derive(Clone)]
pub struct {}Controller {{}}

impl {}Controller {{
    pub fn new() -> Self {{
        {}Controller {{}}
    }}
}}

#[async_trait]
impl {} for {}Controller {{
{}
}}",
            import_statement,
            service.name.to_case(Case::Snake),
            service.name.to_case(Case::Snake),
            service.name,
            service.name,
            service.name,
            service.name,
            service.name,
            service.name,
            controller_body
        );
        file_with_contents.push((file_name, file_contents))
    }

    file_with_contents
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
