use eyre::Context;
use serde::Deserialize;
use std::collections::HashMap;
use prettytable::{Table, row, cell};

#[derive(Deserialize)]
struct ApiResponse {
    models: HashMap<String, usize>,
}

pub async fn check_command() -> eyre::Result<()> {
    let url = "https://dkn.dria.co/dashboard/supply/v0/models/executions/per-model/last-week";

    let response = reqwest::get(url)
        .await
        .wrap_err("Could not make request")?;

    let data: ApiResponse = response
        .json()
        .await
        .wrap_err("Could not parse body")?;

    let mut models: Vec<(&String, &usize)> = data.models.iter().collect();
    models.sort_by(|a, b| b.1.cmp(a.1));
    let mut table = Table::new();
    table.add_row(row!["Model Name", "Tasks"]);

    for (model, task) in models {
        table.add_row(row![model, task]);
    }

    table.printstd();

    Ok(())
}
