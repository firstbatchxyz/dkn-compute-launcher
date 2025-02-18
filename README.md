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
    <a href="./.github/workflows/release.yml" target="_blank">
        <img alt="Workflow: Release" src="https://github.com/firstbatchxyz/dkn-compute-launcher/actions/workflows/release.yml/badge.svg?branch=master">
    </a>
    <a href="./.github/workflows/tests.yml" target="_blank">
        <img alt="Workflow: Tests" src="https://github.com/firstbatchxyz/dkn-compute-launcher/actions/workflows/tests.yml/badge.svg?branch=master">
    </a>
    <a href="https://discord.gg/dria" target="_blank">
        <img alt="Discord" src="https://dcbadge.vercel.app/api/server/dria?style=flat">
    </a>
</p>

The **Dria Compute Launcher** is a simple and efficient way to set up and run the [Dria Compute Node](https://github.com/firstbatchxyz/dkn-compute-node). The launcher automatically handles environment setup, model selection, and binary management, making it easy to start the node with minimal configuration.

It is packed with many features:

- [x] **Environment Editor**: You can change various settings such as your wallet, ports and API keys, all without leaving the launcher. You can also open a raw text-editor in terminal.
- [x] **Model Selection**: You can choose your models with a nice menu.
- [x] **Model Benchmarking**: You can measure TPS for Ollama models to see if your machine can handle them.
- [x] **Automatic Updates**: Launcher will automatically update a running compute node when there is an update & restart it; furthermore, it will update & replace its own binary when there is a new launcher!
- [x] **Version Control**: You can select & run a specific compute node release.
- [x] **Auto-detect Ollama**: Launcher will check Ollama if you are using it's model, and start its server if required.

## Installation

You can download the latest executable for your operating system from:

- from [dria.co/join](https://dria.co/join)
- from [github](https://github.com/firstbatchxyz/dkn-compute-launcher/releases)
- via [cargo](https://www.rust-lang.org/) globally with `cargo install --git https://github.com/firstbatchxyz/dkn-compute-launcher`
- via [cargo]() locally with `cargo install --git https://github.com/firstbatchxyz/dkn-compute-launcher --root .`

> [!NOTE]
>
> The [minimum supported rust version](https://github.com/foresterre/cargo-msrv) (MSRV) for the launcher is `1.78.0`.

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

### Starting a Node

Start your node with `start` command:

```sh
./dkn-compute-launcher start
```

> [!NOTE]
>
> When you are running for the first time, the launcher will prompt you to fill in
> node information, such as your private key, chosen models and their respective provider information.

TODO: !!!

### Changing Settings

You can use the `settings` command to change anything about your node:

```sh
> ./dkn-compute-launcher settings
? Choose settings (for .env)
> Wallet
  Port
  Models
  Ollama
  API Keys
  Log Levels
  Save & Exit
```

With this command you are able to change:

- **Wallet**: change your secret key
- **Port**: edit your listen address port, defaults to `4001`
- **Models**: view all models & edit the models that you want to serve
- **Ollama**: edit host & port of the Ollama server
- **API Keys**: change API keys for providers
- **Log Levels**: change log-levels for modules within compute node

Within a menu, you can go back with the <kbd>ESC</kbd> key. At the top level, you must select **Save & Exit** to save your changes & write them to the environment file. If you ESC here without saving changes, your changes will be lost.

### Measuring Local Models

You can test your machine's performance on locally served Ollama models using the `bench` command:

```sh
./dkn-compute-launcher bench
```

Within Dria Knowledge Network, local models require you to reach a certain level of TPS. This command will measure your selected models, and then print a table of the results. We are interested in **Eval TPS** here, and if it's above the threshold the value will be written in green color; if it doesn't meet the requirement it will be printed in red color.

```sh
Model                                Prompt TPS   Eval TPS
phi3.5:3.8b                          13.1730      34.6144
deepseek-r1:1.5b                     10.9718      66.5954
```

### Update Manually

Using the `update` command you can check for updates & automatically update your compute node and launcher.

```sh
./dkn-compute-launcher update
```

You can check the launcher version

### Choosing a Specific Release

Using the `specific` command you can choose to run a specific release:

```sh
# select & download
./dkn-compute-launcher specific

# run after downloading
./dkn-compute-launcher specific --run

# specify tag, skipping the selection menu
./dkn-compute-launcher specific --run --tag 0.3.4
```

This is completely optional, and should mostly be used for debugging and testing on the live network. Note that when you run a specific release your node & launcher will not be automatically updated!

> [!CAUTION]
>
> The Dria Knowledge Network always considers the latest `minor` version as the active version; therefore, if the latest is `0.3.x` and you decide to run a smaller version like `0.2.x` you will most likely kept out of network due to protocol mismatch.

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/license/Apache-2.0).
