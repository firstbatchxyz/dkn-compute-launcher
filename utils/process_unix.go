//go:build darwin || linux
// +build darwin linux

package utils

// Dummy test thing

import (
	"errors"
	"fmt"
	"os"
	"syscall"
	"time"
)

// IsProcessRunning checks if a process with the given PID is running on Unix-based systems.
func IsProcessRunning(pid int) bool {
	process, err := os.FindProcess(pid)
	if err != nil {
		return false
	}

	// Send signal 0 to check if the process is running
	err = process.Signal(syscall.Signal(0))
	return err == nil
}

func StopProcess(pid int) error {
	// Check if the process exists
	if _, err := os.FindProcess(pid); err != nil {
		return fmt.Errorf("could not find process: %w", err)
	}

	// Send SIGTERM to the process group
	err := syscall.Kill(-pid, syscall.SIGTERM)
	if err != nil {
		return fmt.Errorf("could not send SIGTERM to process: %w", err)
	}
	fmt.Printf("SIGTERM sent to process group %d\n", pid)

	// Wait for graceful termination
	timeout := time.After(10 * time.Second)
	ticker := time.NewTicker(500 * time.Millisecond)
	defer ticker.Stop()

	for {
		select {
		case <-timeout:
			// Escalate to SIGKILL if process does not terminate
			err = syscall.Kill(-pid, syscall.SIGKILL)
			if err != nil {
				return fmt.Errorf("could not kill process: %w", err)
			}
			fmt.Printf("SIGKILL sent to process group %d\n", pid)
			time.Sleep(1 * time.Second)
			if IsProcessRunning(pid) {
				return errors.New("process did not terminate even after SIGKILL")
			}
			return fmt.Errorf("process did not terminate gracefully; sent SIGKILL")
		case <-ticker.C:
			if !IsProcessRunning(pid) {
				fmt.Printf("Process group %d terminated successfully\n", pid)
				return nil
			}
		}
	}
}

func SetFileDescriptorLimit(limit uint64) error {
	var rLimit syscall.Rlimit
	rLimit.Max = limit
	rLimit.Cur = limit
	if err := syscall.Setrlimit(syscall.RLIMIT_NOFILE, &rLimit); err != nil {
		return err
	}

	return nil
}
