package utils

import (
	"bufio"
	"encoding/hex"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
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
// to stdout, wait for completion, and set custom environment variables.
//
// Parameters:
//   - working_dir: The directory where the command will be executed.
//   - printToStdout: If true, the command's stdout and stderr are connected to the terminal.
//   - wait: If true, waits for the command to finish before returning.
//   - envs: A slice of environment variables to set for the command, in the form of key=value.
//   - command: The command to execute.
//   - args: Additional arguments for the command.
//
// Returns:
//   - int: The PID of the started command.
//   - error: Returns an error if the command fails to start or completes with an error, otherwise nil.
func RunCommand(working_dir string, printToStdout, wait bool, envs []string, command string, args ...string) (int, error) {
	cmd := exec.Command(command, args...)

	// Set the environment variable
	cmd.Env = append(os.Environ(), envs...)

	// set working dir
	cmd.Dir = working_dir

	if printToStdout {
		// Connect stdout and stderr to the terminal
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
	} else {
		// Capture output if not printing to stdout
		cmd.Stdout = nil
		cmd.Stderr = nil
	}

	// Start the command
	err := cmd.Start()
	if err != nil {
		return 0, fmt.Errorf("failed to start command: %w", err)
	}

	// Get the PID
	pid := cmd.Process.Pid

	// Wait for the command to finish
	if wait {
		err = cmd.Wait()
		if err != nil {
			return pid, fmt.Errorf("command finished with error: %w", err)
		}
	}
	return pid, nil
}

// PickModels prompts the user to pick models from the available OpenAI and Ollama models.
//
// Parameters:
//   - openai_models: A slice of available OpenAI model names.
//   - ollama_models: A slice of available Ollama model names.
//
// Returns:
//   - string: A comma-separated string of selected model names.
func PickModels(openai_models, ollama_models []string) string {
	fmt.Print("\nPlease pick the model you want to run:\n\n")
	fmt.Printf("ID\tProvider\tName\n")
	for id, model := range openai_models {
		fmt.Printf("%d\tOpenAI\t%s\n", id+1, model)
	}
	for id, model := range ollama_models {
		fmt.Printf("%d\tOllama\t%s\n", len(openai_models)+id+1, model)
	}
	models := GetUserInput("Enter the model ids (comma separated, e.g: 1,2,4): ", true)

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
		} else if id > len(openai_models) && id <= len(ollama_models)+len(openai_models) {
			// ollama model picked
			if !picked_models_map[id] {
				// if not already picked, add it to bin
				picked_models_map[id] = true
				picked_models_str = fmt.Sprintf("%s,%s", picked_models_str, ollama_models[id-len(openai_models)-1])
			}
		} else {
			// out of index, invalid
			invalid_selections[i] = true
			continue
		}
	}
	if len(invalid_selections) != 0 {
		fmt.Printf("Skipping the invalid selections: %s \n", FormatMapKeys(invalid_selections))
	}
	fmt.Printf("\n")
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
	skey := GetUserInput("Please enter your DKN Wallet Secret Key (32-bytes hex encoded): ", true)
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
