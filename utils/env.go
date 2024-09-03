package utils

import (
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"runtime"

	"github.com/joho/godotenv"
)

func GetWorkingDir() string {
	// Check if running in a binary or with `go run`
	if runtime.Compiler == "gc" {
		// Using `go run`, find the current working directory
		workingDir, err := os.Getwd()
		if err != nil {
			fmt.Printf("Error getting the current working directory: %s\n", err)
			ExitWithDelay(1)
		}
		return workingDir
	} else {
		// Running as a compiled binary
		ex, err := os.Executable()
		if err != nil {
			fmt.Printf("Error getting the executable path: %s\n", err)
			ExitWithDelay(1)
		}
		workingDir := filepath.Dir(ex)
		return workingDir
	}
}

// todo take envvar as pointer for safety
func CheckRequiredEnvVars(envvars map[string]string, default_admin_pkey string) {
	if envvars["DKN_WALLET_SECRET_KEY"] == "" {
		fmt.Println("DKN_WALLET_SECRET_KEY env-var is not set, getting it interactively")
		skey, err := GetDknSecretKey()
		if err != nil {
			fmt.Printf("Error during user input: %s\n", err)
			ExitWithDelay(1)
		}
		envvars["DKN_WALLET_SECRET_KEY"] = skey
	}

	if envvars["DKN_ADMIN_PUBLIC_KEY"] == "" {
		envvars["DKN_ADMIN_PUBLIC_KEY"] = default_admin_pkey
	}
}

func FileExists(parts ...string) bool {
	joinedPath := filepath.Join(parts...)
	info, err := os.Stat(joinedPath)
	if os.IsNotExist(err) {
		return false
	}
	return !info.IsDir()
}

func DownloadFile(url, path string) error {
	resp, err := http.Get(url)
	if err != nil {
		return fmt.Errorf("failed to download file: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("bad status: %s", resp.Status)
	}

	// write it as .env
	out, err := os.Create(path)
	if err != nil {
		return fmt.Errorf("failed to create file: %v", err)
	}
	defer out.Close()

	// write the body to file
	_, err = io.Copy(out, resp.Body)
	if err != nil {
		return fmt.Errorf("failed to write to file: %v", err)
	}
	return nil
}

func FetchEnvFileFromDknRepo(working_dir string) (map[string]string, error) {
	// fetch from github
	url := "https://raw.githubusercontent.com/firstbatchxyz/dkn-compute-node/master/.env.example"
	path := filepath.Join(working_dir, ".env")
	if err := DownloadFile(url, path); err != nil {
		return nil, err
	}

	// load the created file as envs
	envvars, err := godotenv.Read(path)
	if err != nil {
		return nil, fmt.Errorf("failed to load env file: %v", err)
	}

	return envvars, nil
}

func FetchComposeFileFromDknRepo(working_dir string) error {
	// fetch from github
	url := "https://raw.githubusercontent.com/firstbatchxyz/dkn-compute-node/master/compose.yml"
	path := filepath.Join(working_dir, "compose.yml")
	if err := DownloadFile(url, path); err != nil {
		return err
	}
	return nil
}
