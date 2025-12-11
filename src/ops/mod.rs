use std::sync::Arc;

use serenity::http::Http;

mod interaction;
mod message;

deno_core::extension!(
    oakmoss_ops,
    ops = [
        message::op_log,
        message::op_send_message,
        interaction::op_send_interaction_response,
        interaction::op_upsert_guild_commands,
    ],
    options = { http: Arc<Http> },
    state = |state, options| {
        state.put(options.http.clone());
    }
);

pub fn extension(http: Arc<Http>) -> deno_core::Extension {
    oakmoss_ops::init(http)
}
