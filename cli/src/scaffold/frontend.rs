use std::{fs, path::Path, process::Command};

pub fn create_frontend(project_name: &str) {
    let frontend_path = format!("{}/frontend", project_name);

    println!("Creating frontend with SvelteKit...");

    // Create SvelteKit project using npx sv
    let status = Command::new("npx")
        .args(&[
            "sv",
            "create",
            &frontend_path,
            "--template",
            "minimal",
            "--types",
            "ts",
            "--no-add-ons",
        ])
        .status()
        .expect("Failed to create SvelteKit project");

    if !status.success() {
        eprintln!("Failed to create SvelteKit project");
        return;
    }

    println!("Installing frontend dependencies...");

    // Install Connect-Web and dev dependencies
    let install_status = Command::new("npm")
        .args(&[
            "install",
            "--prefix",
            &frontend_path,
            "@connectrpc/connect@^1.6.1",
            "@connectrpc/connect-web@^1.6.1",
            "@bufbuild/protobuf@^1.10.1",
        ])
        .status()
        .expect("Failed to install dependencies");

    if !install_status.success() {
        eprintln!("Failed to install dependencies");
        return;
    }

    let devinstall_status = Command::new("npm")
        .args(&[
            "install",
            "--save-dev",
            "--prefix",
            &frontend_path,
            "@bufbuild/protoc-gen-es@^1.10.1",
            "@connectrpc/protoc-gen-connect-es@^1.6.1",
            "prettier",
            "prettier-plugin-svelte",
            "eslint",
            "@typescript-eslint/eslint-plugin",
            "@typescript-eslint/parser",
            "eslint-plugin-svelte",
            "@eslint/js",
            "typescript-eslint",
            "globals",
        ])
        .status()
        .expect("Failed to install dev dependencies");

    if !devinstall_status.success() {
        eprintln!("Failed to install dev dependencies");
        return;
    }

    // Create necessary directories
    let scripts_path = format!("{}/scripts", frontend_path);
    fs::create_dir_all(&scripts_path).expect("Failed to create scripts directory");

    let lib_api_path = format!("{}/src/lib/api", frontend_path);
    fs::create_dir_all(&lib_api_path).expect("Failed to create lib/api directory");

    let generated_path = format!("{}/src/lib/generated", frontend_path);
    fs::create_dir_all(&generated_path).expect("Failed to create lib/generated directory");

    // Copy template files
    create_template_files(&frontend_path);

    println!("✓ Frontend created successfully!");
    println!("  Run 'cd {}/frontend && npm run generate' to generate TypeScript clients", project_name);
}

fn create_template_files(frontend_path: &str) {
    // Generate clients script
    let generate_script = include_str!("../../templates/frontend/generate-clients.sh.tt");
    let script_path = format!("{}/scripts/generate-clients.sh", frontend_path);
    fs::write(&script_path, generate_script).expect("Failed to write generate script");

    // Make script executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    // Client transport
    let client_ts = include_str!("../../templates/frontend/client.ts.tt");
    fs::write(
        format!("{}/src/lib/api/client.ts", frontend_path),
        client_ts,
    )
    .expect("Failed to write client.ts");

    // Update package.json with generate script
    let package_json_path = format!("{}/package.json", frontend_path);
    let package_json = fs::read_to_string(&package_json_path)
        .expect("Failed to read package.json");

    // Simple string replacement to add generate script
    let updated = package_json.replace(
        r#""scripts": {"#,
        r#""scripts": {
    "generate": "bash scripts/generate-clients.sh","#,
    );

    fs::write(&package_json_path, updated).expect("Failed to update package.json");

    println!("✓ Created frontend scaffold files");
}

pub fn generate_frontend_clients(project_root: Option<&str>) {
    // Try to find the frontend directory
    let frontend_path = if let Some(root) = project_root {
        format!("{}/frontend", root)
    } else {
        // Check if we're in the project root or in the frontend directory
        if Path::new("frontend/scripts/generate-clients.sh").exists() {
            "frontend".to_string()
        } else if Path::new("scripts/generate-clients.sh").exists() {
            ".".to_string()
        } else {
            eprintln!("Error: Could not find frontend directory");
            eprintln!("Run this command from either:");
            eprintln!("  - The project root (where frontend/ directory is)");
            eprintln!("  - Inside the frontend/ directory itself");
            return;
        }
    };

    let script_path = format!("{}/scripts/generate-clients.sh", frontend_path);

    if !Path::new(&script_path).exists() {
        eprintln!("Error: Frontend generation script not found at {}", script_path);
        eprintln!("Run 'cali new --frontend' to create a project with frontend support");
        return;
    }

    println!("Generating TypeScript clients from proto files...");

    // Resolve to absolute path for bash execution
    let abs_script_path = std::env::current_dir()
        .expect("Failed to get current directory")
        .join(&script_path);

    let status = Command::new("bash")
        .arg(&abs_script_path)
        .current_dir(&frontend_path)
        .status()
        .expect("Failed to run generation script");

    if status.success() {
        println!("✓ Successfully generated TypeScript clients");
    } else {
        eprintln!("✗ Failed to generate TypeScript clients");
    }
}
