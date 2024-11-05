package utils

import (
	"bufio"
	"context"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"syscall"
	"time"
)

// IsCommandAvailable checks if a given command is available in the system PATH.
//
// Parameters:
//   - command: The name of the command to check for availability.
//
// Returns:
//   - bool: Returns true if the command is available, otherwise false.
func IsCommandAvailable(command string) bool {
	_, err := exec.LookPath(command)
	return err == nil
}

// RunCommand executes a command in a specified working directory, with options to print output
// to stdout, wait for completion, set custom environment variables, and optionally specify a timeout.
//
// Parameters:
//   - working_dir: The directory where the command will be executed.
//   - printToStdout: If true, the command's stdout and stderr are connected to the terminal.
//   - wait: If true, waits for the command to finish before returning.
//   - timeout: If provided, the duration after which the command will be killed. If 0, no timeout will be applied.
//   - envs: A slice of environment variables to set for the command, in the form of key=value.
//   - command: The command to execute.
//   - args: Additional arguments for the command.
//
// Returns:
//   - int: The PID of the started command.
//   - error: Returns an error if the command fails to start, times out, or completes with an error.
func RunCommand(working_dir string, outputDest string, wait bool, timeout time.Duration, envs []string, command string, args ...string) (int, error) {
	var cmd *exec.Cmd
	var ctx context.Context
	var cancel context.CancelFunc

	// Create the command with or without a timeout depending on the timeout value
	if timeout > 0 {
		// Create a context with timeout
		ctx, cancel = context.WithTimeout(context.Background(), timeout)
		defer cancel()
		cmd = exec.CommandContext(ctx, command, args...)
	} else {
		// No timeout, use regular command
		cmd = exec.Command(command, args...)
	}

	// Set environment variables
	cmd.Env = append(os.Environ(), envs...)

	// Set working dir
	cmd.Dir = working_dir

	var logFile *os.File
	var err error

	// Set output handling based on outputDest
	switch outputDest {
	case "stdout":
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
	case "file":
		logFile, err = os.OpenFile(filepath.Join(working_dir, "logs.txt"), os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0644)
		if err != nil {
			return 0, fmt.Errorf("failed to open log file: %w", err)
		}
		// Close the log file when the function ends
		defer logFile.Close()
		cmd.Stdout = logFile
		cmd.Stderr = logFile
	case "none":
		cmd.Stdout = nil
		cmd.Stderr = nil
	default:
		return 0, fmt.Errorf("invalid output destination: %s", outputDest)
	}

	// Start the command
	err = cmd.Start()
	if err != nil {
		return 0, fmt.Errorf("failed to start command: %w", err)
	}

	// Get the PID
	pid := cmd.Process.Pid

	// If wait is false, handle output asynchronously
	if !wait {
		go func() {
			// Ensure to check if logFile is not nil
			if logFile != nil {
				// Start goroutines to copy the command's stdout and stderr to the log file
				stdoutPipe, stdoutErr := cmd.StdoutPipe()
				stderrPipe, stderrErr := cmd.StderrPipe()

				// Check for pipe errors before starting goroutines
				if stdoutErr == nil && stdoutPipe != nil {
					go io.Copy(logFile, stdoutPipe)
				}
				if stderrErr == nil && stderrPipe != nil {
					go io.Copy(logFile, stderrPipe)
				}
			}
			// Ensure the process runs to completion
			cmd.Wait()
		}()
	} else {
		// If wait is true, wait for the command to finish
		err = cmd.Wait()
		if timeout > 0 && ctx.Err() == context.DeadlineExceeded {
			return pid, ctx.Err()
		}
		if err != nil {
			return pid, fmt.Errorf("command finished with error; %w", err)
		}
	}

	return pid, nil
}

