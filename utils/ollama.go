package utils

import (
	"fmt"
	"net/http"
	"runtime"
	"strconv"
	"strings"
	"time"
)

var (
	DEFAULT_OLLAMA_PORT = 11434
	DOCKER_HOST         = "http://host.docker.internal"
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
	pid, err := RunCommand("", false, false, []string{ollama_env}, "ollama", "serve")
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

// HandleOllamaEnv sets up the environment for running the Ollama service, either locally or using Docker.
//
// Parameters:
//   - ollamaHost: The initial host address for Ollama (can be overridden based on checks).
//   - ollamaPort: The initial port number for Ollama (can be overridden based on checks).
//   - dockerOllama: A boolean indicating whether to use Docker to run Ollama.
//
// Returns:
//   - string: The final host address for Ollama.
//   - string: The final port number for Ollama.
//   - string: The Docker network mode to use ("bridge" or "host").
//   - string: The Docker compose profile to use based on the GPU type (e.g., "ollama-cuda", "ollama-rocm", or "ollama-cpu").
func HandleOllamaEnv(ollamaHost, ollamaPort string, dockerOllama bool) (string, string, string, string) {
	// local ollama
	if !dockerOllama {
		if IsCommandAvailable("ollama") {
			// host machine has ollama installed
			// we first going to check whether its serving or not
			// if not script runs ollama serve command manually and stores its pid

			// prepare local ollama url
			if ollamaHost == "" || ollamaHost == DOCKER_HOST {
				// we have to check Ollama at host, but if the given host is
				// host.docker.internal we still have to check the localhost
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
					fmt.Println("You can use the --docker-ollama flag to use the Docker Ollama image instead.")
					ExitWithDelay(1)
				} else {
					fmt.Printf("Local Ollama server is up at %s:%s and running with PID %d\n", ollamaHost, ollamaPort, ollama_pid)
				}
			}

			// to use the local Ollama, we need to configure the network depending on the Host
			// Windows and Mac should work with host.docker.internal alright,
			// but Linux requires `host` network mode with `localhost` as the Host URL
			if runtime.GOOS == "darwin" {
				ollamaHost = DOCKER_HOST
			} else if runtime.GOOS == "windows" {
				ollamaHost = DOCKER_HOST
			} else if runtime.GOOS == "linux" {
				ollamaHost = LOCAL_HOST
			}
		} else {
			// although --docker-ollama was not passed, we checked and couldnt find Ollama
			// so we will use Docker anyways
			fmt.Println("Ollama is not installed on this machine, will use Docker Ollama service.")
			dockerOllama = true
		}
	}

	composeProfile := ""
	if dockerOllama {
		// using docker-ollama, check profiles
		if IsCommandAvailable("nvidia-smi") {
			composeProfile = "ollama-cuda"
			fmt.Println("GPU type detected: CUDA")
		} else if IsCommandAvailable("rocminfo") {
			fmt.Println("GPU type detected: ROCM")
			composeProfile = "ollama-rocm"
		} else {
			fmt.Println("No GPU found, using ollama-cpu")
			composeProfile = "ollama-cpu"
		}

		// since docker-ollama is using, set docker.internal for the Ollama host
		ollamaHost = DOCKER_HOST
		ollamaPort = strconv.Itoa(DEFAULT_OLLAMA_PORT)
	}

	// depending on the OS, use host or bridge network modes
	// https://docs.docker.com/engine/network/#drivers
	dockerNetworkMode := ""
	if runtime.GOOS == "darwin" {
		dockerNetworkMode = "bridge"
	} else if runtime.GOOS == "windows" {
		dockerNetworkMode = "bridge"
	} else if runtime.GOOS == "linux" {
		dockerNetworkMode = "host"
	}

	return ollamaHost, ollamaPort, dockerNetworkMode, composeProfile
}
