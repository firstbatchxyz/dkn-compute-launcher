//go:build windows
// +build windows

package utils

import (
	"fmt"
	"syscall"
	"time"
)

// IsProcessRunning checks if a process with the given PID is running on Windows.
func IsProcessRunning(pid int) bool {
	handle, err := syscall.OpenProcess(syscall.PROCESS_QUERY_INFORMATION, false, uint32(pid))
	if err != nil {
		return false // Process not running or no permissions
	}
	syscall.CloseHandle(handle) // Close handle after checking
	return true
}

// StopProcess stops a process by its PID on Windows.
func StopProcess(pid int) error {
	handle, err := syscall.OpenProcess(syscall.PROCESS_TERMINATE, false, uint32(pid))
	if err != nil {
		return fmt.Errorf("could not open process: %w", err)
	}
	defer syscall.CloseHandle(handle)

	// Terminate the process with an exit code of 1
	err = syscall.TerminateProcess(handle, 1)
	if err != nil {
		return fmt.Errorf("could not terminate process: %w", err)
	}

	// in windows termination might take some time and it will affect the next steps during update, sleep 5 seconds just in case
	time.Sleep(5 * time.Second)

	return nil
}
