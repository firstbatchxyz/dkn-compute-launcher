package utils

import "strings"

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
