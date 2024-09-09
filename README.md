<p align="center">
  <img src="https://raw.githubusercontent.com/firstbatchxyz/dria-js-client/master/logo.svg" alt="logo" width="142">
</p>

<h1 align="center">
  Dria Compute Node - Launcher
</h1>
<p align="center">
  <i>Dria Compute Node Launcher for easily starting the node.</i>
</p>

## About

This repository contains the CLI application for the [DKN Compute Node](https://github.com/firstbatchxyz/dkn-compute-node). It provides a simple and efficient way to set up and run the Dria Compute Node. The launcher automatically handles environment setup, model selection, and binary management, making it easy to start the node with minimal configuration.

## Features

- **Environment Setup:** Automatically loads and manages environment variables.
- **Model Selection:** Supports both predefined models from OpenAI and Ollama. Users can choose models interactively or via command-line flags.
- **Binary Management:** The launcher automatically pulls the latest binary from the main repository and checks for updates each time it runs.
- **Flexible Logging:** Allows setting different logging levels for easier debugging and monitoring.
- **Background/Foreground Modes:** Run the node in either background or foreground mode based on your preference.

## Quick Start

1. **Download the executable**: Simply download the latest executable for your operating system from the [releases page](https://github.com/firstbatchxyz/dkn-compute-launcher/releases/tag/v0.0.1).

2. **Run the Launcher**: Double-click the executable or run it via the command line. Use `-h` or `--help` to see all available options.

   ```sh
   # macos or linux
   ./dkn-compute-launcher --help

   # windows
   .\dkn-compute-launcher.exe --help
   ```

> [!TIP]
>
> > Some Apple devices need you to bypass macOS's security warning. If you see "macOS cannot verify that this app is free from malware" when using the launcher use the following command:
>
> ```sh
> xattr -d com.apple.quarantine dkn-compute-launcher
> ```


3. **Select Models**: Choose the models to be used in your node setup. You can pass multiple `-m` or `--model` flags or use `--pick-models` flag for interactive model picker.

4. **Start the Node**: Follow the prompts to complete the setup and start your node.

## More Information

For a detailed guide on running the Dria Compute Node and its full capabilities, please refer to the [DKN Compute Node repository](https://github.com/firstbatchxyz/dkn-compute-node/blob/master/docs/NODE_GUIDE.md).
