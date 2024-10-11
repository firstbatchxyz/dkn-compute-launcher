package utils

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"path/filepath"
	"runtime"
	"strings"
)

// GetComputeLatestTag fetches a specific tag from the DKN Compute Node repository on GitHub based on the provided parameters.
// It can return the latest stable release, the latest development version, or the previous stable release.
//
// Parameters:
//   - latest: If true, it returns the latest stable release from the repository.
//   - dev: If true, it returns the latest tag with the '-dev' suffix.
//   - previous_latest: If true, it returns the previous stable release before the latest.
//
// Returns:
//   - string: The requested tag (version) as a string, based on the provided parameters.
//   - error: An error if the request fails, the response cannot be parsed, or no valid tags are found.
//
// Note:
//   - If `latest` is true, the function fetches the latest release from the GitHub API.
//   - If `dev` is true, it searches for the latest tag with the '-dev' suffix from the sorted tags.
//   - If `previous_latest` is true, it returns the previous stable release tag (ignoring '-dev' tags).
//   - The function prioritizes parameters in the following order: latest, dev, previous_latest.
func GetComputeLatestTag(latest bool, dev bool, previous_latest bool) (string, error) {
	if latest {
		url := "https://api.github.com/repos/firstbatchxyz/dkn-compute-node/releases/latest"

		resp, err := http.Get(url)
		if err != nil {
			return "", fmt.Errorf("failed to make request: %v", err)
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			return "", fmt.Errorf("failed to get latest release, status code: %d", resp.StatusCode)
		}

		body, err := io.ReadAll(resp.Body)
		if err != nil {
			return "", fmt.Errorf("failed to read response: %w", err)
		}

		// Create a map to store the response
		var result map[string]interface{}
		if err := json.Unmarshal(body, &result); err != nil {
			return "", fmt.Errorf("failed to parse JSON: %v", err)
		}

		// Extract the tag_name from the map
		tagName, ok := result["tag_name"].(string)
		if !ok {
			return "", fmt.Errorf("tag_name not found or not a string")
		}

		return tagName, nil

	} else if dev {
		tags, err := GetSortedTags()
		if err != nil {
			return "", err
		}
		// Iterate through the tags and return the first one based on the 'dev' parameter
		for _, tag := range tags {
			tagName, ok := tag["name"].(string)
			if !ok {
				return "", fmt.Errorf("failed to extract tag name")
			}

			// Return the first tag with '-dev' suffix if dev is true
			if strings.HasSuffix(tagName, "-dev") {
				return tagName, nil
			}
		}
	} else if previous_latest {
		tags, err := GetSortedTags()
		if err != nil {
			return "", err
		}
		latest_encountered := false
		// Iterate through the tags and return the previous latest (in the order of semantic versioning)
		for _, tag := range tags {
			tagName, ok := tag["name"].(string)
			if !ok {
				return "", fmt.Errorf("failed to extract tag name")
			}

			// skip the latest tag and -dev suffix tags
			if !strings.HasSuffix(tagName, "-dev") && !latest_encountered {
				latest_encountered = true
				continue
			}
			//
			if !strings.HasSuffix(tagName, "-dev") && latest_encountered {
				return tagName, nil
			}
		}
	}

	return "", fmt.Errorf("no valid tags found")
}

// GetSortedTags retrieves all tags from the DKN Compute Node repository on GitHub and returns them as a sorted list.
//
// It fetches the tags from the GitHub API and parses them into a list of maps. Each map represents a tag with its attributes.
//
// Returns:
//   - []map[string]interface{}: A slice of maps representing the tags from the repository, each containing tag attributes (e.g., name, commit).
//   - error: An error if the request fails, the response cannot be parsed, or no tags are found.
func GetSortedTags() ([]map[string]interface{}, error) {
	url := "https://api.github.com/repos/firstbatchxyz/dkn-compute-node/tags"

	// get and parse the all the tags
	resp, err := http.Get(url)
	if err != nil {
		return nil, fmt.Errorf("failed to make request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("request failed with status code: %d", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	var tags []map[string]interface{}
	if err := json.Unmarshal(body, &tags); err != nil {
		return nil, fmt.Errorf("failed to parse JSON: %w", err)
	}
	if len(tags) == 0 {
		return nil, fmt.Errorf("no tags found")
	}
	return tags, nil
}

// DownloadLatestComputeBinary downloads the latest compute binary for the current operating system and architecture
// from the DKN Compute Node GitHub repository, and saves it to the specified directory with the specified file name.
//
// Parameters:
//   - version: The version of the binary to download (e.g., v0.2.4).
//   - workingDir: The directory where the binary will be saved.
//   - file: The name of the file to save the binary as.
//
// Returns:
//   - error: An error if the download, file preparation, or version retrieval fails.
//
// Behavior:
//   - Constructs the download URL based on the provided version, operating system, and architecture.
//   - If the specified version cannot be downloaded (e.g., due to a 404 error), the function attempts to download the previous stable version.
//   - If the previous version download also fails, an error is returned.
//   - After downloading, the function applies necessary permissions to the binary by calling `PrepareComputeBinary`.
func DownloadLatestComputeBinary(version, workingDir, file string) error {
	os, arch := GetOSAndArch()
	extension := ""
	if os == "windows" {
		extension = ".exe"
	}
	asset_name := fmt.Sprintf("dkn-compute-binary-%s-%s%s", os, arch, extension)
	// releases/download/v0.2.4-dev
	url := fmt.Sprintf("https://github.com/firstbatchxyz/dkn-compute-node/releases/download/%s/%s", version, asset_name)
	destPath := filepath.Join(workingDir, file)
	status_code, err := DownloadFile(url, destPath)
	if err != nil {
		if status_code == 404 {
			// if the release exists but the downloads responds with 404, it means the build didn't finished yet
			// use the previous latest version
			fmt.Println("Warning: The latest compute binaries are currently being built. Downloading the previous version. You can restart the launcher in ~20 minutes to run the latest version.")
			version, err = GetComputeLatestTag(false, false, true)
			if err != nil {
				return err
			}
			asset_name := fmt.Sprintf("dkn-compute-binary-%s-%s%s", os, arch, extension)
			url := fmt.Sprintf("https://github.com/firstbatchxyz/dkn-compute-node/releases/download/%s/%s", version, asset_name)
			_, err = DownloadFile(url, destPath)
			if err != nil {
				// if its couldn't download the previous latest version, raise an error
				return err
			}
		} else {
			// raise error for any other status code
			return err
		}
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
