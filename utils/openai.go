package utils

import "strings"

// IsOpenAIRequired checks if any of the picked models require OpenAI by comparing them against a list of available OpenAI models.
//
// Parameters:
//   - picked_models: A comma-separated string of model names selected by the user.
//   - openai_models: A pointer to a slice of strings containing available OpenAI model names.
//
// Returns:
//   - bool: Returns true if any of the picked models match OpenAI models, indicating that OpenAI is required, otherwise false.
func IsOpenAIRequired(picked_models string, openai_models *[]string) bool {
	required := false
	for _, model := range strings.Split(picked_models, ",") {
		for _, openai_model := range *openai_models {
			if model == openai_model {
				required = true
				break
			}
		}
	}
	return required
}
