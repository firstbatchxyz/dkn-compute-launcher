package utils

import (
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"
)

var (
	DEFAULT_OLLAMA_PORT = 11434
	LOCAL_HOST          = "http://localhost"
	OLLAMA_MAX_RETRIES  = 5
)

// IsOllamaRequired checks if any of the picked models are in the list of Ollama models,
// indicating whether Ollama is required.
//
// Parameters:
//   - picked_models: A comma-separated string of model names selected by the user.
//   - ollama_models: A pointer to a slice of strings containing available Ollama model names.
//
// Returns:
//   - bool: Returns true if any of the picked models require Ollama, otherwise false.
func IsOllamaRequired(picked_models string, ollama_models *[]string) bool {
	required := false
	for _, model := range strings.Split(picked_models, ",") {
		for _, ollama_model := range *ollama_models {
			if model == ollama_model {
				required = true
				break
			}
		}
	}
	return required
}

// IsOllamaServing checks if the Ollama service is running by making an HTTP GET request to the specified host and port.
//
// Parameters:
//   - host: The host address of the Ollama service.
//   - port: The port number on which the Ollama service is expected to be running.
//
// Returns:
//   - bool: Returns true if the service is running (i.e., returns HTTP 200 OK), otherwise false.
func IsOllamaServing(host, port string) bool {
	client := http.Client{
		Timeout: 2 * time.Second,
	}

	resp, err := client.Get(fmt.Sprintf("%s:%s", host, port))
	if err != nil {
		return false
	}
	defer resp.Body.Close()

	return resp.StatusCode == http.StatusOK
}

// RunOllamaServe starts the Ollama service on the specified host and port, and checks if it starts successfully.
//
// Parameters:
//   - host: The host address where Ollama should run.
//   - port: The port number on which Ollama should listen.
//
// Returns:
//   - int: The process ID (PID) of the Ollama service.
//   - error: Returns an error if the Ollama service fails to start, otherwise nil.
func RunOllamaServe(host, port string) (int, error) {
	ollama_env := fmt.Sprintf("OLLAMA_HOST=%s:%s", host, port)
	pid, err := RunCommand("", "none", false, 0, []string{ollama_env}, "ollama", "serve")
	if err != nil {
		return 0, fmt.Errorf("failed during running ollama serve: %w", err)
	}

	for retryCount := 0; retryCount < OLLAMA_MAX_RETRIES; retryCount++ {
		if IsOllamaServing(host, port) {
			return pid, nil
		}
		fmt.Printf("Waiting for the local Ollama server to start... (Attempt %d/%d)\n", retryCount+1, OLLAMA_MAX_RETRIES)
		time.Sleep(2 * time.Second)
	}

	return pid, fmt.Errorf("ollama failed to start after %d retries", OLLAMA_MAX_RETRIES)
}

// HandleOllamaEnv sets up the environment for running the Ollama service
//
// Parameters:
//   - ollamaHost: The initial host address for Ollama (can be overridden based on checks).
//   - ollamaPort: The initial port number for Ollama (can be overridden based on checks).
//
// Returns:
//   - string: The final host address for Ollama.
//   - string: The final port number for Ollama.
func HandleOllamaEnv(ollamaHost, ollamaPort string) (string, string) {
	// local ollama
	if IsCommandAvailable("ollama") {
		// host machine has ollama installed
		// we first going to check whether its serving or not
		// if not script runs ollama serve command manually and stores its pid

		// prepare local ollama url
		if ollamaHost == "" {
			ollamaHost = LOCAL_HOST
		}
		if ollamaPort == "" {
			ollamaPort = strconv.Itoa(DEFAULT_OLLAMA_PORT)
		}

		// check is it already serving
		if IsOllamaServing(ollamaHost, ollamaPort) {
			fmt.Printf("Local Ollama is already up at %s:%s and running, using it\n", ollamaHost, ollamaPort)
		} else {
			// ollama is not live, so we launch it ourselves
			fmt.Println("Local Ollama is not live, running ollama serve")
			ollama_pid, err := RunOllamaServe(ollamaHost, ollamaPort)
			if err != nil {
				// ollama failed to start, exit
				fmt.Println(err)
				ExitWithDelay(1)
			} else {
				fmt.Printf("Local Ollama server is up at %s:%s and running with PID %d\n", ollamaHost, ollamaPort, ollama_pid)
			}
		}
	} else {
		fmt.Println("Ollama is not installed on this machine.")
		fmt.Println("Please download it first from https://ollama.com/download.")
		ExitWithDelay(1)
		return "", ""
	}

	return ollamaHost, ollamaPort
}
