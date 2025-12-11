use std::sync::Arc;

use serde::Serialize;
use serenity::all::{
    ChannelId, CommandInteraction, Context, EventHandler, GuildId, Interaction, Message, MessageId,
    MessageUpdateEvent, Ready, User, async_trait,
};
use tracing::{error, info};

use crate::runtime::BotRuntime;

#[derive(Clone)]
pub struct DiscordHandler {
    pub runtime: Arc<BotRuntime>,
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        let payload = ReadyPayload::from(&ready);
        if let Err(err) = self
            .runtime
            .dispatch_js_event("ready", None, serde_json::to_value(payload).unwrap_or_default())
            .await
        {
            error!("dispatch_js_event (ready) error: {:?}", err);
        }
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        info!(
            target: "oakmoss:discord",
            "message event channel={} author={} content={}",
            msg.channel_id,
            msg.author.id,
            msg.content
        );
        let payload = MessagePayload::from(&msg);
        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message payload: {:?}", err);
                return;
            }
        };

        let guild_id = msg.guild_id.map(|guild| guild.get().to_string());
        if let Err(err) = self.runtime.dispatch_js_event("messageCreate", guild_id, value).await {
            error!("dispatch_js_event error: {:?}", err);
        }
    }

    async fn message_update(
        &self,
        _ctx: Context,
        old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        let payload = MessageUpdatePayload::from_parts(old, new, &event);
        let guild_id = payload.guild_id.clone();
        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message update payload: {:?}", err);
                return;
            }
        };

        if let Err(err) = self.runtime.dispatch_js_event("messageUpdate", guild_id, value).await {
            error!("dispatch_js_event (messageUpdate) error: {:?}", err);
        }
    }

    async fn message_delete(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
    ) {
        let payload = MessageDeletePayload {
            id: deleted_message_id.get().to_string(),
            channel_id: channel_id.get().to_string(),
            guild_id: guild_id.map(|g| g.get().to_string()),
        };
        let guild_id = payload.guild_id.clone();

        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message delete payload: {:?}", err);
                return;
            }
        };

        if let Err(err) = self.runtime.dispatch_js_event("messageDelete", guild_id, value).await {
            error!("dispatch_js_event (messageDelete) error: {:?}", err);
        }
    }

    async fn message_delete_bulk(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        multiple_deleted_messages_ids: Vec<MessageId>,
        guild_id: Option<GuildId>,
    ) {
        let payload = MessageDeleteBulkPayload {
            ids: multiple_deleted_messages_ids.into_iter().map(|id| id.get().to_string()).collect(),
            channel_id: channel_id.get().to_string(),
            guild_id: guild_id.map(|g| g.get().to_string()),
        };
        let guild_id = payload.guild_id.clone();

        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message bulk delete payload: {:?}", err);
                return;
            }
        };

        if let Err(err) = self.runtime.dispatch_js_event("messageDeleteBulk", guild_id, value).await
        {
            error!("dispatch_js_event (messageDeleteBulk) error: {:?}", err);
        }
    }

    async fn interaction_create(&self, _ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                info!(
                    target: "oakmoss:discord",
                    "slash command interaction guild={:?} channel={} name={}",
                    command.guild_id,
                    command.channel_id,
                    command.data.name
                );

                let payload = InteractionCreatePayload::from(&command);
                let guild_id = payload.guild_id.clone();
                let value = match serde_json::to_value(payload) {
                    Ok(value) => value,
                    Err(err) => {
                        error!("Failed to serialize interaction payload: {:?}", err);
                        return;
                    }
                };

                if let Err(err) =
                    self.runtime.dispatch_js_event("interactionCreate", guild_id, value).await
                {
                    error!("dispatch_js_event (interactionCreate) error: {:?}", err);
                }
            }
            _ => {}
        }
    }
}

#[derive(Serialize)]
struct UserPayload {
    id: String,
    username: String,
    discriminator: Option<u16>,
    bot: bool,
}

impl From<&User> for UserPayload {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.get().to_string(),
            username: user.name.clone(),
            discriminator: user.discriminator.map(|d| d.get()),
            bot: user.bot,
        }
    }
}

