package utils

import (
	"fmt"
	"strings"
)

func MapToList(m map[string]string) []string {
	var list []string
	for key, value := range m {
		list = append(list, fmt.Sprintf("%s=%s", key, value))
	}
	return list
}

func FormatMapKeys(m map[string]bool) string {
	var keys []string
	for key := range m {
		keys = append(keys, key)
	}
	return "[" + strings.Join(keys, ", ") + "]"
}