// PickModels prompts the user to pick models from the available OpenAI, Google and Ollama models.
//
// Parameters:
//   - openai_models: A slice of available OpenAI model names.
//   - gemini_models: A slice of available Gemini model names.
//   - ollama_models: A slice of available Ollama model names.
//
// Returns:
//   - string: A comma-separated string of selected model names.
func PickModels(openai_models, gemini_models, ollama_models []string) string {

	// column widths
	idWidth := 4
	providerWidth := 10
	nameWidth := 50

	header := fmt.Sprintf("| %-*s | %-*s | %-*s |", idWidth, "ID", providerWidth, "Provider", nameWidth, "Name")
	separator := "+" + strings.Repeat("-", idWidth+2) + "+" + strings.Repeat("-", providerWidth+2) + "+" + strings.Repeat("-", nameWidth+2) + "+"

	// print the table
	fmt.Print("\nPlease pick the model you want to run:\n\n")
	fmt.Println(separator)
	fmt.Println(header)
	fmt.Println(separator)

	// print the rows
	for id, model := range openai_models {
		modelId := id + 1
		provider := "OpenAI"
		fmt.Printf("| %-*d | %-*s | %-*s |\n", idWidth, modelId, providerWidth, provider, nameWidth, model)
	}
	for id, model := range gemini_models {
		modelId := len(openai_models) + id + 1
		provider := "Gemini"
		fmt.Printf("| %-*d | %-*s | %-*s |\n", idWidth, modelId, providerWidth, provider, nameWidth, model)
	}
	for id, model := range ollama_models {
		modelId := len(openai_models) + len(gemini_models) + id + 1
		provider := "Ollama"
		fmt.Printf("| %-*d | %-*s | %-*s |\n", idWidth, modelId, providerWidth, provider, nameWidth, model)
	}

	// print end
	fmt.Println(separator)

	models := GetUserInput("Enter the model ids (comma separated, e.g: 1,2,4) ", true)

	models = strings.ReplaceAll(models, " ", "")
	models_list := strings.Split(models, ",")
	picked_models_map := make(map[int]bool, 0)
	picked_models_str := ""
	invalid_selections := make(map[string]bool, 0)
	for _, i := range models_list {
		// if selection is already in invalids list, continue
		if invalid_selections[i] || i == "" {
			continue
		}

		id, err := strconv.Atoi(i)
		if err != nil {
			// not integer, invalid
			invalid_selections[i] = true
			continue
		}
		if id > 0 && id <= len(openai_models) {
			// openai model picked
			if !picked_models_map[id] {
				// if not already picked, add it to bin
				picked_models_map[id] = true
				picked_models_str = fmt.Sprintf("%s,%s", picked_models_str, openai_models[id-1])
			}
		} else if id > len(openai_models) && id <= len(gemini_models)+len(openai_models) {
			// gemini model picked
			if !picked_models_map[id] {
				// if not already picked, add it to bin
				picked_models_map[id] = true
				picked_models_str = fmt.Sprintf("%s,%s", picked_models_str, gemini_models[id-len(openai_models)-1])
			}
		} else if id > len(openai_models)+len(gemini_models) && id <= len(ollama_models)+len(gemini_models)+len(openai_models) {
			// ollama model picked
			if !picked_models_map[id] {
				// if not already picked, add it to bin
				picked_models_map[id] = true
				picked_models_str = fmt.Sprintf("%s,%s", picked_models_str, ollama_models[id-len(gemini_models)-len(openai_models)-1])
			}
		} else {
			// out of index, invalid
			invalid_selections[i] = true
			continue
		}
	}
	if len(invalid_selections) != 0 {
		fmt.Printf("Skipping the invalid selections: %s \n\n", FormatMapKeys(invalid_selections))
	}
	return picked_models_str
}

