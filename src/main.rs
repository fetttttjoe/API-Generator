use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
mod endpoint;

#[tokio::main]
async fn main() {
    // Get the JSON file path from the command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <JSON_FILE>");
        return;
    }

    let json_file_path = &args[1];

    // Read the JSON file
    let mut file = File::open(json_file_path).expect("Failed to open the file.");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read the file.");

    // Parse JSON into Vec<Endpoint>
    let endpoints: Vec<endpoint::Endpoint> =
        serde_json::from_str(&contents).expect("Failed to parse JSON.");

    let generated_code = format!(
        r#"
{}

{}
pub fn generate_router() -> Router {{
    let mut router = Router::new();
    router = router.route("/", get(root_handler));
        {}
    return router;
}}

#[tokio::main]
async fn main() {{
    // initialize tracing
    tracing_subscriber::fmt::init();
    let app = generate_router();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // run it with hyper on localhost:3000
    println!("Listening on http://{{}}", addr);
    tracing::debug!("listening on {{}}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}}"#,
        generate_imports(),
        generate_route_handlers(&endpoints),
        generate_routes(&endpoints)
    );

    // Write the generated code to a new file
    let output_folder = "output";
    fs::create_dir_all(output_folder).expect("Failed to create output folder.");
    // Write the Cargo File to the output folder
    write_cargo_toml(output_folder);
    let output_src_folder = format!("{}/src", output_folder);
    fs::create_dir_all(output_src_folder.clone()).expect("Failed to create src folder.");
    let output_file_path = format!("{}/main.rs", output_src_folder);
    fs::write(&output_file_path, generated_code).expect("Failed to write the generated file.");
    // Generate and write the Cargo.toml file
    println!("Generated server code written to {}", output_file_path);
}
use std::io::Write;

fn generate_cargo_toml() -> String {
    r#"
[package]
name = "api_template"
version = "0.0.1"
edition = "2021"

[dependencies]
axum = { version = "0.6.19" } 
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
serde_json = "1.0"
"#
    .to_string()
}

fn write_cargo_toml(output_folder: &str) {
    let cargo_toml_content = generate_cargo_toml();
    let cargo_toml_file_path = format!("{}/Cargo.toml", output_folder);
    let mut cargo_toml_file =
        File::create(&cargo_toml_file_path).expect("Failed to create Cargo.toml file.");
    cargo_toml_file
        .write_all(cargo_toml_content.as_bytes())
        .expect("Failed to write to Cargo.toml file.");
    println!("Cargo.toml file written to {}", cargo_toml_file_path);
}

fn generate_route_handlers(endpoints: &[endpoint::Endpoint]) -> String {
    let mut code = String::new();
    code.push_str(&format!(
        r#"
async fn root_handler() -> &'static str {{
    "Server is up and running!"
}}"#,
    ));
    for endpoint in endpoints {
        match endpoint.method.as_str() {
            "GET" => {
                code.push_str(&format!(
                    r#"
async fn {}_handler() -> &'static str {{
    "Hello from {} handler. Using {}!"
}}"#,
                    generate_handler_name(&endpoint.route),
                    endpoint.purpose,
                    endpoint.method,
                ));
            }
            "POST" => {
                code.push_str(&format!(
                    r#"
async fn {}_handler() -> &'static str {{
    "I am a {} handler. My Purpose is: {}!"
}}"#,
                    generate_handler_name(&endpoint.route),
                    endpoint.method,
                    endpoint.purpose,
                ));
            }
            // Add other methods (PUT, DELETE, etc.) as needed
            _ => {
                eprintln!("Unsupported HTTP method: {}", endpoint.method);
            }
        }
    }
    return code;
}

fn generate_routes(endpoints: &[endpoint::Endpoint]) -> String {
    let mut code = String::new();
    for endpoint in endpoints {
        match endpoint.method.as_str() {
            "GET" => {
                code.push_str(&format!(
                    r#"
    router = router.route("{}", get({}_handler));"#,
                    endpoint.route,
                    generate_handler_name(&endpoint.route)
                ));
            }
            "POST" => {
                code.push_str(&format!(
                    r#"
    router = router.route("{}", post({}_handler));"#,
                    endpoint.route,
                    generate_handler_name(&endpoint.route)
                ));
            }
            // Add other methods (PUT, DELETE, etc.) as needed
            _ => {
                eprintln!("Unsupported HTTP method: {}", endpoint.method);
            }
        }
    }
    code
}

fn generate_handler_name(route: &str) -> String {
    route.replace("/", "").replace(":", "_")
}

fn generate_imports() -> String {
    return String::from(
        "
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;",
    );
}
