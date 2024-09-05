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
	"time"

	"github.com/joho/godotenv"
)

var (
	OLLAMA_MODELS = []string{
		"adrienbrault/nous-hermes2theta-llama3-8b:q8_0",
		"phi3:14b-medium-4k-instruct-q4_1",
		"phi3:14b-medium-128k-instruct-q4_1",
		"phi3:3.8b",
		"phi3.5:3.8b",
		"phi3.5:3.8b-mini-instruct-fp16",
		"llama3.1:latest",
		"llama3.1:8b-instruct-q8_0",
	}
	OPENAI_MODELS = []string{"gpt-3.5-turbo", "gpt-4-turbo", "gpt-4o", "gpt-4o-mini"}

	// Default admin public key, it will be used unless --dkn-admin-public-key is given
	DKN_ADMIN_PUBLIC_KEY = "0208ef5e65a9c656a6f92fb2c770d5d5e2ecffe02a6aade19207f75110be6ae658"
)

// main is the entry point of the DKN Compute Node Launcher.
// It sets up the environment, checks required conditions, and launches the compute node using Docker.
//
// The function processes command-line flags, including:
//
//	-h, --help: Display help message.
//	-m, --model: Specify models to be used in the compute node (multiple models can be specified).
//	-b, --background: Run the compute node in background mode.
//	--dev: Set logging level to debug.
//	--trace: Set logging level to trace.
//	--docker-ollama: Use Ollama Docker image.
//	--dkn-admin-public-key: Set the DKN Admin Node Public Key.
//	--pick-models: Pick models interactively, suppressing the -m flags.
//
// The function performs the following tasks:
//  1. Initializes the environment, checking for required files and fetching them if necessary.
//  2. Loads environment variables from .env files or fetches them from the dkn-compute-node repository if not present.
//  3. Configures and verifies models, API keys, and logging settings.
//  4. Starts the compute node using Docker Compose, either in foreground or background mode.
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
	dockerOllama := flag.Bool("docker-ollama", false, "Indicates the Ollama docker image is being used (default: false)")
	dkn_admin_pkey_flag := flag.String("dkn-admin-public-key", DKN_ADMIN_PUBLIC_KEY, "DKN Admin Node Public Key, usually dont need this since it's given by default")
	pick_model := flag.Bool("pick-models", false, "Pick the models using cli, supprases the -m flags (default: false)")
	flag.Parse()

	// Display help and exit if -h or --help is provided
	if *help {
		flag.Usage()
		os.Exit(0)
	}

	fmt.Printf("Setting up the environment...\n")

	// get the current working directory
	working_dir := utils.GetWorkingDir()

	// Check Docker Compose
	fmt.Println("Checking Docker...")
	composeCommand, composeUpArgs, composeDownArgs := utils.CheckDockerComposeCommand()
	// check docker is up by waiting 10 seconds
	if !utils.IsDockerUp(10 * time.Second) {
		utils.ExitWithDelay(1)
	}

	// check compose.yml
	if !utils.FileExists(working_dir, "compose.yml") {
		fmt.Println("Couldn't find compose.yml, fetching it from github.com/firstbatchxyz/dkn-compute-node")
		if err := utils.FetchComposeFileFromDknRepo(working_dir); err != nil {
			fmt.Printf("ERROR during fetching the compose.yml file from the repo %s\n", err)
			utils.ExitWithDelay(1)
		}

	}

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
		ollamaHost, ollamaPort, dockerNetworkMode, composeProfile := utils.HandleOllamaEnv(envvars["OLLAMA_HOST"], envvars["OLLAMA_PORT"], *dockerOllama)
		envvars["OLLAMA_HOST"] = ollamaHost
		envvars["OLLAMA_PORT"] = ollamaPort
		envvars["COMPOSE_PROFILES"] = composeProfile
		envvars["DKN_DOCKER_NETWORK_MODE"] = dockerNetworkMode

		fmt.Printf("Ollama host: %s (network mode: %s)\n\n", envvars["OLLAMA_HOST"], envvars["DKN_DOCKER_NETWORK_MODE"])
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

	// Update the image
	fmt.Println("Pulling the latest compute node image...")
	_, err = utils.RunCommand(working_dir, true, true, 0, []string{"DOCKER_CLI_HINTS=false"}, "docker", "pull", "firstbatch/dkn-compute-node:latest")
	if err != nil {
		fmt.Println("Error during pulling the latest compute node image")
		utils.ExitWithDelay(1)
	}

	// dump the final env
	if err := godotenv.Write(envvars, filepath.Join(working_dir, ".env")); err != nil {
		fmt.Printf("Failed to dump the .env file, continuing to running the node though. error message: %s\n", err)
	}

	// log final status
	fmt.Printf("\nLog level: %s\n", envvars["RUST_LOG"])
	fmt.Printf("Models: %s\n", envvars["DKN_MODELS"])
	fmt.Printf("Operating System: %s\n", runtime.GOOS)
	fmt.Printf("COMPOSE_PROFILES: [%s]\n", envvars["COMPOSE_PROFILES"])
	if *background {
		fmt.Printf("\nStarting in BACKGROUND mode...\n")
	} else {
		fmt.Printf("\nStarting in FOREGROUND mode...\n")
	}

	// Run docker-compose up
	_, err = utils.RunCommand(working_dir, true, true, 0, utils.MapToList(envvars), composeCommand, composeUpArgs...)
	if err != nil {
		fmt.Printf("ERROR: docker-compose, %s", err)
		utils.ExitWithDelay(1)
	}

	fmt.Println("All good! Compute node is up and running.")
	fmt.Println("You can check logs with: docker compose logs -f compute.")

	// Foreground mode
	if !(*background) {
		fmt.Println("\nUse Control-C to exit")
		sig := make(chan os.Signal, 1)
		signal.Notify(sig, os.Interrupt)
		<-sig

		fmt.Println("\nShutting down...")
		_, err = utils.RunCommand(working_dir, true, true, 0, utils.MapToList(envvars), composeCommand, composeDownArgs...)
		if err != nil {
			fmt.Printf("Error during docker compose down; %s\n", err)
		}

		fmt.Println("\nbye")
		os.Exit(0)
	}
}
