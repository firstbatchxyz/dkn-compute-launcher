<p align="center">
  <img src="https://raw.githubusercontent.com/firstbatchxyz/.github/refs/heads/master/branding/dria-logo-square.svg" alt="logo" width="168">
</p>

<p align="center">
  <h1 align="center">
    Dria Compute Launcher
  </h1>
  <p align="center">
    <i>A version manager, settings editor, and launcher for Dria Compute Node!</i>
  </p>
</p>

<p align="center">
    <a href="https://opensource.org/license/apache-2-0" target="_blank">
        <img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache%202.0-7CB9E8.svg">
    </a>
    <a href="./.github/workflows/test.yml" target="_blank">
        <img alt="Workflow: Tests" src="https://github.com/firstbatchxyz/dkn-compute-node/actions/workflows/tests.yml/badge.svg?branch=master">
    </a>
    <a href="https://discord.gg/dria" target="_blank">
        <img alt="Discord" src="https://dcbadge.vercel.app/api/server/dria?style=flat">
    </a>
</p>

## About

The **Dria Compute Launcher** is a simple and efficient way to set up and run the [Dria Compute Node](https://github.com/firstbatchxyz/dkn-compute-node). The launcher automatically handles environment setup, model selection, and binary management, making it easy to start the node with minimal configuration.

It is packed with some nice features:

- **Environment Editor**: You can change various settings such as your wallet, ports and API keys, all without leaving the launcher. You can also open a raw text-editor in terminal.

- **Model Selection**: You can choose your models with a nice multi-select menu.

- **Auto-Version Control**: Launcher will automatically update a running compute node when there is an update & restart it; furthermore, it will update & replace its own binary when there is a new launcher!

<!-- TODO: ollama checks? -->



## Installation

## Usage

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

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/license/Apache-2.0).
