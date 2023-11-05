use regex::Regex;
use std::{fs, path::Path};

#[derive(Debug)]
pub struct ProtoData {
    pub services: Vec<ProtoService>,
}

#[derive(Debug)]
pub struct ProtoService {
    pub name: String,
    pub rpcs: Vec<ProtoRPC>,
}

#[derive(Debug)]
pub struct ProtoRPC {
    pub name: String,
    pub request_name: String,
    pub response_name: String,
}

pub fn get_proto_data(service_root: &Path) -> Result<ProtoData, String> {
    let service_capture_regex =
        Regex::new(r"service\s*(?P<name>\w+)\s+\{\n(?P<content>(?:(?:.+)\n+)*)\}").unwrap();
    let rpc_capture_regex =
        Regex::new(r"\s*rpc\s+(?P<name>\w+)\s+\((?P<req>\w+)\)\s+returns\s+\((?P<resp>\w+)\)")
            .unwrap();
    let service_files: Vec<String> = fs::read_dir(service_root)
        .expect("Could not read contents of interface directory")
        .filter(|entry| entry.is_ok())
        .map(|entry| entry.unwrap().path().to_str().unwrap().to_string())
        .collect();

    let mut proto_services = Vec::new();
    for service_file in service_files.into_iter() {
        let service_path = format!("{}", service_file);
        let service = fs::read_to_string(service_path).expect("Could not read service root");
        for service_cap in service_capture_regex.captures_iter(&service[..]) {
            let mut proto_calls = Vec::new();
            for rpc_cap in rpc_capture_regex.captures_iter(&service_cap["content"]) {
                proto_calls.push(ProtoRPC {
                    name: rpc_cap["name"].to_string(),
                    request_name: rpc_cap["req"].to_string(),
                    response_name: rpc_cap["resp"].to_string(),
                });
            }
            proto_services.push(ProtoService {
                name: service_cap["name"].to_string(),
                rpcs: proto_calls,
            });
        }
    }

    Ok(ProtoData {
        services: proto_services,
    })
}
