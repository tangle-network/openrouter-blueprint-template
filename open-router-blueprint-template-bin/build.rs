use blueprint_sdk::build;
use blueprint_sdk::tangle::blueprint;
use open_router_blueprint_template_lib::jobs::process_llm_request;
use std::path::Path;
use std::process;

fn main() {
    let contracts_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("contracts");

    let contract_dirs: Vec<&str> = vec![contracts_dir.to_str().unwrap()];
    build::utils::soldeer_install();
    build::utils::soldeer_update();
    build::utils::build_contracts(contract_dirs);

    println!("cargo::rerun-if-changed=../open-router-blueprint-template-lib");

    let blueprint = blueprint! {
        name: "open-router-blueprint",
        master_manager_revision: "Latest",
        manager: { Evm = "OpenRouterBlueprint" },
        jobs: [process_llm_request]
    };

    match blueprint {
        Ok(blueprint) => {
            let json = blueprint_sdk::tangle::metadata::macros::ext::serde_json::to_string_pretty(
                &blueprint,
            )
            .unwrap();
            std::fs::write(
                Path::new(env!("CARGO_WORKSPACE_DIR")).join("blueprint.json"),
                json.as_bytes(),
            )
            .unwrap();
        }
        Err(e) => {
            println!("cargo::error={e:?}");
            process::exit(1);
        }
    }
}
