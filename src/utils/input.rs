use commands::util::longest_common_prefix;
use inquire::{autocompletion::Replacement, Autocomplete, CustomUserError};

#[derive(Clone)]
pub struct CustomAutocomplete {
    suggestions: Vec<String>,
}

impl CustomAutocomplete {
    pub fn new(suggestions: Vec<String>) -> Self {
        Self { suggestions }
    }
}

impl Autocomplete for CustomAutocomplete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        let input_lower = input.to_lowercase();
        Ok(self
            .suggestions
            .iter()
            .filter(|s| s.to_lowercase().contains(&input_lower))
            // NOTE(meshde): the following line converts Vec<&String> to Vec<String>
            .map(|s| s.clone())
            .collect())
    }
    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        Ok(match highlighted_suggestion {
            Some(suggestion) => Replacement::Some(suggestion),
            None => Replacement::Some(
                longest_common_prefix(
                    self.get_suggestions(input)
                        .unwrap()
                        .iter()
                        .map(|x| x.as_str())
                        .collect(),
                )
                .to_string(),
            ),
        })
    }
}
