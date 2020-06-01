use crate::protocol::{Connection, Message, Notification, Request};
use crate::Result;
use anyhow::anyhow;
use async_std::sync::RwLock;
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use ra_vfs::{RootEntry, Vfs};
use serde::{de::DeserializeOwned, Serialize};
use std::{path::PathBuf, sync::Arc};

enum Event {
    Msg(Message),
}

/// State for the language server
struct LanguageServerState {
    // Interface to the vfs, a virtual filesystem that supports the overlaying of files
    pub vfs: Arc<RwLock<Vfs>>,
    pub task_receiver: UnboundedReceiver<ra_vfs::VfsTask>,
}

/// Filter for to choose which files the ra_vfs should ignore
struct MunFilter {}

/// Implement the filter provided by ra_vfs
impl ra_vfs::Filter for MunFilter {
    fn include_dir(&self, _dir_path: &ra_vfs::RelativePath) -> bool {
        true
    }

    fn include_file(&self, file_path: &ra_vfs::RelativePath) -> bool {
        file_path.extension() == Some(".mun")
    }
}

impl LanguageServerState {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        // Create a channel for use by the vfs
        let (task_sender, task_receiver) = futures::channel::mpsc::unbounded();

        let task_sender = Box::new(move |t| task_sender.unbounded_send(t).unwrap());
        // Create the vfs
        let vfs = Vfs::new(
            roots
                .into_iter()
                .map(|root| RootEntry::new(root, Box::new(MunFilter {})))
                .collect(),
            task_sender,
            ra_vfs::Watch(true),
        );
        LanguageServerState {
            vfs: Arc::new(RwLock::new(vfs.0)),
            task_receiver,
        }
    }
}

/// Runs the main loop of the language server. This will receive requests and handle them.
pub async fn main_loop(mut connection: Connection, workspace_folders: Vec<PathBuf>) -> Result<()> {
    // Create the state for the language server
    let mut state = LanguageServerState::new(workspace_folders);
    loop {
        // Determine what to do next. This selects from different channels, the first message to
        // arrive is returned. If an error occurs on one of the channel the main loop is shutdown
        // with an error.
        let event = futures::select! {
            msg = connection.receiver.next() => match msg {
                Some(msg) => Event::Msg(msg),
                None => return Err(anyhow::anyhow!("client exited without shutdown")),
            }
        };

        // Handle the event
        match handle_event(event, &mut connection, &mut state).await? {
            LoopState::Continue => {}
            LoopState::Shutdown => {
                break;
            }
        }
    }

    Ok(())
}

/// A `LoopState` enumerator determines the state of the main loop
enum LoopState {
    Continue,
    Shutdown,
}

/// Cast a notification to a specific type
fn notification_cast<N>(notification: Notification) -> std::result::Result<N::Params, Notification>
where
    N: lsp_types::notification::Notification,
    N::Params: DeserializeOwned,
{
    notification.try_extract(N::METHOD)
}

/// Create an new notification with the specified parameters
fn notification_new<N>(params: N::Params) -> Notification
where
    N: lsp_types::notification::Notification,
    N::Params: Serialize,
{
    Notification::new(N::METHOD.to_string(), params)
}

/// Handles a received request
async fn handle_request(request: Request, connection: &mut Connection) -> Result<LoopState> {
    if connection.handle_shutdown(&request).await? {
        return Ok(LoopState::Shutdown);
    };
    Ok(LoopState::Continue)
}

/// Handles a received notification
async fn on_notification(
    notification: Notification,
    connection: &mut Connection,
    state: &LanguageServerState,
) -> Result<LoopState> {
    let notification =
        // When a a text document is opened
        match notification_cast::<lsp_types::notification::DidOpenTextDocument>(notification) {
            Ok(params) => {
                // Get the uri
                let uri = params.text_document.uri;
                // And convert into a file path
                let path = uri
                    .to_file_path()
                    .map_err(|()| anyhow!("invalid uri: {}", uri))?;
                if let Some(_) = state
                    .vfs
                    .write()
                    .await
                    .add_file_overlay(&path, params.text_document.text)
                {
                    //loop_state.subscriptions.add_sub(FileId(file_id.0));
                }
                return Ok(LoopState::Continue);
            }
            Err(not) => not,
        };

    // When a text document is closed
    let notification =
        match notification_cast::<lsp_types::notification::DidCloseTextDocument>(notification) {
            Ok(params) => {
                let uri = params.text_document.uri;
                let path = uri
                    .to_file_path()
                    .map_err(|()| anyhow!("invalid uri: {}", uri))?;
                if let Some(_) = state.vfs.write().await.remove_file_overlay(path.as_path()) {
                    //loop_state.subscriptions.remove_sub(FileId(file_id.0));
                }
                let params = lsp_types::PublishDiagnosticsParams {
                    uri,
                    diagnostics: Vec::new(),
                    version: None,
                };
                let not = notification_new::<lsp_types::notification::PublishDiagnostics>(params);
                connection.sender.try_send(not.into()).unwrap();
                return Ok(LoopState::Continue);
            }
            Err(not) => not,
        };

    match notification_cast::<lsp_types::notification::DidChangeTextDocument>(notification) {
        Ok(params) => {
            let lsp_types::DidChangeTextDocumentParams {
                text_document,
                content_changes,
            } = params;
            //let world = state.snapshot();
            //let file_id = from_proto::file_id(&world, &text_document.uri)?;
            //let line_index = world.analysis().file_line_index(file_id)?;
            let uri = text_document.uri;
            let path = uri
                .to_file_path()
                .map_err(|()| anyhow!("invalid uri: {}", uri))?;
            // TODO: I assume that since we are using *FULL* as the support change mode, that get
            // the text as a single change
            state
                .vfs
                .write()
                .await
                .change_file_overlay(&path, |old_text| {
                    // TODO: Change this to incremental later
                    *old_text = content_changes.get(0).unwrap().text.clone();
                });
            return Ok(LoopState::Continue);
        }
        Err(not) => not,
    };
    Ok(LoopState::Continue)
}

/// Handles an incoming event. Returns a `LoopState` state which determines whether processing
/// should continue.
#[allow(irrefutable_let_patterns)]
async fn handle_event(
    event: Event,
    connection: &mut Connection,
    state: &LanguageServerState,
) -> Result<LoopState> {
    if let Event::Msg(msg) = event {
        match msg {
            Message::Request(req) => handle_request(req, connection).await,
            Message::Response(_) => Ok(LoopState::Continue),
            Message::Notification(notification) => {
                on_notification(notification, connection, state).await
            }
        }
    } else {
        Ok(LoopState::Continue)
    }
}
