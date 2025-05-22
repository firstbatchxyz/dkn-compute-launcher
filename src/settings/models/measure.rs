use std::collections::HashSet;

use colored::Colorize;
use dkn_executor::ollama_rs::{
    generation::completion::{request::GenerationRequest, GenerationResponse},
    Ollama,
};
use dkn_executor::{Model, ModelProvider};
use inquire::MultiSelect;

use crate::utils::{check_ollama, pull_model_with_progress, DriaEnv};

const MINIMUM_EVAL_TPS: f64 = 15.0;
const MINIMUM_DURATION_MS: u64 = 120 * 1000;

/// Prompts the user to select Ollama models, and measures the TPS for each one.
/// The user can select multiple models to be benchmarked.
///
///
/// ### Errors
/// - If Ollama is not available / something is wrong about the chosen model.
pub async fn measure_tps(dria_env: &DriaEnv) -> eyre::Result<()> {
    // ensure Ollama is available
    if !check_ollama(dria_env).await {
        eyre::bail!("Ollama is not available, please run Ollama server.");
    }

    // get all Ollama models available
    let all_ollama_models = Model::all_with_provider(&ModelProvider::Ollama).collect::<Vec<_>>();

    // get users ollama models
    let models = dria_env.get_models();
    let my_ollama_models = models
        .iter()
        .filter(|m| m.provider() == ModelProvider::Ollama)
        .collect::<HashSet<_>>();

    // find indexes of existing chosen ollama models on the user
    let default_selected_idxs = all_ollama_models
        .iter()
        .enumerate()
        .filter_map(|(idx, model)| {
            if my_ollama_models.contains(model) {
                Some(idx)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // prompt the user to select models to be benchmarked
    let selected_ollama_models = MultiSelect::new(
        "Choose the Ollama models that you would like to measure:",
        all_ollama_models,
    )
    .with_default(&default_selected_idxs)
    .with_help_message(
        "↑↓ to move, SPACE to select one, ←/→ to select all/none, type to filter models, ENTER to confirm",
    )
    .prompt()?;

    if selected_ollama_models.is_empty() {
        log::info!("No models selected, exiting.");
        return Ok(());
    }

    // create a table
    let mut table = Table::default();

    // create ollama instance
    let (host, port) = dria_env.get_ollama_config();
    let ollama = Ollama::new(host, port);

    // get local models
    let local_model_names = ollama
        .list_local_models()
        .await?
        .into_iter()
        .map(|m| m.name)
        .collect::<Vec<_>>();

    // iterate over selected models and run a benchmark on each one
    log::info!(
        "Starting measurements (min TPS: {}, max duration: {}ms)",
        MINIMUM_EVAL_TPS,
        MINIMUM_DURATION_MS
    );
    for model in selected_ollama_models
        .into_iter()
        .filter(|m| m.provider() == ModelProvider::Ollama)
    {
        let model_name = model.to_string();

        if !local_model_names.contains(&model_name) {
            log::info!(
                "Model {} does not exist locally, pulling it from Ollama.",
                model_name
            );

            // pull the model with nice logs
            pull_model_with_progress(&ollama, model_name).await?;
        }

        // run a dummy generation for warm-up
        log::debug!("Warming up Ollama for model {}", model);
        if let Err(e) = ollama
            .generate(GenerationRequest::new(
                model.to_string(),
                "Write a short poem about hedgehogs and squirrels.".to_string(),
            ))
            .await
        {
            log::warn!("Ignoring model {}: Workflow failed with error {}", model, e);
            continue;
        }

        // generate a prompt
        log::info!("Measuring {}", model.to_string().bold());
        match ollama
            .generate(GenerationRequest::new(
                model.to_string(),
                "Write a poem about Kapadokya.".to_string(),
            ))
            .await
        {
            Ok(response) => {
                table.add_row(response.into());
            }
            Err(e) => {
                log::warn!("Model {} failed with error {}", model, e);
                continue;
            }
        }
    }

    // print the final result
    log::info!("Finished TPS measurements.");
    eprintln!("{}", table);

    Ok(())
}

struct TableRow {
    model: String,
    prompt_tps: f64,
    prompt_dur_ms: u64,
    eval_tps: f64,
    eval_dur_ms: u64,
    total_dur_ms: u64,
}

impl From<GenerationResponse> for TableRow {
    fn from(res: GenerationResponse) -> Self {
        let prompt_tps = (res.prompt_eval_count.unwrap_or_default() as f64)
            / (res.prompt_eval_duration.unwrap_or(1) as f64)
            * 1e9;

        let eval_tps = (res.eval_count.unwrap_or_default() as f64)
            / (res.eval_duration.unwrap_or(1) as f64)
            * 1e9;

        Self {
            model: res.model,
            prompt_tps,
            prompt_dur_ms: res.prompt_eval_duration.unwrap_or_default() / 1e6 as u64,
            eval_tps,
            eval_dur_ms: res.eval_duration.unwrap_or_default() / 1e6 as u64,
            total_dur_ms: res.total_duration.unwrap_or_default() / 1e6 as u64,
        }
    }
}

impl TableRow {
    fn print_row(&self) -> String {
        let eval_tps = self.eval_tps;
        let dur = self.total_dur_ms;
        format!(
            "{:<36} {:<12.4} {:<12} {} {:<12} {}",
            self.model,
            self.prompt_tps,
            self.prompt_dur_ms,
            if eval_tps > 1.5 * MINIMUM_EVAL_TPS {
                format!("{:<12.4}", eval_tps).green()
            } else if eval_tps > MINIMUM_EVAL_TPS {
                format!("{:<12.4}", eval_tps).yellow()
            } else {
                format!("{:<12.4}", eval_tps).red()
            },
            self.eval_dur_ms,
            if dur > MINIMUM_DURATION_MS {
                dur.to_string().red()
            } else if dur > MINIMUM_DURATION_MS / 2 {
                dur.to_string().yellow()
            } else {
                dur.to_string().green()
            },
        )
    }
}

#[derive(Default)]
struct Table {
    rows: Vec<TableRow>,
}
impl Table {
    #[inline]
    pub fn add_row(&mut self, row: TableRow) {
        self.rows.push(row);
    }

    /// Returns a line of header string.
    #[inline]
    fn get_header() -> String {
        format!(
            "{:<36} {:<12} {:<12} {:<12} {:<12} {}",
            "Model".bold(),
            "Prompt TPS".bold().dimmed(),
            "Time (ms)".bold().dimmed(),
            "Eval TPS".bold(),
            "Time (ms)".bold(),
            "Total (ms)".bold(),
        )
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", Self::get_header())?;

        for row in &self.rows {
            writeln!(f, "{}", row.print_row(),)?;
        }

        Ok(())
    }
}
