use blade_lsp::parser::BladeParser;
use blade_lsp::phpactor::get_completion_list;
use blade_lsp::treesitter::get_node_from_cursor_position;
use dashmap::DashMap;
use log::info;
use std::fmt::Display;
use structured_logger::{async_json::new_writer, Builder};
use tokio::fs::File;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tree_sitter::Tree;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let current_log_dir = "/home/matthew/Projects/blade-lsp/logger.log";

    let log_file = File::options()
        .create(true)
        .append(true)
        .open(current_log_dir)
        .await
        .unwrap();

    let logger = Builder::with_level("info").with_target_writer("*", new_writer(log_file));
    logger.init();

    info!("Logger has been initialized");

    let (service, socket) = LspService::new(|client| Backend {
        client,
        entry_map: DashMap::new(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}

struct DocumentEntry {
    ast: Tree,
    source_code: String,
}

impl Display for DocumentEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source code: {}", self.source_code)
    }
}

struct Backend {
    client: Client,
    entry_map: DashMap<String, DocumentEntry>,
}

impl Backend {
    async fn handle_file_change(&self, text: String, uri: Url) {
        let mut parser = BladeParser::new();
        let err_msg = "An error occurred while parsing the tree.";

        self.entry_map
            .entry(uri.to_string())
            .and_modify(|entry| {
                *entry = DocumentEntry {
                    ast: parser.parse(&text, Some(&entry.ast)).expect(&err_msg),
                    source_code: text.clone(),
                }
            })
            .or_insert_with(|| DocumentEntry {
                ast: parser.parse(&text, None).expect(&err_msg),
                source_code: text.clone(),
            });
    }

    async fn handle_completion(&self, params: CompletionParams) -> Option<CompletionResponse> {
        let TextDocumentPositionParams {
            text_document,
            position,
        } = params.text_document_position;
        let Position { line, character } = position;

        let doc_entry = match self.entry_map.get(text_document.uri.as_str()) {
            Some(entry) => entry,
            None => {
                info!("Unable to get instance from the filetype ast map.");
                panic!()
            }
        };

        let closest_node = match get_node_from_cursor_position(&doc_entry.ast, line, character) {
            Some(node) => node,
            None => {
                info!("Error occurred while getting the node at the position of the cursor");
                panic!()
            }
        };

        let mut completion_list: Vec<CompletionItem> = Vec::new();

        if closest_node.grammar_name() == "php_only" {
            let node_text = closest_node
                .utf8_text(&doc_entry.source_code.as_bytes())
                .unwrap();

            let completion_items = match get_completion_list(node_text.to_string()).await {
                Ok(list) => list,
                Err(e) => {
                    info!("{}", e.to_string());
                    panic!()
                }
            };

            for item in completion_items {
                completion_list.push(CompletionItem::new_simple(item.label, item.documentation));
            }

            info!(
                "Completion data recieved from phpactor: {:?}",
                completion_list
            );

            Some(CompletionResponse::Array(completion_list))
        } else {
            info!("Node is not php");

            Some(CompletionResponse::Array(vec![CompletionItem::new_simple(
                "Nothing found".to_string(),
                "Nothing found".to_string(),
            )]))
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _init_info: InitializedParams) {
        self.client
            .show_message(MessageType::INFO, "Server has been initialized")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let TextDocumentItem { text, uri, .. } = params.text_document;
        info!("File Has been opened");
        self.handle_file_change(text, uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document,
            mut content_changes,
        } = params;

        self.handle_file_change(
            std::mem::take(&mut content_changes[0].text),
            text_document.uri,
        )
        .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        if let Some(list) = self.handle_completion(params).await {
            Ok(Some(list))
        } else {
            Ok(Some(CompletionResponse::Array(vec![
                CompletionItem::new_simple(
                    "Unable to find any details".to_string(),
                    "Skull".to_string(),
                ),
            ])))
        }
    }

    async fn hover(&self, _hover_info: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "This is some hovering text".to_string(),
            )),
            range: None,
        }))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_completion() {}
}
