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

The **Dria Compute Launcher** is a simple and efficient way to set up and run the [Dria Compute Node](https://github.com/firstbatchxyz/dkn-compute-node). The launcher automatically handles environment setup, model selection, and binary management, making it easy to start the node with minimal configuration.

It is packed with many features:

- [x] **Environment Editor**: You can change various settings such as your wallet, ports and API keys, all without leaving the launcher. You can also open a raw text-editor in terminal.
- [x] **Model Selection**: You can choose your models with a nice multi-select menu.
- [x] **Automatic Updates**: Launcher will automatically update a running compute node when there is an update & restart it; furthermore, it will update & replace its own binary when there is a new launcher!
- [x] **Version Control**: You can select & run a specific compute node release.
- [x] **Auto-detect Ollama**: Launcher will check Ollama if you are using it's model, and start its server if required.

## Installation

You can download the latest executable for your operating system from:

- from [dria.co/join](https://dria.co/join)
- from [github](https://github.com/firstbatchxyz/dkn-compute-launcher/releases)
- via [cargo](https://www.rust-lang.org/) globally with `cargo install --git https://github.com/firstbatchxyz/dkn-compute-launcher`
- via [cargo]() locally with `cargo install --git https://github.com/firstbatchxyz/dkn-compute-launcher --root .`

## Usage

Double-click the executable or run it via the command line. The `help` to see available options:

```sh
# with cargo
dkn-compute-launcher help

# macos or linux
./dkn-compute-launcher help

# windows
.\dkn-compute-launcher.exe help
```

> [!CAUTION]
>
> > Some Apple devices need you to bypass macOS's security warning. If you see "macOS cannot verify that this app is free from malware" when using the launcher use the following command:
>
> ```sh
> xattr -d com.apple.quarantine dkn-compute-launcher
> ```

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/license/Apache-2.0).
