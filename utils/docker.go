package utils

import "fmt"

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

func IsDockerUp() bool {
	_, err := RunCommand("", false, true, nil, "docker", "info")
	return err == nil
}
