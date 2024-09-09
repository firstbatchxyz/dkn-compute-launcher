package utils

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"path/filepath"
	"runtime"
)

// GetComputeLatestTag fetches the latest tag from the DKN Compute Node repository on GitHub.
// This tag represents the latest version of the compute node.
//
// Returns:
//   - string: The latest tag (version) as a string.
//   - error: An error if the request fails or the response cannot be parsed.
func GetComputeLatestTag() (string, error) {
	url := "https://api.github.com/repos/firstbatchxyz/dkn-compute-node/tags"

	// get and parse the tags
	resp, err := http.Get(url)
	if err != nil {
		return "", fmt.Errorf("failed to make request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("request failed with status code: %d", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("failed to read response: %w", err)
	}

	var tags []map[string]interface{}
	if err := json.Unmarshal(body, &tags); err != nil {
		return "", fmt.Errorf("failed to parse JSON: %w", err)
	}
	if len(tags) == 0 {
		return "", fmt.Errorf("no tags found")
	}
	latestTag, ok := tags[0]["name"].(string)
	if !ok {
		return "", fmt.Errorf("failed to extract tag name")
	}

	return latestTag, nil
}

// DownloadLatestComputeBinary downloads the latest compute binary for the current operating system and architecture
// from the DKN Compute Node GitHub repository.
//
// Parameters:
//   - workingDir: The directory where the binary will be saved.
//   - file: The name of the file to save the binary as.
//
// Returns:
//   - error: An error if the download or file preparation fails.
func DownloadLatestComputeBinary(workingDir, file string) error {
	os, arch := GetOSAndArch()
	extension := ""
	if os == "windows" {
		extension = ".exe"
	}
	asset_name := fmt.Sprintf("dkn-compute-binary-%s-%s%s", os, arch, extension)
	url := fmt.Sprintf("https://github.com/firstbatchxyz/dkn-compute-node/releases/latest/download/%s", asset_name)
	destPath := filepath.Join(workingDir, file)
	if err := DownloadFile(url, destPath); err != nil {
		return err
	}

	// give the executable privledges etc.
	if err := PrepareComputeBinary(workingDir, file); err != nil {
		return err
	}

	return nil
}

// PrepareComputeBinary grants execute privileges to the DKN Compute binary on Linux or macOS.
//
// Parameters:
//   - working_dir: The directory where the binary is located.
//   - file: The name of the file (binary) to modify.
//
// Returns:
//   - error: An error if the file's permissions cannot be changed or if there is an issue with execution.
func PrepareComputeBinary(working_dir, file string) error {
	if runtime.GOOS == "linux" || runtime.GOOS == "darwin" {
		// chmod compute node binary
		_, err := RunCommand(working_dir, "stdout", true, 0, nil, "chmod", "+x", file)
		if err != nil {
			return fmt.Errorf("coudln't give exec privileges to the dkn-compute binary: %s", err)
		}
	}
	return nil
}

// ComputeBinaryFileName returns the appropriate name for the DKN Compute binary based on the operating system.
//
// Returns:
//   - string: The name of the DKN Compute binary, with ".exe" appended for Windows.
func ComputeBinaryFileName() string {
	dkn_compute_exe := "dkn_compute"
	if runtime.GOOS == "windows" {
		dkn_compute_exe += ".exe"
	}
	return dkn_compute_exe
}
