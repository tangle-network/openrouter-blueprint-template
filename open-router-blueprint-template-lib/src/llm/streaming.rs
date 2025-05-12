use std::pin::Pin;
// use std::task::{Context, Poll};
// Removed unused import: async_trait::async_trait
use futures::stream::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use super::{
    ChatCompletionChoice, ChatCompletionResponse, ChatMessage, LlmError, Result,
    TextCompletionChoice, TextCompletionResponse,
};

/// A chunk of a streaming chat completion response
#[derive(Debug, Clone)]
pub struct ChatCompletionChunk {
    /// The ID of the completion
    pub id: String,

    /// The type of object (always "chat.completion.chunk")
    pub object: String,

    /// The timestamp of the chunk (Unix timestamp in seconds)
    pub created: u64,

    /// The model used for the completion
    pub model: String,

    /// The generated choices
    pub choices: Vec<ChatCompletionStreamChoice>,
}

/// A choice in a streaming chat completion response
#[derive(Debug, Clone)]
pub struct ChatCompletionStreamChoice {
    /// The index of this choice
    pub index: usize,

    /// The delta content for this chunk
    pub delta: ChatMessageDelta,

    /// The reason the generation stopped, if applicable
    pub finish_reason: Option<String>,
}

/// A delta for a chat message in a streaming response
#[derive(Debug, Clone)]
pub struct ChatMessageDelta {
    /// The role of the message sender, if this is the first chunk
    pub role: Option<String>,

    /// The content delta for this chunk
    pub content: Option<String>,
}

/// A chunk of a streaming text completion response
#[derive(Debug, Clone)]
pub struct TextCompletionChunk {
    /// The ID of the completion
    pub id: String,

    /// The type of object (always "text_completion.chunk")
    pub object: String,

    /// The timestamp of the chunk (Unix timestamp in seconds)
    pub created: u64,

    /// The model used for the completion
    pub model: String,

    /// The generated choices
    pub choices: Vec<TextCompletionStreamChoice>,
}

/// A choice in a streaming text completion response
#[derive(Debug, Clone)]
pub struct TextCompletionStreamChoice {
    /// The index of this choice
    pub index: usize,

    /// The text delta for this chunk
    pub text: String,

    /// The reason the generation stopped, if applicable
    pub finish_reason: Option<String>,
}

/// A stream of chat completion chunks
pub type ChatCompletionStream = Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>;

/// A stream of text completion chunks
pub type TextCompletionStream = Pin<Box<dyn Stream<Item = Result<TextCompletionChunk>> + Send>>;

// pub trait StreamingLlmClient: Send + Sync {
//     /// Process a streaming chat completion request
//     async fn streaming_chat_completion(
//         &self,
//         request: ChatCompletionRequest,
//     ) -> Result<ChatCompletionStream>;

//     /// Process a streaming text completion request
//     async fn streaming_text_completion(
//         &self,
//         request: TextCompletionRequest,
//     ) -> Result<TextCompletionStream>;
// }

/// Create a chat completion stream from a channel
pub fn create_chat_completion_stream(
    receiver: mpsc::Receiver<Result<ChatCompletionChunk>>,
) -> ChatCompletionStream {
    Box::pin(ReceiverStream::new(receiver))
}

/// Create a text completion stream from a channel
pub fn create_text_completion_stream(
    receiver: mpsc::Receiver<Result<TextCompletionChunk>>,
) -> TextCompletionStream {
    Box::pin(ReceiverStream::new(receiver))
}

/// Utility to collect a chat completion stream into a single response
pub async fn collect_chat_completion_stream(
    mut stream: ChatCompletionStream,
) -> Result<ChatCompletionResponse> {
    // let mut id = String::new();
    // let mut model = String::new();
    // let mut created = 0;
    let mut choices = Vec::new();

    // Process the first chunk to get metadata
    if let Some(first_chunk_result) = stream.next().await {
        let first_chunk = first_chunk_result?;
        // id = first_chunk.id;
        // model = first_chunk.model;
        // created = first_chunk.created;

        // Initialize choices with empty content
        for choice in first_chunk.choices {
            let role = choice.delta.role.unwrap_or_else(|| "assistant".to_string());
            choices.push((choice.index, role, String::new(), choice.finish_reason));
        }
    } else {
        return Err(LlmError::RequestFailed("Empty stream".to_string()));
    }

    // Process the rest of the chunks
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;

        for choice in chunk.choices {
            if let Some(content) = choice.delta.content {
                if let Some((_, _, content_buffer, _)) = choices
                    .iter_mut()
                    .find(|(idx, _, _, _)| *idx == choice.index)
                {
                    content_buffer.push_str(&content);
                }
            }

            if choice.finish_reason.is_some() {
                if let Some((_, _, _, finish_reason)) = choices
                    .iter_mut()
                    .find(|(idx, _, _, _)| *idx == choice.index)
                {
                    *finish_reason = choice.finish_reason;
                }
            }
        }
    }

    // Convert to ChatCompletionResponse
    let response_choices = choices
        .into_iter()
        .map(
            |(index, role, content, finish_reason)| ChatCompletionChoice {
                index,
                message: ChatMessage {
                    role,
                    content,
                    name: None,
                },
                finish_reason,
            },
        )
        .collect();

    Ok(ChatCompletionResponse {
        id: "stream-collected".to_string(),
        object: "chat.completion".to_string(),
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        model: "unknown".to_string(),
        choices: response_choices,
        usage: None, // Usage information is not available when streaming
    })
}

/// Utility to collect a text completion stream into a single response
pub async fn collect_text_completion_stream(
    mut stream: TextCompletionStream,
) -> Result<TextCompletionResponse> {
    // let mut id = String::new();
    // let mut model = String::new();
    // let mut created = 0;
    let mut choices = Vec::new();

    // Process the first chunk to get metadata
    if let Some(first_chunk_result) = stream.next().await {
        let first_chunk = first_chunk_result?;
        // id = first_chunk.id;
        // model = first_chunk.model;
        // created = first_chunk.created;

        // Initialize choices with empty content
        for choice in first_chunk.choices {
            choices.push((choice.index, choice.text, choice.finish_reason));
        }
    } else {
        return Err(LlmError::RequestFailed("Empty stream".to_string()));
    }

    // Process the rest of the chunks
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;

        for choice in chunk.choices {
            if let Some((_, text_buffer, _)) =
                choices.iter_mut().find(|(idx, _, _)| *idx == choice.index)
            {
                text_buffer.push_str(&choice.text);
            }

            if choice.finish_reason.is_some() {
                if let Some((_, _, finish_reason)) =
                    choices.iter_mut().find(|(idx, _, _)| *idx == choice.index)
                {
                    *finish_reason = choice.finish_reason;
                }
            }
        }
    }

    // Convert to TextCompletionResponse
    let response_choices = choices
        .into_iter()
        .map(|(index, text, finish_reason)| TextCompletionChoice {
            index,
            text,
            finish_reason,
        })
        .collect();

    Ok(TextCompletionResponse {
        id: "stream-collected".to_string(),
        object: "text_completion".to_string(),
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        model: "unknown".to_string(),
        choices: response_choices,
        usage: None, // Usage information is not available when streaming
    })
}