// GetUserInput reads a line of input from the terminal and optionally trims spaces.
//
// Parameters:
//   - message: The message to display to the user before reading input.
//   - trim: If true, trims spaces from the input.
//
// Returns:
//   - string: The user's input as a trimmed string.
func GetUserInput(message string, trim bool) string {
	reader := bufio.NewReader(os.Stdin)
	fmt.Printf("%s: ", message)
	answer, err := reader.ReadString('\n')
	if err != nil {
		return ""
	}
	answer = strings.TrimSpace(answer)
	answer = strings.Split(answer, "\n")[0]
	if trim {
		answer = strings.ReplaceAll(answer, " ", "")
	}
	fmt.Println()
	return answer
}

// ExitWithDelay exits the program with a specified exit code after a 5-second delay.
//
// Parameters:
//   - code: The exit code to return when terminating the program.
func ExitWithDelay(code int) {
	fmt.Println("Terminating in 5 seconds...")
	time.Sleep(5 * time.Second)
	os.Exit(code)
}

// GetDknSecretKey prompts the user to enter their DKN Wallet Secret Key, validates it, and returns it.
//
// Returns:
//   - string: The validated DKN Wallet Secret Key.
//   - error: Returns an error if the key is not 32-bytes hex encoded or if there are decoding issues.
func GetDknSecretKey() (string, error) {
	skey := GetUserInput("Please enter your DKN Wallet Secret Key (32-bytes hex encoded) ", true)
	skey = strings.TrimPrefix(skey, "0x")
	// decode the hex string into bytes
	decoded_skey, err := hex.DecodeString(skey)
	if err != nil {
		return "", fmt.Errorf("DKN Wallet Secret Key should be 32-bytes hex encoded")
	}
	// ensure the decoded bytes are exactly 32 bytes
	if len(decoded_skey) != 32 {
		return "", fmt.Errorf("DKN Wallet Secret Key should be 32 bytes long")
	}
	return skey, nil
}

// ModelList is a type that allows multiple values for the -m command-line flag.
type ModelList []string

// String returns the ModelList as a comma-separated string.
func (models *ModelList) String() string {
	str := ""
	for _, m := range *models {
		str = fmt.Sprintf("%s, %s", str, m)
	}
	return str
}

// Set appends a new model name to the ModelList.
//
// Parameters:
//   - value: The model name to add to the list.
//
// Returns:
//   - error: Returns nil as there are no constraints to enforce here.
func (models *ModelList) Set(value string) error {
	*models = append(*models, value)
	return nil
}

// isProcessRunning checks if a process with the given PID is running.
func IsProcessRunning(pid int) bool {
	// Try to find the process
	process, err := os.FindProcess(pid)
	if err != nil {
		// If there's an error finding the process, it's not running
		return false
	}

	// Try to send signal 0 to the process (this does not kill it)
	err = process.Signal(syscall.Signal(0))
	return err == nil
}

// stopProcess stops a process by its PID.
func StopProcess(pid int) error {
	// Find the process by PID
	process, err := os.FindProcess(pid)
	if err != nil {
		return fmt.Errorf("could not find process: %w", err)
	}

	// Send the SIGTERM signal to the process to terminate it gracefully
	if err := process.Signal(syscall.SIGTERM); err != nil {
		return fmt.Errorf("could not terminate process: %w", err)
	}

	return fmt.Errorf("")
}

// renameFile renames a file in the given working directory.
func RenameFile(workingDir, oldName, newName string) error {
	// Construct full paths for the old and new file names
	oldPath := filepath.Join(workingDir, oldName)
	newPath := filepath.Join(workingDir, newName)

	// Rename the file
	if err := os.Rename(oldPath, newPath); err != nil {
		return fmt.Errorf("could not rename file: %w", err)
	}

	return nil
}

// deleteFile deletes a file in the given working directory.
func DeleteFile(workingDir, fileName string) error {
	// Construct the full path to the file
	filePath := filepath.Join(workingDir, fileName)

	// Delete the file
	if err := os.Remove(filePath); err != nil {
		return fmt.Errorf("could not delete file: %w", err)
	}

	return nil
}
