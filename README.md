# <h1 align="center">OpenRouter Blueprint Template for Tangle 🌐</h1>

## 📚 Overview

This Tangle Blueprint provides a template for creating an OpenRouter provider that can support any locally hosted LLM. It enables Tangle to act as a load balancer for multiple LLM instances, allowing operators to participate in the network by running LLM nodes.

Blueprints are specifications for <abbr title="Actively Validated Services">AVS</abbr>s on the Tangle Network. An AVS is an off-chain service that runs arbitrary computations for a user-specified period of time.

This blueprint template allows Tangle to serve as a provider on OpenRouter, balancing requests across locally hosted LLMs running on blueprints for people accessing the models through Tangle.

For more details, please refer to the [project documentation](https://docs.tangle.tools/developers/blueprints/introduction).

## 🚀 Features

- Standardized interface for local LLMs to connect to Tangle
- Support for chat completions, text completions, and embeddings
- Metrics reporting for load balancing
- Compatible with OpenAI/OpenRouter API formats
- Extensible design to support various LLM frameworks

## 📋 Prerequisites

Before you can run this project, you will need to have the following software installed on your machine:

- [Rust](https://www.rust-lang.org/tools/install)
- [Forge](https://getfoundry.sh)

You will also need to install [cargo-tangle](https://crates.io/crates/cargo-tangle), our CLI tool for creating and
deploying Tangle Blueprints:

To install the Tangle CLI, run the following command:

> Supported on Linux, MacOS, and Windows (WSL2)

```bash
cargo install cargo-tangle --git https://github.com/tangle-network/blueprint
```

## ⭐ Getting Started

Once `cargo-tangle` is installed, you can create a new project with the following command:

```sh
cargo tangle blueprint create --name <project-name>
```

and follow the instructions to create a new project.

## 🛠️ Development

### Project Structure

The blueprint follows the standard Tangle Blueprint structure:

- `open-router-blueprint-template-bin/`: Binary crate for initializing the blueprint
- `open-router-blueprint-template-lib/`: Library crate containing the core logic
  - `src/llm/`: LLM interface abstraction and implementations
  - `src/context.rs`: Context structure for the blueprint
  - `src/jobs.rs`: Job handlers for processing LLM requests

### Building and Testing

To build the project:

```sh
cargo build
```

To run the tests:

```sh
cargo test
```

### Deployment

To deploy the blueprint to the Tangle network:

```sh
cargo tangle blueprint deploy
```

### Customizing for Your LLM

To customize this blueprint for your specific LLM implementation:

1. Create a new implementation of the `LlmClient` trait in `src/llm/`
2. Update the context initialization in `src/context.rs` to use your implementation
3. Modify the configuration as needed for your LLM

## 📜 License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 📬 Feedback and Contributions

We welcome feedback and contributions to improve this blueprint.
Please open an issue or submit a pull request on our GitHub repository.
Please let us know if you fork this blueprint and extend it too!

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
