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

func IsCommandAvailable(command string) bool {
	// LookPath searches for an executable named command in the directories
	// named by the PATH environment variable.
	_, err := exec.LookPath(command)
	return err == nil
}

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

func PickModels(openai_models, ollama_models []string) string {
	reader := bufio.NewReader(os.Stdin)
	fmt.Print("\nPlease pick the model you want to run:\n\n")
	fmt.Printf("ID\tProvider\tName\n")
	for id, model := range openai_models {
		fmt.Printf("%d\tOpenAI\t%s\n", id+1, model)
	}
	for id, model := range ollama_models {
		fmt.Printf("%d\tOllama\t%s\n", len(openai_models)+id+1, model)
	}
	fmt.Printf("Enter the model ids (comma seperated, e.g: 1,2,4): ")
	models, err := reader.ReadString('\n')
	if err != nil {
		return ""
	}
	models = strings.TrimSpace(models)
	models = strings.Split(models, "\n")[0]
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

func ExitWithDelay(code int) {
	fmt.Println("Terminating in 5 seconds...")
	time.Sleep(5 * time.Second)
	os.Exit(code)
}

func GetDknSecretKey() (string, error) {
	reader := bufio.NewReader(os.Stdin)
	// get DKN_WALLET_SECRET_KEY
	fmt.Print("Please enter your DKN Wallet Secret Key (32-bytes hex encoded): ")
	skey, err := reader.ReadString('\n')
	if err != nil {
		return "", fmt.Errorf("couldn't get DKN Wallet Secret Key")
	}
	skey = strings.TrimSpace(skey)
	skey = strings.Split(skey, "\n")[0]
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

type ModelList []string

func (models *ModelList) String() string {
	str := ""
	for _, m := range *models {
		str = fmt.Sprintf("%s, %s", str, m)
	}
	return str
}

func (models *ModelList) Set(value string) error {
	*models = append(*models, value)
	return nil
}
