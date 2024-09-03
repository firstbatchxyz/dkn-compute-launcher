package utils

import "fmt"

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
	if _, err := RunCommand("", false, true, nil, "docker", "compose", "version"); err == nil {
		return "docker", []string{"compose", "up", "-d"}, []string{"compose", "down"}
	}

	// check docker-compose
	if _, err := RunCommand("", false, true, nil, "docker-compose", "version"); err == nil {
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
func IsDockerUp() bool {
	_, err := RunCommand("", false, true, nil, "docker", "info")
	return err == nil
}
