//go:build darwin || linux
// +build darwin linux

package utils

import (
	"fmt"
	"os"
	"syscall"
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

// StopProcess stops a process by its PID on Unix-based systems.
func StopProcess(pid int) error {
	process, err := os.FindProcess(pid)
	if err != nil {
		return fmt.Errorf("could not find process: %w", err)
	}

	// Send SIGTERM (soft termination)
	err = process.Signal(syscall.SIGTERM)
	if err != nil {
		return fmt.Errorf("could not terminate process: %w", err)
	}

	return nil
}
