use std::{path::Path, process::Command, fs};

pub fn create_app(name: &str) {
    create_directories(name);
    let project_root = format!("./{}", name);
    let project_root_canon = Path::new(&project_root);
    Command::new("cargo")
        .args(["new", "web", "--bin", "--name", name])
        .current_dir(project_root_canon)
        .output()
        .expect("failed to execute process");
}

fn create_directories(name: &str) {
    let directories_to_create = [
        "interface",
        "interface/grpc",
        "interface/grpc/services",
        "interface/grpc/models",
    ];

    directories_to_create.iter().for_each(|dir| {
        fs::create_dir(format!("./{}/{}", name, dir))
            .expect("Should be able to create models directory");
    });
}
