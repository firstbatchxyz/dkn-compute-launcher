<p align="center">
  <img src="https://raw.githubusercontent.com/firstbatchxyz/.github/refs/heads/master/branding/dria-logo-square.svg" alt="dria-logo" width="168">
</p>

<p align="center">
  <h1 align="center">
    Dria Compute Launcher
  </h1>
  <p align="center">
    <i>A launcher, editor, and version manager for Dria Compute Node.</i>
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
    <a href="https://github.com/firstbatchxyz/dkn-compute-launcher/releases" target="_blank">
        <img alt="Downloads" src="https://img.shields.io/github/downloads/firstbatchxyz/dkn-compute-launcher/total?logo=github&logoColor=%23F2FFEE&color=%2332C754">
    </a>
    <a href="https://discord.gg/dria" target="_blank">
        <img alt="Discord" src="https://dcbadge.vercel.app/api/server/dria?style=flat">
    </a>
</p>

The **Dria Compute Launcher** is a simple and efficient way to set up and run the [Dria Compute Node](https://github.com/firstbatchxyz/dkn-compute-node). The launcher automatically handles environment setup, model selection, and binary management, making it easy to start the node with minimal configuration.

It is packed with many features:

- [x] **Settings Menu**: You can change various settings such as your wallet, ports and API keys, all without leaving the launcher. You can also open a raw text-editor in terminal.
- [x] **Model Selection**: You can choose your models with a nice menu.
- [x] **Model Benchmarking**: You can measure TPS for Ollama models to see if your machine can handle them.
- [x] **Automatic Updates**: Launcher will automatically update a running compute node when there is an update & restart it; furthermore, it will update itself when there is a new launcher as well!
- [x] **Version Control**: You can select & run a specific compute node release.
- [x] **Auto-detect Ollama**: Launcher will check Ollama if you are using it's model, and start its server if required.

## Installation

The recommended way to install the launcher is to run the command below in your terminal based on your machine:

**Linux / MacOS**

```sh
curl -fsSL https://dria.co/launcher | bash
```

**Windows**

```sh
powershell -c "irm dria.co/launcher.ps1 | iex"
```

That's all!

### Download Specific Release

You can visit [GitHub releases](https://github.com/firstbatchxyz/dkn-compute-launcher/releases) and download a specific release for your machine.

### Build from Source

You can build from source using [Rust](https://www.rust-lang.org/) & install the launcher globally using the command below:

```sh
cargo install --git https://github.com/firstbatchxyz/dkn-compute-launcher
```

Note that the [minimum supported rust version](https://github.com/foresterre/cargo-msrv) (MSRV) for the launcher is `1.78.0`.

## Usage

Double-click the executable or run it via the command line. The `help` to see available options:

```sh
# as a Unix executable
./dkn-compute-launcher help

# as a Windows executable
.\dkn-compute-launcher.exe help
```

> [!CAUTION]
>
> Some Apple devices need you to bypass macOS's security warning. If you see "macOS cannot verify that this app is free from malware" when using the launcher use the following command:
>
> ```sh
> xattr -d com.apple.quarantine dkn-compute-launcher
> ```

All commands that you see with `help` have their own help messages within as well, you can view it with:

```sh
./dkn-compute-launcher <some-command> --help
```

### Model Providers

The purpose of running a Dria Compute Node is to serve LLMs to the network. These models can either be locally-hosted models via Ollama, or API-based models such as Gemini and OpenAI.

= To serve API-based models ([OpenAI](https://openai.com/), [Gemini](https://gemini.google.com/app), [OpenRouter](https://openrouter.ai/)), you will need to get their API keys.

- To serve a locally-hosted model with [Ollama](https://ollama.com/), you of course need Ollama installed, and you must make sure that your machine can handle your chosen models. See ["Measuring Local Models"](#measuring-local-models) chapter below to see the command-line tools that help you measure TPS.

### Starting a Node

Start your node with `start` command:

```sh
./dkn-compute-launcher start
```

> [!NOTE]
>
> When you are running for the first time, the launcher will prompt you to fill in
> node information, such as your private key, chosen models and their respective provider information.

You can stop the node with <kbd>CTRL+C</kbd> (on Linux / Windows) or <kbd>CMD+C</kbd> (on macOS)

### Changing Settings

You can use the `settings` command to change anything about your node:

```sh
./dkn-compute-launcher settings
```

You will be greeted with a menu, where you can navigate with arrow keys <kbd>↑</kbd> <kbd>↓</kbd> and select an option with enter <kbd>ENTER</kbd> :

```yaml
? Choose settings (for .env)
> Wallet
  Port
  Models
  Ollama
  API Keys
  Log Levels
  ✓ Save & Exit
```

Using this menu, you are able to change the following settings:

- **Wallet**: change your secret key
- **Port**: edit your listen address port, defaults to `4001`
- **Models**: view all models & edit the models that you want to serve
- **Ollama**: edit host & port of the Ollama server
- **API Keys**: change API keys for providers
- **Log Levels**: change log-levels for modules within compute node & launcher

Within a menu, you can go back by selecting <kbd>← Go Back</kbd>. Within the main menu, you can select <kbd>✓ Save & Exit</kbd> to save your changes & write them to the environment file, or you can choose <kbd>✗ Abort Changes</kbd> to abort all changes.

> [!TIP]
>
> You can always exit the process (ungracefully) with <kbd>CTRL+C</kbd> (on Linux / Windows) or <kbd>CMD+C</kbd> (on macOS).

### Choosing Models

When you select <kbd>Model</kbd> option in the Settings menu, you will be greeted with a list of model providers:

```sh
? Select a model provider:
> ollama
  openai
  gemini
  openrouter
  VLLM
  ← Go Back
```

Here, you can select a provider to choose models served by them, where you will be greeted with the following menu:

```sh
> Select a model provider: openai
? Choose your models with SPACE, then press ENTER:
  [ ] gpt-4-turbo
  [x] gpt-4o
> [ ] gpt-4o-mini
  [ ] o1-mini
  [ ] o1-preview
  [ ] o1
  [ ] o3-mini
# ...
```

Within this menu you can navigate by using the arrow keys <kbd>↑</kbd> <kbd>↓</kbd> and press <kbd>SPACE</kbd> to select a model. You can select all models using <kbd>→</kbd>, or de-select everything with <kbd>←</kbd>. To finish selecting models, press <kbd>ENTER</kbd>.

When you are done selecting models for all providers, you can go back to the main menu by selecting <kbd>← Go Back</kbd>.

### Measuring Local Models

You can test your machine's performance on locally served Ollama models using the `measure` command:

```sh
./dkn-compute-launcher measure
```

Within Dria Knowledge Network, local models require you to reach a certain level of TPS. This command will measure your selected models, and then print a table of the results. We are particularly interested in **Eval TPS** and **Total (ms)** for our model performance.

```sh
Model                                Prompt TPS   Time (ms)    Eval TPS     Time (ms)    Total (ms)
qwen2.5-coder:1.5b                   40.7747      981          67.9260      2488         3496
deepseek-r1:1.5b                     21.4724      652          63.3591      16588        17255
driaforall/tiny-agent-a:1.5b         22.5653      842          47.1771      2586         3443
```

### Update Manually

Using the `update` command you can check for updates & automatically update your compute node and launcher.

```sh
./dkn-compute-launcher update
```

You don't need to do this usually, as the launcher will always check for updates when you run the `start` command.

### Environment Editor

For more advanced users that would like to view the environment file in more detail & plain-text, we provide the `env-editor` command:

```sh
./dkn-compute-launcher env-editor
```

This command will open the selected environment file using a terminal-native text editor, allowing you to edit everything in it. If there happens to be multiple keys for a single value in the environment, the `settings` command will edit the _last uncommented key_ on **Save**.

### Running a Specific Release

Using the `specific` command you can choose to run a specific release:

```sh
# select & download
./dkn-compute-launcher specific

# run after downloading
./dkn-compute-launcher specific --run

# specify tag, skipping the selection menu
./dkn-compute-launcher specific --run --tag 0.3.4
```

This is completely optional, and should mostly be used for debugging and testing on the live network. When you run a specific release your node & launcher will **not** be automatically updated!

> [!CAUTION]
>
> The Dria Knowledge Network always considers the latest `minor` version as the active version; therefore, if the latest is `0.3.x` and you decide to run a smaller version like `0.2.x` you will most likely kept out of network due to protocol mismatch.

## Documentation

To see the launcher's internal documentation clone the repo and run:

```sh
cargo doc --open --no-deps
```

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/license/Apache-2.0).
