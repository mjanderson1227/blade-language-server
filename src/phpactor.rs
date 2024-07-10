use completion_types::*;
use serde_json::json;
use std::process::Stdio;
use tokio::{io::AsyncWriteExt, process::Command};

pub mod completion_types {
    use serde::Deserialize;
    use serde_json::Value;

    #[derive(Deserialize, Debug)]
    pub struct CompletionResponse {
        pub version: String,
        pub action: String,
        pub parameters: CompletionParameter,
    }

    #[derive(Deserialize, Debug)]
    pub struct CompletionParameter {
        pub value: CompletionValue,
    }

    #[derive(Deserialize, Debug)]
    pub struct CompletionValue {
        pub suggestions: Vec<CompletionItem>,
        // TODO: Maybe figure out what this is later.
        pub issues: Vec<Value>,
    }

    #[derive(Deserialize, Debug)]
    pub struct CompletionItem {
        #[serde(rename = "type")]
        pub completion_type: String,
        pub name: String,
        pub snippet: String,
        pub label: String,
        pub short_description: String,
        pub documentation: String,
        pub class_import: Option<String>,
        pub name_import: Option<String>,
        pub fqn: Option<String>,
        pub range: Option<String>,
        pub info: String,
    }
}

// TODO: Make this function return a result so that tower lsp can handle it directly.
pub async fn get_completion_list(text: String) -> std::io::Result<Vec<CompletionItem>> {
    let mut process = Command::new("phpactor")
        .arg("rpc")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let command = json!({
            "action": "complete",
            "parameters": { "source": text, "offset": text.len()}
    });

    if let Some(mut proc_stdin) = process.stdin.take() {
        proc_stdin.write_all(command.to_string().as_bytes()).await?;
    } else {
        process.kill().await?;
    }

    let output = process.wait_with_output().await?;
    let rpc_response = serde_json::from_slice::<CompletionResponse>(&output.stdout)?;

    Ok(rpc_response.parameters.value.suggestions)
}

#[cfg(test)]
mod tests {
    use super::get_completion_list;

    #[tokio::test]
    async fn check_completion() {
        let statement = "<?php $foo = new Exception(); $foo->".to_string();

        if let Ok(completion_items) = get_completion_list(statement).await {
            completion_items
                .iter()
                .for_each(|item| println!("{:?}", item));
        } else {
            panic!("An error occurred while calling the function.");
        }
    }

    #[tokio::test]
    async fn check_completion_str_replace() {
        let statement = "<?php $s = 'something'; str_rep".to_string();

        if let Ok(completion_items) = get_completion_list(statement).await {
            completion_items
                .iter()
                .for_each(|item| println!("{:?}", item));
        } else {
            panic!("An error occurred while calling the function.");
        }
    }

    #[tokio::test]
    async fn check_completion_with_invalid_statement() {
        let statement = "<?php $foo->".to_string();

        if let Ok(completion_items) = get_completion_list(statement).await {
            completion_items
                .iter()
                .for_each(|item| println!("{:?}", item));
        } else {
            panic!("An error occurred while calling the function.");
        }
    }
}
