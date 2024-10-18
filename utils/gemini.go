package utils

import "strings"

// IsGeminiRequired checks if any of the picked models require Google Gemini by comparing them against a list of available Gemini models.
//
// Parameters:
//   - picked_models: A comma-separated string of model names selected by the user.
//   - gemini_models: A pointer to a slice of strings containing available Gemini model names.
//
// Returns:
//   - bool: Returns true if any of the picked models match Gemini models, indicating that Gemini is required, otherwise false.
func IsGeminiRequired(picked_models string, gemini_models *[]string) bool {
	required := false
	for _, model := range strings.Split(picked_models, ",") {
		for _, gemini_model := range *gemini_models {
			if model == gemini_model {
				required = true
				break
			}
		}
	}
	return required
}
