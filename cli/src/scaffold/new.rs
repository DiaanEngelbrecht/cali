use serde::Serialize;
use std::{fs, path::Path};

use tinytemplate::TinyTemplate;

pub fn create_app(name: &str) {
    create_directories(name);
    create_files(name);
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
        "store",
        "store/src",
        "store/src/repositories",
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

static MAINRS_TEMPLATE: &'static str = include_str!("../../templates/web/main.rs.tt");
static CARGO_TEMPLATE: &'static str = include_str!("../../templates/web/Cargo.toml.tt");
static BUILD_TEMPLATE: &'static str = include_str!("../../templates/web/build.rs.tt");
static LIB_TEMPLATE: &'static str = include_str!("../../templates/web/lib.rs.tt");
static CONFIG_TEMPLATE: &'static str = include_str!("../../templates/web/config.rs.tt");

fn create_files(name: &str) {
    let files = [
        (MAINRS_TEMPLATE, format!("./{}/web/src/entry/main.rs", name)),
        (CARGO_TEMPLATE, format!("./{}/web/Cargo.toml", name)),
        (BUILD_TEMPLATE, format!("./{}/web/build.rs", name)),
        (LIB_TEMPLATE, format!("./{}/web/src/lib.rs", name)),
        (CONFIG_TEMPLATE, format!("./{}/web/src/config.rs", name)),
    ];
    let context = Context {
        name: name.to_string(),
    };

    files.iter().for_each(|(content, path)| {
        let rendered = create_template_file(content, &context);
        let main_path = Path::new(&path);
        fs::write(main_path, rendered).expect("Could not write main file");
    });
}

fn create_template_file<T: Serialize>(template: &str, ctx: &T) -> String {
    let mut tt = TinyTemplate::new();
    tt.add_template("t", template)
        .expect("Could not create template");

    tt.render("t", ctx).expect("Could not render template")
}
