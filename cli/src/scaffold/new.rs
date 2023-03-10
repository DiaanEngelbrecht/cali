use serde::Serialize;
use std::{fs, path::Path, process::Command};

use tinytemplate::TinyTemplate;

pub fn create_app(name: &str) {
    create_directories(name);
    create_main_file(name);
}

fn create_directories(name: &str) {
    let directories_to_create = [
        "",
        "interface",
        "interface/grpc",
        "interface/grpc/services",
        "interface/grpc/models",
        "web",
        "web/config",
        "web/src",
        "web/src/entry",
        "web/src/controllers",
        "web/src/protos",
    ];

    directories_to_create.iter().for_each(|dir| {
        fs::create_dir(format!("./{}/{}", name, dir))
            .expect("Should be able to create main directories");
    });
}

#[derive(Serialize)]
struct Context {
    name: String,
}

static MAINRS_TEMPLATE: &'static str = include_str!("../../templates/web.main.rs.tt");
static CARGO_TEMPLATE: &'static str = include_str!("../../templates/web.Cargo.toml.tt");

fn create_main_file(name: &str) {
    let mut tt = TinyTemplate::new();
    tt.add_template("main.rs", MAINRS_TEMPLATE)
        .expect("Could not create main file template");

    let context = Context {
        name: name.to_string(),
    };

    let rendered = tt
        .render("main.rs", &context)
        .expect("Could not render main file template");

    let main_path_str = &format!("./{}/web/src/entry/main.rs", name);
    let main_path = Path::new(&main_path_str);
    fs::write(main_path, rendered).expect("Could not write main file");
}

fn create_cargo_file(name: &str) {
    let mut tt = TinyTemplate::new();
    tt.add_template("Cargo.toml", CARGO_TEMPLATE)
        .expect("Could not create Cargo.toml template");

    let context = Context {
        name: name.to_string(),
    };

    let rendered = tt
        .render("Cargo.toml", &context)
        .expect("Could not render Cargo.toml template");

    let path_str = &format!("./{}/web/Cargo.toml", name);
    let path = Path::new(&path_str);
    fs::write(path, rendered).expect("Could not write main file");
}
