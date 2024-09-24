package main

import (
	"dkn-compute-launcher/utils"
	"flag"
	"fmt"
	"os"
	"os/signal"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/joho/godotenv"
)

var (
	// https://github.com/andthattoo/ollama-workflows/edit/main/src/program/models.rs#L13
	OLLAMA_MODELS = []string{
		"finalend/hermes-3-llama-3.1:8b-q8_0",

		"phi3:14b-medium-4k-instruct-q4_1",
		"phi3:14b-medium-128k-instruct-q4_1",

		"phi3.5:3.8b",
		"phi3.5:3.8b-mini-instruct-fp16",

		"gemma2:9b-instruct-q8_0",
		"gemma2:9b-instruct-fp16",

		"qwen2.5:7b-instruct-q5_0",
		"qwen2.5:7b-instruct-fp16",
		"qwen2.5:32b-instruct-fp16",

		"llama3.1:latest",
		"llama3.1:8b-instruct-q8_0",
		"llama3.1:8b-instruct-fp16",
		"llama3.1:70b-instruct-q4_0",
		"llama3.1:70b-instruct-q8_0",
	}
	OPENAI_MODELS = []string{
		"gpt-4-turbo",
		"gpt-4o",
		"gpt-4o-mini",

		"o1-mini",
		"o1-preview",
	}

	// Default admin public key, it will be used unless --dkn-admin-public-key is given
	DKN_ADMIN_PUBLIC_KEY = "0208ef5e65a9c656a6f92fb2c770d5d5e2ecffe02a6aade19207f75110be6ae658"
)

