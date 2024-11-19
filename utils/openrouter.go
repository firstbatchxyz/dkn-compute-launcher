package utils

import "strings"

// IsOpenRouterRequired checks if any of the picked models require OpenRouter by comparing them against a list of available OR models.
//
// Parameters:
//   - picked_models: A comma-separated string of model names selected by the user.
//   - or_models: A pointer to a slice of strings containing available OpenRouter model names.
//
// Returns:
//   - bool: Returns true if any of the picked models match OR models, indicating that OpenRouter api key is required, otherwise false.
func IsOpenRouterRequired(picked_models string, or_models *[]string) bool {
	required := false
	for _, model := range strings.Split(picked_models, ",") {
		for _, or_model := range *or_models {
			if model == or_model {
				required = true
				break
			}
		}
	}
	return required
}
