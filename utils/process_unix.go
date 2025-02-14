//go:build darwin || linux
// +build darwin linux

package utils

import (
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

	// termination might take some time and it will effect the next steps during update, sleep 5 seconds just in case
	time.Sleep(5 * time.Second)

	return nil
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
