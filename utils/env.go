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

// GetWorkingDir returns the current working directory. If the application is running using `go run`,
// it returns the directory where the command is executed. If running as a compiled binary, it returns
// the directory where the executable is located.
//
// Returns:
//   - string: The path to the working directory. Exits the program with a delay if an error occurs.
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

// CheckRequiredEnvVars checks if the required environment variables are set in the provided map pointer.
// If `DKN_WALLET_SECRET_KEY` is not set, it prompts the user to input it interactively.
// If `DKN_ADMIN_PUBLIC_KEY` is not set, it sets it to the provided default value.
//
// Parameters:
//   - envvars: A pointer to a map of environment variables to check and update.
//   - default_admin_pkey: The default admin public key to use if `DKN_ADMIN_PUBLIC_KEY` is not set.
func CheckRequiredEnvVars(envvars *map[string]string, default_admin_pkey string) {
	if (*envvars)["DKN_WALLET_SECRET_KEY"] == "" {
		fmt.Println("DKN_WALLET_SECRET_KEY env-var is not set, getting it interactively")
		skey, err := GetDknSecretKey()
		if err != nil {
			fmt.Printf("Error during user input: %s\n", err)
			ExitWithDelay(1)
		}
		(*envvars)["DKN_WALLET_SECRET_KEY"] = skey
	}

	if (*envvars)["DKN_ADMIN_PUBLIC_KEY"] == "" {
		(*envvars)["DKN_ADMIN_PUBLIC_KEY"] = default_admin_pkey
	}
}

// FileExists checks if a file exists at the given path, constructed from the provided path parts.
//
// Parameters:
//   - parts: A variadic parameter that takes parts of the file path to check.
//
// Returns:
//   - bool: Returns true if the file exists and is not a directory, otherwise false.
func FileExists(parts ...string) bool {
	joinedPath := filepath.Join(parts...)
	info, err := os.Stat(joinedPath)
	if os.IsNotExist(err) {
		return false
	}
	return !info.IsDir()
}

// DownloadFile downloads a file from the specified URL and saves it to the specified path.
//
// Parameters:
//   - url: The URL from which to download the file.
//   - path: The local file path where the downloaded file will be saved.
//
// Returns:
//   - error: Returns an error if the download or file writing fails, otherwise nil.
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

// FetchEnvFileFromDknRepo downloads the .env example file from the DKN GitHub repository
// and loads its contents into a map of environment variables.
//
// Parameters:
//   - working_dir: The directory where the .env file will be saved.
//
// Returns:
//   - map[string]string: A map containing the loaded environment variables.
//   - error: Returns an error if the download or loading of the env file fails, otherwise nil.
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

// FetchComposeFileFromDknRepo downloads the Docker Compose file from the DKN GitHub repository
// and saves it to the specified working directory.
//
// Parameters:
//   - working_dir: The directory where the compose.yml file will be saved.
//
// Returns:
//   - error: Returns an error if the download fails, otherwise nil.
func FetchComposeFileFromDknRepo(working_dir string) error {
	// fetch from github
	url := "https://raw.githubusercontent.com/firstbatchxyz/dkn-compute-node/master/compose.yml"
	path := filepath.Join(working_dir, "compose.yml")
	if err := DownloadFile(url, path); err != nil {
		return err
	}
	return nil
}
