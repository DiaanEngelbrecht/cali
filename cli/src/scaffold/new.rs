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
        "core",
        "core/src"
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
static WEB_MAINRS_T: &'static str = include_str!("../../templates/web/src/entry/main.rs.tt");
static WEB_CARGO_T: &'static str = include_str!("../../templates/web/Cargo.toml.tt");
static WEB_BUILD_T: &'static str = include_str!("../../templates/web/build.rs.tt");
static WEB_LIB_T: &'static str = include_str!("../../templates/web/src/lib.rs.tt");
static WEB_CONFIG_T: &'static str = include_str!("../../templates/web/src/config.rs.tt");
static WEB_DEV_CONFIG_EXAMPLE: &'static str = include_str!("../../templates/web/config/dev.yml.tt");
static WEB_TEST_CONFIG_EXAMPLE: &'static str = include_str!("../../templates/web/config/test.yml.tt");
static WEB_CONTROLLER_MOD_T: &'static str = include_str!("../../templates/web/src/controllers/mod.rs.tt");
static CORE_CARGO_T: &'static str = include_str!("../../templates/core/Cargo.toml.tt");
static CORE_LIB_T: &'static str = include_str!("../../templates/core/src/lib.rs.tt");
static STORE_CARGO_T: &'static str = include_str!("../../templates/store/Cargo.toml.tt");
static STORE_LIB_T: &'static str = include_str!("../../templates/store/src/lib.rs.tt");
static STORE_REP_MOD_T: &'static str = include_str!("../../templates/store/src/repositories/mod.rs.tt");
static CARGO_WORKSPACE_T: &'static str = include_str!("../../templates/Cargo.toml.tt");
static GITIGNORE_WORKSPACE_T: &'static str = include_str!("../../templates/.gitignore.tt");
static README_WORKSPACE_T: &'static str = include_str!("../../templates/README.md.tt");

fn create_files(name: &str) {
    let files = [
        (WEB_MAINRS_T, format!("./{}/web/src/entry/main.rs", name)),
        (WEB_CARGO_T, format!("./{}/web/Cargo.toml", name)),
        (WEB_BUILD_T, format!("./{}/web/build.rs", name)),
        (WEB_LIB_T, format!("./{}/web/src/lib.rs", name)),
        (WEB_CONFIG_T, format!("./{}/web/src/config.rs", name)),
        (WEB_DEV_CONFIG_EXAMPLE, format!("./{}/web/config/dev.yml", name)),
        (WEB_CONTROLLER_MOD_T, format!("./{}/web/src/controllers/mod.rs", name)),
        (
            WEB_TEST_CONFIG_EXAMPLE,
            format!("./{}/web/config/test.yml", name),
        ),
        (CORE_CARGO_T, format!("./{}/core/Cargo.toml", name)),
        (CORE_LIB_T, format!("./{}/core/src/lib.rs", name)),
        (STORE_CARGO_T, format!("./{}/store/Cargo.toml", name)),
        (STORE_LIB_T, format!("./{}/store/src/lib.rs", name)),
        (STORE_REP_MOD_T, format!("./{}/store/src/repositories/mod.rs", name)),
        (CARGO_WORKSPACE_T, format!("./{}/Cargo.toml", name)),
        (GITIGNORE_WORKSPACE_T, format!("./{}/.gitignore", name)),
        (README_WORKSPACE_T, format!("./{}/README.md", name)),
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
