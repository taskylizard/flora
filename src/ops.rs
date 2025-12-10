use std::{cell::RefCell, rc::Rc, sync::Arc};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use serde::Deserialize;
use serenity::{
    builder::CreateMessage,
    http::Http,
    model::id::{ChannelId, MessageId},
};
use tracing::info;

#[derive(Deserialize)]
pub struct SendMessageArgs {
    pub channel_id: String,
    pub content: String,
    pub message_id: Option<String>,
}

#[op2]
pub fn op_log(_state: &mut OpState, #[serde] args: Vec<serde_json::Value>) {
    let text = args
        .into_iter()
        .map(|v| match v {
            serde_json::Value::String(s) => s,
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ");

    info!(target = "oakmoss:js", "{}", text);
}

#[op2(async)]
pub async fn op_send_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: SendMessageArgs,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };

    let channel_id_num = args
        .channel_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid channel id"))?;
    let channel_id = ChannelId::new(channel_id_num);
    tracing::info!(
        target: "oakmoss:ops",
        "op_send_message channel={} reply_to={:?}",
        channel_id,
        args.message_id
    );
    if let Some(message_id_str) = args.message_id {
        let message_id = message_id_str
            .parse::<u64>()
            .map_err(|_| JsErrorBox::generic("Invalid message id"))?;
        let reference = MessageId::new(message_id);
        let message = CreateMessage::new()
            .content(args.content)
            .reference_message((channel_id, reference));
        channel_id
            .send_message(&http, message)
            .await
            .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    } else {
        channel_id
            .say(&http, args.content)
            .await
            .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    }
    Ok(())
}

deno_core::extension!(
    oakmoss_ops,
    ops = [op_log, op_send_message],
    options = { http: Arc<Http> },
    state = |state, options| {
        state.put(options.http.clone());
    }
);

pub fn extension(http: Arc<Http>) -> deno_core::Extension {
    oakmoss_ops::init(http)
}