#[derive(Serialize)]
struct MessagePayload {
    id: String,
    channel_id: String,
    guild_id: Option<String>,
    content: String,
    author: UserPayload,
}

impl From<&Message> for MessagePayload {
    fn from(msg: &Message) -> Self {
        Self {
            id: msg.id.get().to_string(),
            channel_id: msg.channel_id.get().to_string(),
            guild_id: msg.guild_id.map(|g| g.get().to_string()),
            content: msg.content.clone(),
            author: UserPayload {
                id: msg.author.id.get().to_string(),
                username: msg.author.name.clone(),
                discriminator: msg.author.discriminator.map(|d| d.get()),
                bot: msg.author.bot,
            },
        }
    }
}

#[derive(Serialize)]
struct MessageUpdatePayload {
    id: String,
    channel_id: String,
    guild_id: Option<String>,
    content: Option<String>,
    author: Option<UserPayload>,
    edited_timestamp: Option<String>,
    old: Option<MessagePayload>,
    new: Option<MessagePayload>,
}

impl MessageUpdatePayload {
    fn from_parts(old: Option<Message>, new: Option<Message>, event: &MessageUpdateEvent) -> Self {
        let guild_id = event
            .guild_id
            .map(|g| g.get().to_string())
            .or_else(|| new.as_ref().and_then(|m| m.guild_id).map(|g| g.get().to_string()))
            .or_else(|| old.as_ref().and_then(|m| m.guild_id).map(|g| g.get().to_string()));

        let content = event.content.clone().or_else(|| new.as_ref().map(|m| m.content.clone()));

        let author = event
            .author
            .as_ref()
            .map(UserPayload::from)
            .or_else(|| new.as_ref().map(|m| UserPayload::from(&m.author)));

        let edited_timestamp =
            event.edited_timestamp.and_then(|ts| ts.to_rfc3339()).or_else(|| {
                new.as_ref().and_then(|m| m.edited_timestamp.and_then(|ts| ts.to_rfc3339()))
            });

        Self {
            id: event.id.get().to_string(),
            channel_id: event.channel_id.get().to_string(),
            guild_id,
            content,
            author,
            edited_timestamp,
            old: old.as_ref().map(MessagePayload::from),
            new: new.as_ref().map(MessagePayload::from),
        }
    }
}

#[derive(Serialize)]
struct MessageDeletePayload {
    id: String,
    channel_id: String,
    guild_id: Option<String>,
}

#[derive(Serialize)]
struct MessageDeleteBulkPayload {
    ids: Vec<String>,
    channel_id: String,
    guild_id: Option<String>,
}

#[derive(Serialize)]
struct ReadyPayload {
    user: UserPayload,
    guild_ids: Vec<String>,
}

#[derive(Serialize)]
struct InteractionCreatePayload {
    interaction_id: String,
    interaction_token: String,
    application_id: String,
    guild_id: Option<String>,
    channel_id: Option<String>,
    user: UserPayload,
    command_name: String,
    data: serde_json::Value,
    locale: Option<String>,
    guild_locale: Option<String>,
}

impl From<&CommandInteraction> for InteractionCreatePayload {
    fn from(interaction: &CommandInteraction) -> Self {
        let data = serde_json::to_value(&interaction.data).unwrap_or_default();
        Self {
            interaction_id: interaction.id.get().to_string(),
            interaction_token: interaction.token.clone(),
            application_id: interaction.application_id.get().to_string(),
            guild_id: interaction.guild_id.map(|g| g.get().to_string()),
            channel_id: Some(interaction.channel_id.get().to_string()),
            user: UserPayload::from(&interaction.user),
            command_name: interaction.data.name.clone(),
            data,
            locale: Some(interaction.locale.clone()),
            guild_locale: interaction.guild_locale.clone(),
        }
    }
}

impl From<&Ready> for ReadyPayload {
    fn from(ready: &Ready) -> Self {
        Self {
            user: UserPayload {
                id: ready.user.id.get().to_string(),
                username: ready.user.name.clone(),
                discriminator: ready.user.discriminator.map(|d| d.get()),
                bot: ready.user.bot,
            },
            guild_ids: ready.guilds.iter().map(|g| g.id.get().to_string()).collect(),
        }
    }
}
