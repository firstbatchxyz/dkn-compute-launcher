package utils

import (
	"context"
	"fmt"
	"time"
)

// CheckDockerComposeCommand checks whether the system has Docker Compose installed and returns
// the appropriate command and arguments for starting and stopping Docker containers.
//
// Returns:
//   - string: The command to use for Docker Compose ("docker" or "docker-compose").
//   - []string: A slice of arguments for bringing up the Docker containers.
//   - []string: A slice of arguments for bringing down the Docker containers.
//
// Exits the program with a delay if neither Docker Compose nor docker-compose is installed.
func CheckDockerComposeCommand() (string, []string, []string) {
	// check docker compose
	if _, err := RunCommand("", false, true, 0, nil, "docker", "compose", "version"); err == nil {
		return "docker", []string{"compose", "up", "-d"}, []string{"compose", "down"}
	}

	// check docker-compose
	if _, err := RunCommand("", false, true, 0, nil, "docker-compose", "version"); err == nil {
		return "docker-compose", []string{"up", "-d"}, []string{"down"}
	}

	// both not found, exit
	fmt.Println("docker compose is not installed on this machine. It's required to run the node.")
	fmt.Println("Check https://docs.docker.com/compose/install/ for installation.")
	ExitWithDelay(1)
	return "", nil, nil
}

// IsDockerUp checks if Docker is running on the system by executing the "docker info" command.
//
// Returns:
//   - bool: Returns true if Docker is running (i.e., "docker info" executes successfully), otherwise false.
func IsDockerUp(timeout time.Duration) bool {
	_, err := RunCommand("", false, true, timeout, nil, "docker", "info")
	if err != nil {
		if err == context.DeadlineExceeded {
			fmt.Println("Error: Docker did not respond within the expected time.")
			fmt.Println("Suggested actions:")
			fmt.Println("  1. Quit and restart Docker Desktop.")
			fmt.Println("  3. Try restarting Docker with 'sudo systemctl restart docker' (Linux/macOS) or 'Restart-Service docker' (Windows).")
			fmt.Println("  4. Verify that your Docker daemon has sufficient resources.")
			fmt.Println("")
		} else {
			fmt.Println("Error: Docker is not up, please start Docker-Desktop first.")
		}
		return false
	}

	return true
}
