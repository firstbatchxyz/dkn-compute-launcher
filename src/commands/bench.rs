use dkn_workflows::{Model, ModelProvider};
use eyre::eyre;
use futures::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::MultiSelect;
use ollama_rs::{
    error::OllamaError,
    generation::completion::{request::GenerationRequest, GenerationResponse},
    Ollama,
};
use prettytable::{Cell, Row, Table};

use crate::utils::{check_ollama, DriaEnv};

pub async fn run_benchmarks() -> eyre::Result<()> {
    let dria_env = DriaEnv::new_from_env();

    // ensure Ollama is available
    if !check_ollama(&dria_env).await {
        return Err(eyre!("Ollama is not available, please run Ollama server."));
    }

    // get all Ollama models available
    let all_ollama_models = Model::all_with_provider(ModelProvider::Ollama).collect::<Vec<_>>();

    // get users ollama models
    let models_config = dria_env.get_model_config();
    let my_ollama_models = models_config
        .models
        .iter()
        .filter_map(|(p, m)| {
            if *p == ModelProvider::Ollama {
                Some(m.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

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
    let Some(selected_ollama_models) = MultiSelect::new(
        "Choose the Ollama models that you would like to measure:",
        all_ollama_models,
    )
    .with_default(&default_selected_idxs)
    .with_help_message(
        "↑↓ to move, space to select one, → to all, ← to none, type to filter, ESC to go back",
    )
    .prompt_skippable()?
    else {
        return Ok(());
    };

    // create a table
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Model"),
        Cell::new("Prompt TPS"),
        Cell::new("Eval TPS"),
    ]));

    // create ollama instance
    let (host, port) = dria_env.get_ollama_config();
    let ollama = Ollama::new(host, port.parse().unwrap_or(11434));

    // get local models
    let local_model_names = ollama
        .list_local_models()
        .await?
        .into_iter()
        .map(|m| m.name)
        .collect::<Vec<_>>();

    // iterate over selected models and run a benchmark on each
    log::info!(
        "Measuring models: {}",
        selected_ollama_models
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    for model in selected_ollama_models
        .into_iter()
        .filter(|m| ModelProvider::from(m.clone()) == ModelProvider::Ollama)
    {
        let model_name = model.to_string();

        if local_model_names.contains(&model_name) {
            log::debug!("Model {} exists locally.", model_name);
        } else {
            log::info!(
                "Model {} does not exist locally, pulling it from Ollama.",
                model_name
            );

            // pull the model with nice logs
            let mut pull_stream = ollama.pull_model_stream(model_name.clone(), false).await?;
            let mut pull_error: Option<OllamaError> = None;
            let mut pull_bar: Option<ProgressBar> = None;
            while let Some(status) = pull_stream.next().await {
                match status {
                    Ok(status) => {
                        // if there is a bar & status, log it
                        if let Some(ref pb) = pull_bar {
                            if let Some(completed) = status.completed {
                                pb.set_position(completed);
                            }
                        } else
                        // otherwise try to create bar
                        if let Some(total) = status.total {
                            let pb = ProgressBar::new(total);
                            pb.set_message(format!("Pulling {}", model_name));
                            // styles taken from `self_update` to be coherent with the rest of the app
                            pb.set_style(
                                ProgressStyle::default_bar()
                                    .template("[{elapsed_precise}] [{bar:40}] {bytes}/{total_bytes} ({eta}) {msg}")?
                                    .progress_chars("=>-"),
                            );
                            pull_bar = Some(pb);
                        }
                    }
                    Err(err) => {
                        pull_error = Some(err);
                        break;
                    }
                }
            }
            if let Some(err) = pull_error {
                log::error!("Failed to pull model {}: {:?}", model, err);
                continue;
            } else if let Some(pb) = pull_bar {
                pb.finish_with_message("Pull complete.");
            }
        }

        // generate a prompt
        log::info!("Measuring TPS for model {}", model);
        match ollama
            .generate(GenerationRequest::new(
                model.to_string(),
                "Write a poem about hedgehogs and squirrels.".to_string(),
            ))
            .await
        {
            Ok(response) => {
                log::debug!("Got response for model {}", model);

                table.add_row(Row::new(vec![
                    Cell::new(&model.to_string()),
                    Cell::new(&get_response_prompt_tps(&response).to_string()),
                    Cell::new(&get_response_eval_tps(&response).to_string()),
                ]));
            }
            Err(e) => {
                log::warn!("Model {} failed with error {}", model, e);
                continue;
            }
        }
    }

    // print the final result
    log::info!("Finished TPS measurements.");
    if let Err(e) = table.print_tty(false) {
        log::error!("Failed to print eval table: {}", e);
    }

    Ok(())
}

#[inline]
fn get_response_eval_tps(res: &GenerationResponse) -> f64 {
    (res.eval_count.unwrap_or_default() as f64) / (res.eval_duration.unwrap_or(1) as f64)
        * 1_000_000_000f64
}

#[inline]
fn get_response_prompt_tps(res: &GenerationResponse) -> f64 {
    (res.prompt_eval_count.unwrap_or_default() as f64)
        / (res.prompt_eval_duration.unwrap_or(1) as f64)
        * 1_000_000_000f64
}