// main is the entry point of the DKN Compute Node Launcher.
// It sets up the environment, checks required conditions, and launches the compute node using dkn-compute executable.
//
// The function processes command-line flags, including:
//
//	-h, --help: Display help message.
//	-m, --model: Specify models to be used in the compute node (multiple models can be specified).
//	-b, --background: Run the compute node in background mode.
//	--dev: Set logging level to debug.
//	--trace: Set logging level to trace.
//	--dkn-admin-public-key: Set the DKN Admin Node Public Key.
//	--pick-models: Pick models interactively, suppressing the -m flags.
//
// The function performs the following tasks:
//  1. Initializes the environment, checking for required files and fetching them if necessary.
//  2. Loads environment variables from .env files or fetches them from the dkn-compute-node repository if not present.
//  3. Configures and verifies models, API keys, and logging settings.
//  4. Starts the compute node, either in foreground or background mode.
//  5. Handles graceful shutdown in foreground mode by capturing interrupt signals.
func main() {
	fmt.Println("************ DKN - Compute Node ************")

	help := flag.Bool("h", false, "Displays this help message")
	flag.BoolVar(help, "help", false, "Displays this help message")
	var models utils.ModelList
	flag.Var(&models, "m", "Indicates the model to be used within the compute node. Can be used multiple times for multiple models.")
	flag.Var(&models, "model", "Indicates the model to be used within the compute node. Can be used multiple times for multiple models.")
	background := flag.Bool("b", false, "Enables background mode for running the node (default: FOREGROUND)")
	flag.BoolVar(background, "background", false, "Enables background mode for running the node (default: FOREGROUND)")
	dev := flag.Bool("dev", false, "Sets the logging level to debug (default: false)")
	trace := flag.Bool("trace", false, "Sets the logging level to trace (default: false)")
	dkn_admin_pkey_flag := flag.String("dkn-admin-public-key", DKN_ADMIN_PUBLIC_KEY, "DKN Admin Node Public Key, usually dont need this since it's given by default")
	pick_model := flag.Bool("pick-models", false, "Pick the models using cli, supprases the -m flags (default: false)")
	use_compute_dev_version := flag.Bool("compute-dev-version", false, "For using the latest dev version of dkn-compute node (optional, only for development purposes)")
	flag.Parse()

	// Display help and exit if -h or --help is provided
	if *help {
		flag.Usage()
		os.Exit(0)
	}

	fmt.Printf("Setting up the environment...\n")

	// get the current working directory
	working_dir := utils.GetWorkingDir()

	// load the env vars
	envvars, err := utils.LoadEnv(working_dir)
	if err != nil {
		fmt.Println(err)
		utils.ExitWithDelay(1)
	}

	// override DKN_ADMIN_PUBLIC_KEY if flag is a different value
	DKN_ADMIN_PUBLIC_KEY = *dkn_admin_pkey_flag
	utils.CheckRequiredEnvVars(&envvars, DKN_ADMIN_PUBLIC_KEY)

	// if -m flag is given, set them as DKN_MODELS
	if len(models) != 0 {
		envvars["DKN_MODELS"] = strings.Join(models, ",")
	}

	// if DKN_MODELS are still empty, pick model interactively
	if envvars["DKN_MODELS"] == "" || *pick_model {
		pickedModels := utils.PickModels(OPENAI_MODELS, OLLAMA_MODELS)
		if pickedModels == "" {
			fmt.Println("No valid model picked")
			utils.ExitWithDelay(1)
		}
		envvars["DKN_MODELS"] = pickedModels
	}

	// check openai api key
	if utils.IsOpenAIRequired(envvars["DKN_MODELS"], &OPENAI_MODELS) && envvars["OPENAI_API_KEY"] == "" {
		apikey := utils.GetUserInput("Enter your OpenAI API Key", true)
		if apikey == "" {
			fmt.Println("Invalid input, please place your OPENAI_API_KEY to .env file")
			utils.ExitWithDelay(1)
		}
		envvars["OPENAI_API_KEY"] = apikey
	}

	// check ollama environment
	if utils.IsOllamaRequired(envvars["DKN_MODELS"], &OLLAMA_MODELS) {
		ollamaHost, ollamaPort := utils.HandleOllamaEnv(envvars["OLLAMA_HOST"], envvars["OLLAMA_PORT"])
		envvars["OLLAMA_HOST"] = ollamaHost
		envvars["OLLAMA_PORT"] = ollamaPort

		fmt.Printf("Ollama host: %s\n\n", envvars["OLLAMA_HOST"])
	} else {
		fmt.Printf("No Ollama model provided. Skipping the Ollama execution\n\n")
	}

	// get jina and serper api keys
	if envvars["JINA_API_KEY"] == "" {
		envvars["JINA_API_KEY"] = utils.GetUserInput("Enter your Jina API key (optional, just press enter for skipping it)", true)
	}
	if envvars["SERPER_API_KEY"] == "" {
		envvars["SERPER_API_KEY"] = utils.GetUserInput("Enter your Serper API key (optional, just press enter for skipping it)", true)
	}

	// log level
	if *dev {
		envvars["RUST_LOG"] = "none,dkn_compute=debug,ollama_workflows=info"
	} else if *trace {
		envvars["RUST_LOG"] = "none,dkn_compute=trace"
	} else {
		// default level info
		envvars["RUST_LOG"] = "none,dkn_compute=info"
	}

	// get latest dkn_compute binary version
	latestVersion, err := utils.GetComputeLatestTag(*use_compute_dev_version)
	if err != nil {
		fmt.Println("Couldn't get the latest dkn-compute version")
		utils.ExitWithDelay(1)
	}
	dkn_compute_binary := utils.ComputeBinaryFileName()

	// check dkn-compute binary has already installed
	if utils.FileExists(utils.ComputeBinaryFileName()) {
		// compare current and latest versions
		if latestVersion != envvars["DKN_COMPUTE_VERSION"] {
			fmt.Printf("New dkn-compute version detected (%s), downloading it...\n", latestVersion)
			if err := utils.DownloadLatestComputeBinary(latestVersion, working_dir, dkn_compute_binary); err != nil {
				fmt.Printf("Error during downloading the latest dkn-compute binary %s\n", err)
				utils.ExitWithDelay(1)
			}
			envvars["DKN_COMPUTE_VERSION"] = latestVersion
		} else {
			fmt.Printf("Current version is up to date (%s)\n", envvars["DKN_COMPUTE_VERSION"])
		}
	} else {
		// couldn't find the dkn-compute binary, download it
		fmt.Printf("Downloading the latest dkn-compute binary (%s)\n", latestVersion)
		if err := utils.DownloadLatestComputeBinary(latestVersion, working_dir, dkn_compute_binary); err != nil {
			fmt.Printf("Error during downloading the latest dkn-compute binary %s\n", err)
			utils.ExitWithDelay(1)
		}
		envvars["DKN_COMPUTE_VERSION"] = latestVersion
	}

	// dump the final env
	if err := godotenv.Write(envvars, filepath.Join(working_dir, ".env")); err != nil {
		fmt.Printf("Failed to dump the .env file, continuing to running the node though. error message: %s\n", err)
	}

	// log final status
	fmt.Printf("\nLog level: %s\n", envvars["RUST_LOG"])
	fmt.Printf("Models: %s\n", envvars["DKN_MODELS"])
	fmt.Printf("Operating System: %s\n", runtime.GOOS)

	// get the binary execution command; "./dkn-compute" (linux/macos), ".\\dkn-compute.exe" (windows)
	exec_command := ""
	if runtime.GOOS == "windows" {
		exec_command = fmt.Sprintf(".\\%s", dkn_compute_binary)
	} else {
		exec_command = fmt.Sprintf("./%s", dkn_compute_binary)
	}

	// Run dkn-compute
	if *background {
		fmt.Printf("\nStarting in BACKGROUND mode...\n\n")
		dkn_pid, err := utils.RunCommand(working_dir, "file", false, 0, utils.MapToList(envvars), exec_command)
		if err != nil {
			fmt.Printf("ERROR during running exe, %s", err)
			utils.ExitWithDelay(1)
		}

		fmt.Printf("All good! Compute node is up and running with PID: %d\n", dkn_pid)
		fmt.Printf("You can check the logs from %s\n", filepath.Join(working_dir, "logs.txt"))

		fmt.Printf("For stopping the background node you can run the following command: ")
		if runtime.GOOS == "windows" {
			fmt.Printf("taskkill /PID %d /F\n", dkn_pid)
		} else {
			fmt.Printf("kill %d\n", dkn_pid)
		}
	} else {
		fmt.Printf("\nStarting in FOREGROUND mode...\n")

		_, err := utils.RunCommand(working_dir, "stdout", true, 0, utils.MapToList(envvars), exec_command)
		if err != nil {
			fmt.Printf("ERROR during running exe, %s", err)
			utils.ExitWithDelay(1)
		}
		fmt.Println("All good! Compute node is up and running.")
		fmt.Println("\nUse Control-C to exit")

		sig := make(chan os.Signal, 1)
		signal.Notify(sig, os.Interrupt)
		<-sig

		fmt.Println("\nShutting down...")

		fmt.Println("\nbye")
		os.Exit(0)
	}

}
