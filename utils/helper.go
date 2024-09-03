package utils

import (
	"fmt"
	"strings"
)

// MapToList converts a map of string key-value pairs into a slice of strings,
// where each string is formatted as "key=value".
//
// Parameters:
//   - m: A map with string keys and string values.
//
// Returns:
//   - []string: A slice of strings, each representing a key-value pair from the map in the format "key=value".
func MapToList(m map[string]string) []string {
	var list []string
	for key, value := range m {
		list = append(list, fmt.Sprintf("%s=%s", key, value))
	}
	return list
}

// FormatMapKeys formats the keys of a map into a single string, with each key
// separated by a comma and enclosed in square brackets.
//
// Parameters:
//   - m: A map with string keys and boolean values.
//
// Returns:
//   - string: A string representation of the map's keys, formatted as "[key1, key2, ...]".
func FormatMapKeys(m map[string]bool) string {
	var keys []string
	for key := range m {
		keys = append(keys, key)
	}
	return "[" + strings.Join(keys, ", ") + "]"
}
