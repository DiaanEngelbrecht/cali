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
        "web/tests",
        "web/tests/common",
        "store",
        "store/src",
        "store/migrations",
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

// Template static strings
static MAINRS_T: &'static str = include_str!("../../templates/web/main.rs.tt");
static CARGO_T: &'static str = include_str!("../../templates/web/Cargo.toml.tt");
static BUILD_T: &'static str = include_str!("../../templates/web/build.rs.tt");
static LIB_T: &'static str = include_str!("../../templates/web/lib.rs.tt");
static CONFIG_T: &'static str = include_str!("../../templates/web/config.rs.tt");
static CONFIG_EXAMPLE: &'static str = include_str!("../../templates/web/dev.yml.tt");
static CARGO_WORKSPACE_T: &'static str = include_str!("../../templates/Cargo.toml.tt");
static GITIGNORE_WORKSPACE_T: &'static str = include_str!("../../templates/.gitignore.tt");
static README_WORKSPACE_T: &'static str = include_str!("../../templates/README.md.tt");

fn create_files(name: &str) {
    let files = [
        (MAINRS_T, format!("./{}/web/src/entry/main.rs", name)),
        (CARGO_T, format!("./{}/web/Cargo.toml", name)),
        (BUILD_T, format!("./{}/web/build.rs", name)),
        (LIB_T, format!("./{}/web/src/lib.rs", name)),
        (CONFIG_T, format!("./{}/web/src/config.rs", name)),
        (CONFIG_EXAMPLE, format!("./{}/web/config/dev.yml", name)),
        (CARGO_WORKSPACE_T, format!("./{}/Cargo.toml", name)),
        (GITIGNORE_WORKSPACE_T, format!("./{}/.gitignore", name)),
        (README_WORKSPACE_T, format!("./{}/README.md", name)),
        ("", format!("./{}/web/src/controllers/mod.rs", name)),
        ("", format!("./{}/web/src/protos/mod.rs", name)),
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
