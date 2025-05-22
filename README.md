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
    <a href="./.github/workflows/tests.yml" target="_blank">
        <img alt="Workflow: Tests" src="https://github.com/firstbatchxyz/dkn-compute-launcher/actions/workflows/tests.yml/badge.svg?branch=master">
    </a>
    <a href="https://github.com/firstbatchxyz/dkn-compute-launcher/releases" target="_blank">
        <img alt="Downloads" src="https://img.shields.io/github/downloads/firstbatchxyz/dkn-compute-launcher/total?logo=github&logoColor=%23F2FFEE&color=%2332C754">
    </a>
    <a href="https://github.com/foresterre/cargo-msrv" target="_blank">
        <img alt="MSRV" src="https://img.shields.io/badge/1.81.0-F74B01?logo=rust&logoColor=white&label=msrv"/>
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

**Linux / MacOS**

Open a terminal and run the following command:

```sh
curl -fsSL https://dria.co/launcher | bash
```

You can verify the installation later with:

```sh
which dkn-compute-launcher
```

**Windows**

Open a Windows terminal ([cmd.exe](https://en.wikipedia.org/wiki/Cmd.exe)) and run the following command:

```sh
powershell -c "irm https://dria.co/launcher.ps1 | iex"
```

You can verify the installation later with:

```sh
where.exe dkn-compute-launcher
```

You may need to allow network access to the launcher if Windows prompts you to do so.

### Build from Source

You can build from source using [Rust](https://www.rust-lang.org/) & install the launcher globally using the command below:

```sh
cargo install --git https://github.com/firstbatchxyz/dkn-compute-launcher --locked
```

The [minimum supported rust version](https://github.com/foresterre/cargo-msrv) (MSRV) for the launcher is `1.81.0`. We use the `--locked` option to ensure you use the existing lockfile that is guaranteed to build.

## Usage

Double-click the executable or run it via the command line. The `help` to see available options:

```sh
# as a Unix executable
dkn-compute-launcher help

# as a Windows executable
dkn-compute-launcher.exe help
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
dkn-compute-launcher <some-command> --help
```

### Model Providers

The purpose of running a Dria Compute Node is to serve LLMs to the network. These models can either be locally-hosted models via Ollama, or API-based models such as Gemini and OpenAI.

- To serve API-based models ([OpenAI](https://openai.com/), [Gemini](https://gemini.google.com/app), [OpenRouter](https://openrouter.ai/)), you will need to get their API keys.

- To serve a locally-hosted model with [Ollama](https://ollama.com/), you of course need Ollama installed, and you must make sure that your machine can handle your chosen models. See ["Measuring Local Models"](#measuring-local-models) chapter below to see the command-line tools that help you measure TPS.

### Starting a Node

Start your node with `start` command:

```sh
dkn-compute-launcher start
```

> [!NOTE]
>
> When you are running for the first time, the launcher will prompt you to fill in
> node information, such as your private key, chosen models and their respective provider information.

You can stop the node with <kbd>CTRL+C</kbd> (on Linux / Windows) or <kbd>CMD+C</kbd> (on macOS)

### Referrals Program

You can earn $DRIA points if you refer other users! When you refer a user, for each point they earn you earn a portion of those points as well.
To get a referral code, enter someone's referral code and such, use the following command:

```sh
dkn-compute-launcher referrals
```

> [!CAUTION]
>
> Each referral code only has 5 uses! Once you have referred 5 users, your code will no longer work.

### Changing Settings

You can use the `settings` command to change anything about your node:

```sh
dkn-compute-launcher settings
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
  ✗ Abort Changes
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
> You can always exit the process (ungracefully) with <kbd>CTRL+C</kbd> (on Linux / Windows) or <kbd>CMD+C</kbd> (on macOS), or <kbd>ESC</kbd> on both systems.

### Models Menu

When you select <kbd>Model</kbd> option in the Settings menu, you will be greeted with another menu:

```py
? Choose model settings:
> Edit model selection
  List chosen models
  Remove local models
  Measure local models
```

#### Selecting Models

Click on `Edit model selection` to select models for your node.

```sh
? Select a model provider:
> ollama
  openai
  gemini
  openrouter
  ← Go Back
```

Here, you can select a provider to choose models served by them, where you will be greeted with the following menu:

```sh
> Select a model provider: ollama
? Choose your models with SPACE, then press ENTER:
  [ ] llama3.1:8b-instruct-q4_K_M
  [ ] llama3.2:1b-instruct-q4_K_M
  [ ] llama3.3:70b-instruct-q4_K_M
> [ ] mistral-nemo:12b
  [ ] gemma3:4b
  [ ] gemma3:12b
  [ ] gemma3:27b
# ...
```

Within this menu you can navigate by using the arrow keys <kbd>↑</kbd> <kbd>↓</kbd> and press <kbd>SPACE</kbd> to select a model. You can select all models using <kbd>→</kbd>, or de-select everything with <kbd>←</kbd>. To finish selecting models, press <kbd>ENTER</kbd>.

When you are done selecting models for all providers, you can go back to the main menu by selecting <kbd>← Go Back</kbd>.

> [!TIP]
>
> You can pick `List chosen models` to show the list of models that you have picked for all providers.

#### Removing Local Models

When you run a node with local models, they are pulled to your machine and are stored within Ollama's files. Our launcher also provides a shortcut for that, you can pick the `Remove local models` option to choose models and remove them from your machine.

> [!TIP]
>
> You can also remove them using Ollama commands:
>
> ```sh
> # show downloaded models
> ollama ls
>
> # remove a model
> ollama rm "model-name-here"
> ```

#### Measuring Local Models

You can test your machine's performance on locally served Ollama models by picking the `Measure local models` option.

Within Dria Knowledge Network, local models require you to reach a certain level of TPS. This command will measure your selected models, and then print a table of the results. We are particularly interested in **Eval TPS** and **Total (ms)** for our model performance.

```sh
Model                                Prompt TPS   Time (ms)    Eval TPS     Time (ms)    Total (ms)
qwen2.5-coder:1.5b                   40.7747      981          67.9260      2488         3496
deepseek-r1:1.5b                     21.4724      652          63.3591      16588        17255
driaforall/tiny-agent-a:1.5b         22.5653      842          47.1771      2586         3443
```

Measurements the fail to meet the Compute Node requirements will be colored in red.

### Displaying $DRIA Points

Use the `points` command to display how much you have earned!

```sh
dkn-compute-launcher points
```

You can also check out your [node dashboard](https://dria.co/edge-ai) for this information.

### Updating Manually

Using the `update` command you can check for updates & automatically update your compute node and launcher.

```sh
dkn-compute-launcher update
```

You don't need to do this usually, as the launcher will always check for updates when you run the `start` command.

### Editing Environment File

For more advanced users that would like to view the environment file in more detail & plain-text, we provide the `env-editor` command:

```sh
dkn-compute-launcher env-editor
```

This command will open the selected environment file using a terminal-native text editor, allowing you to edit everything in it. If there happens to be multiple keys for a single value in the environment, the `settings` command will edit the _last uncommented key_ on **Save**.

### Running a Specific Release

Using the `specific` command you can choose to run a specific release:

```sh
# select & download
dkn-compute-launcher specific

# run after downloading
dkn-compute-launcher specific --run

# specify tag, skipping the selection menu
dkn-compute-launcher specific --run --tag 0.3.4
```

This is completely optional, and should mostly be used for debugging and testing on the live network.
When you run a specific release your node & launcher will **not** be automatically updated!

> [!CAUTION]
>
> The Dria Knowledge Network always considers the latest `minor` version as the active version; therefore,
> if the latest is `0.3.x` and you decide to run a smaller version like `0.2.x` you will most likely kept out of network due to protocol mismatch.

### Running in Background

#### Linux/MacOS

In Linux/MacOS systems you can use [`screen`](https://gist.github.com/jctosta/af918e1618682638aa82) command to run the launcher in the background.

First, create a screen with a given name (here we name it `dkn-compute-node`):

```sh
screen -S dkn-compute-node
```

Within the newly opened screen, start the node:

```sh
dkn-compute-launcher start
```

Now we will _detach_ from this screen and let it run in the background. For this, press <kbd>CTRL + A</kbd> and then press the <kbd>D</kbd> letter. You should now exit the screen, and see a `[detached]` log in the terminal that you have returned to.

At a later time, you can list your screens with:

```sh
screen -list
```

You can connect to a screen via its name directly:

```sh
screen -r dkn-compute-node
```

Within the screen, you can continue to use your launcher as you would normally, or stop the node with <kbd>CTRL+C</kbd>. You can `exit` within the screen to terminate it.

<!--
TODO: test these commands and then publish them here
#### Windows

In Windows systems, you can start the launcher in background using the `start` command:

```cmd
start /B dkn-compute-launcher.exe start
```

To list running processes:

```cmd
tasklist | findstr "dkn-compute"
```

To terminate the process:

```cmd
taskkill /IM dkn-compute-launcher.exe /F
``` -->

## Contributions

Contributions are welcome! You can start by cloning the repo:

```sh
git clone https://github.com/firstbatchxyz/dkn-compute-launcher
```

### Development

The code is pretty laid-out, all commands are under [`commands`](./src/commands/) and settings-related code is under [`settings`](./src/settings/).

> [!NOTE]
>
> When the code is not `--release` mode (e.g. with `cargo run`), the used `.env` file will default to the local file, instead of the one under home directory, and launcher updates
> will be disabled so that you can work on your launcher without version mismatches.

### Documentation

To see the launcher's internal documentation you can run:

```sh
cargo doc --open --no-deps --document-private-items
```

## Uninstallation

You can uninstall the launcher binary along with the environment files and compute node binaries with the `uninstall` command.

Make sure you backup your private key within `.env` before removing these files, so that you do not lose your hard-earned $DRIA points. We have a `--backup` option for this purpose!

```sh
dkn-compute-launcher uninstall

# will save the removed .env file to the given path
dkn-compute-launcher uninstall --backup ./my-backup.txt
```

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/license/Apache-2.0).
