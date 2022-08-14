use crate::protos::parser::ProtoData;
use convert_case::{Case, Casing};

pub fn generate_controller_files_contents(proto_data: ProtoData) -> Vec<(String, String)> {
    let mut mods = Vec::new();
    for service in proto_data.services.iter() {
        println!("Generating service");

        let mut controller_body = "".to_string();
        for rpc in service.rpcs {
            controller_body += format!(
                "
async fn {}(
        &self,
        request: Request<{}>,
    ) -> Result<Response<{}>, Status> {{
        todo!()
    }}
",
                rpc.name.to_case(Case::Snake),
                rpc.request_name,
                rpc.response_name
            );
        }
        let file_contents = format!(
            "use tonic::async_trait;
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
        mods.push(service.name.to_case(Case::Snake));
    }
}

pub fn generate_controller_mod_file_contents(proto_data: ProtoData) -> Vec<u8> {
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
