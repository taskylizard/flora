use std::{cell::RefCell, rc::Rc, sync::Arc};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use serde::Deserialize;
use serenity::{
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    http::Http,
    model::id::InteractionId,
};

use super::message::{
    AllowedMentionsInput, AttachmentInput, EmbedInput, build_allowed_mentions, build_attachment,
    build_embed,
};

#[derive(Deserialize)]
pub(crate) struct InteractionResponseArgs {
    #[serde(alias = "interactionId")]
    pub interaction_id: String,
    pub token: String,
    pub content: Option<String>,
    pub embeds: Option<Vec<EmbedInput>>,
    pub attachments: Option<Vec<AttachmentInput>>,
    pub tts: Option<bool>,
    #[serde(alias = "allowedMentions")]
    pub allowed_mentions: Option<AllowedMentionsInput>,
    pub ephemeral: Option<bool>,
}

#[op2(async)]
pub async fn op_send_interaction_response(
    state: Rc<RefCell<OpState>>,
    #[serde] args: InteractionResponseArgs,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };

    let interaction_id = args
        .interaction_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid interaction id"))?;

    let mut message = CreateInteractionResponseMessage::new();
    let mut has_content = false;
    let mut has_embeds = false;
    let mut has_attachments = false;
    let mut upload_files = Vec::new();

    if let Some(content) = args.content {
        message = message.content(content);
        has_content = true;
    }

    if let Some(tts) = args.tts {
        message = message.tts(tts);
    }

    if let Some(embeds) = args.embeds {
        let embeds = embeds.into_iter().map(build_embed).collect::<Result<Vec<_>, _>>()?;
        has_embeds = !embeds.is_empty();
        message = message.add_embeds(embeds);
    }

    if let Some(mentions) = args.allowed_mentions {
        message = message.allowed_mentions(build_allowed_mentions(mentions));
    }

    if let Some(ephemeral) = args.ephemeral {
        if ephemeral {
            message = message.ephemeral(true);
        }
    }

    if let Some(attachments) = args.attachments {
        let mut files = Vec::with_capacity(attachments.len());
        for attachment in attachments {
            files.push(build_attachment(&http, attachment).await?);
        }
        has_attachments = !files.is_empty();
        upload_files = files.clone();
        message = message.add_files(files);
    }

    if !has_content && !has_embeds && !has_attachments {
        return Err(JsErrorBox::generic("Response must include content, embeds, or attachments"));
    }

    let response = CreateInteractionResponse::Message(message);
    http.create_interaction_response(
        InteractionId::new(interaction_id),
        &args.token,
        &response,
        upload_files,
    )
    .await
    .map_err(|err| JsErrorBox::generic(err.to_string()))?;

    Ok(())
}
