use std::{collections::HashMap, path::PathBuf, rc::Rc, sync::Arc, thread};

use deno_core::{
    FastString, JsRuntime, ModuleName, PollEventLoopOptions, RuntimeOptions,
    error::AnyError,
    serde_v8,
    v8::{self, Global},
};
use serde_json::Value;
use serenity::http::Http;
use tokio::runtime::Builder;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

use crate::{deployments::Deployment, ops};

pub struct BotRuntime {
    sender: mpsc::UnboundedSender<RuntimeCommand>,
}

struct JsRuntimeState {
    runtime: JsRuntime,
    dispatch_fn: Option<Global<v8::Function>>,
}

impl Drop for JsRuntimeState {
    fn drop(&mut self) {
        // V8 requires the isolate to be entered before resetting persistent handles.
        if let Some(dispatch_fn) = self.dispatch_fn.take() {
            let isolate = self.runtime.v8_isolate();
            let _isolate_guard = IsolateEnterGuard::new(isolate);
            // Create a handle scope so V8 is happy when cleaning up persistent handles.
            let scope = v8::HandleScope::new(isolate);
            drop(dispatch_fn);
            // Explicitly drop the scope before leaving the isolate.
            drop(scope);
        }
    }
}

struct IsolateEnterGuard {
    isolate: *mut v8::OwnedIsolate,
}

impl IsolateEnterGuard {
    fn new(isolate: &mut v8::OwnedIsolate) -> Self {
        // Enter the isolate so subsequent scopes are tied to it.
        unsafe { isolate.enter() };
        Self { isolate }
    }
}

impl Drop for IsolateEnterGuard {
    fn drop(&mut self) {
        // SAFETY: isolate lives for the guard's lifetime, we only store the raw pointer.
        let isolate = unsafe { &mut *self.isolate };
        unsafe { isolate.exit() };
    }
}

fn enter_isolate(runtime: &mut JsRuntime) -> IsolateEnterGuard {
    let isolate = runtime.v8_isolate();
    IsolateEnterGuard::new(isolate)
}

enum RuntimeCommand {
    Initialize {
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    LoadScript {
        path: PathBuf,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    LoadGuildDeployment {
        deployment: Deployment,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    Dispatch {
        event: String,
        guild_id: Option<String>,
        payload: Value,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
}

struct RuntimeThreadState {
    default_runtime: JsRuntimeState,
    guild_runtimes: HashMap<String, JsRuntimeState>,
    http: Arc<Http>,
}

impl BotRuntime {
    pub fn new(http: Arc<Http>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        thread::spawn(move || runtime_thread(receiver, http));
        Self { sender }
    }

    pub async fn initialize(&self) -> Result<(), AnyError> {
        self.request(|respond_to| RuntimeCommand::Initialize { respond_to }).await
    }

    pub async fn load_user_script(&self, path: impl Into<PathBuf>) -> Result<(), AnyError> {
        let path = path.into();
        self.request(|respond_to| RuntimeCommand::LoadScript { path, respond_to }).await
    }

    pub async fn deploy_guild_script(&self, deployment: Deployment) -> Result<(), AnyError> {
        self.request(|respond_to| RuntimeCommand::LoadGuildDeployment {
            deployment: deployment.clone(),
            respond_to,
        })
        .await
    }

    pub async fn dispatch_js_event(
        &self,
        event: &str,
        guild_id: Option<String>,
        payload: Value,
    ) -> Result<(), AnyError> {
        let event = event.to_string();
        self.request(|respond_to| RuntimeCommand::Dispatch { event, guild_id, payload, respond_to })
            .await
    }

    async fn request<F>(&self, f: F) -> Result<(), AnyError>
    where
        F: FnOnce(oneshot::Sender<Result<(), AnyError>>) -> RuntimeCommand,
    {
        let (tx, rx) = oneshot::channel();
        let command = f(tx);
        self.sender.send(command).map_err(|_| AnyError::msg("runtime thread is unavailable"))?;

        rx.await.map_err(|_| AnyError::msg("runtime thread stopped"))?
    }
}

fn runtime_thread(mut receiver: mpsc::UnboundedReceiver<RuntimeCommand>, http: Arc<Http>) {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build single-thread runtime");

    runtime.block_on(async move {
        let mut state = RuntimeThreadState {
            default_runtime: new_js_runtime(http.clone()),
            guild_runtimes: HashMap::new(),
            http,
        };

        info!("oakmoss JS runtime thread started");
        while let Some(command) = receiver.recv().await {
            match command {
                RuntimeCommand::Initialize { respond_to } => {
                    let result = initialize_runtime(&mut state.default_runtime).await;
                    if let Err(err) = &result {
                        error!("runtime init error: {:?}", err);
                    }
                    let _ = respond_to.send(result);
                }
                RuntimeCommand::LoadScript { path, respond_to } => {
                    let result = load_script_from_path(&mut state.default_runtime, path).await;
                    if let Err(err) = &result {
                        error!("script load error: {:?}", err);
                    }
                    let _ = respond_to.send(result);
                }
                RuntimeCommand::LoadGuildDeployment { deployment, respond_to } => {
                    let result = load_guild_deployment(&mut state, deployment).await;
                    if let Err(err) = &result {
                        error!("guild deployment load error: {:?}", err);
                    }
                    let _ = respond_to.send(result);
                }
                RuntimeCommand::Dispatch { event, guild_id, payload, respond_to } => {
                    let result = dispatch_event(&mut state, event, guild_id, payload).await;
                    if let Err(err) = &result {
                        error!("dispatch error: {:?}", err);
                    }
                    let _ = respond_to.send(result);
                }
            };
        }
    });
}

fn new_js_runtime(http: Arc<Http>) -> JsRuntimeState {
    JsRuntimeState {
        runtime: JsRuntime::new(RuntimeOptions {
            extensions: vec![ops::extension(http)],
            extension_transpiler: Some(Rc::new(|specifier, source| {
                match crate::transpile::transpile_if_typescript(&specifier, source.as_str())? {
                    Some(result) => Ok((result.code, result.source_map)),
                    None => Ok((source, None)),
                }
            })),
            ..Default::default()
        }),
        dispatch_fn: None,
    }
}

async fn initialize_runtime(js_state: &mut JsRuntimeState) -> Result<(), AnyError> {
    let _isolate_guard = enter_isolate(&mut js_state.runtime);
    js_state.runtime.execute_script("oakmoss:bootstrap", RUNTIME_PRELUDE)?;
    js_state.runtime.run_event_loop(PollEventLoopOptions::default()).await?;
    js_state.dispatch_fn = Some(extract_dispatch_fn(&mut js_state.runtime)?);
    info!("oakmoss JS runtime initialized");
    Ok(())
}

async fn load_script_from_path(
    js_state: &mut JsRuntimeState,
    path: PathBuf,
) -> Result<(), AnyError> {
    let source = tokio::fs::read_to_string(&path).await?;
    let name = path.to_string_lossy().to_string();
    load_script_source(&mut js_state.runtime, ModuleName::from(name.clone()), source, name).await
}

async fn load_script_source(
    js_runtime: &mut JsRuntime,
    module_name: ModuleName,
    source: String,
    name: String,
) -> Result<(), AnyError> {
    info!(target: "oakmoss:runtime", module = module_name.as_str(), "executing module source");
    let _isolate_guard = enter_isolate(js_runtime);
    let code = match crate::transpile::transpile_if_typescript(&module_name, &source)? {
        Some(result) => result.code,
        None => FastString::from(source),
    };

    js_runtime.execute_script(name, code)?;
    js_runtime.run_event_loop(PollEventLoopOptions::default()).await?;
    info!(target: "oakmoss:runtime", module = module_name.as_str(), "module executed");
    Ok(())
}

async fn load_guild_deployment(
    state: &mut RuntimeThreadState,
    deployment: Deployment,
) -> Result<(), AnyError> {
    // Drop any existing isolate for this guild before spinning up a fresh one.
    if let Some(old_runtime) = state.guild_runtimes.remove(&deployment.guild_id) {
        drop(old_runtime);
    }

    info!(
        target: "oakmoss:runtime",
        guild_id = deployment.guild_id,
        "creating guild runtime"
    );
    let mut runtime = new_js_runtime(state.http.clone());
    info!(
        target: "oakmoss:runtime",
        guild_id = deployment.guild_id,
        "initializing guild runtime prelude"
    );
    initialize_runtime(&mut runtime).await?;
    info!(
        target: "oakmoss:runtime",
        guild_id = deployment.guild_id,
        path = SDK_BUNDLE_PATH,
        "loading sdk bundle"
    );
    load_script_from_path(&mut runtime, PathBuf::from(SDK_BUNDLE_PATH)).await?;

    let module_name = ModuleName::from(deployment.language.module_name(&deployment.guild_id));
    let script_name = module_name.as_str().to_string();
    info!(
        target: "oakmoss:runtime",
        guild_id = deployment.guild_id,
        script = script_name,
        "loading guild script source"
    );
    load_script_source(&mut runtime.runtime, module_name, deployment.source.clone(), script_name)
        .await?;

    // Ensure dispatch function is refreshed after loading user script.
    info!(
        target: "oakmoss:runtime",
        guild_id = deployment.guild_id,
        "extracting dispatch function"
    );
    runtime.dispatch_fn = Some(extract_dispatch_fn(&mut runtime.runtime)?);
    info!(
        target: "oakmoss:runtime",
        guild_id = deployment.guild_id,
        "dispatch function extracted"
    );
    state.guild_runtimes.insert(deployment.guild_id.clone(), runtime);
    info!(
        target: "oakmoss:runtime",
        guild_id = deployment.guild_id,
        "loaded guild deployment into isolate"
    );
    Ok(())
}

async fn dispatch_event(
    state: &mut RuntimeThreadState,
    event: String,
    guild_id: Option<String>,
    payload: Value,
) -> Result<(), AnyError> {
    if let Some(guild_id) = guild_id {
        if let Some(runtime) = state.guild_runtimes.get_mut(&guild_id) {
            dispatch_into_runtime(runtime, event, payload).await
        } else {
            dispatch_into_runtime(&mut state.default_runtime, event, payload).await
        }
    } else {
        // Broadcast ready-style events to all runtimes, including the default one.
        let mut result =
            dispatch_into_runtime(&mut state.default_runtime, event.clone(), payload.clone()).await;

        for runtime in state.guild_runtimes.values_mut() {
            if let Err(err) = dispatch_into_runtime(runtime, event.clone(), payload.clone()).await {
                error!("dispatch error in guild runtime: {:?}", err);
                result = Err(err);
            }
        }

        result
    }
}

async fn dispatch_into_runtime(
    js_state: &mut JsRuntimeState,
    event: String,
    payload: Value,
) -> Result<(), AnyError> {
    let dispatch_fn = js_state
        .dispatch_fn
        .as_ref()
        .ok_or_else(|| AnyError::msg("dispatch function not initialized"))?;

    let _isolate_guard = enter_isolate(&mut js_state.runtime);
    {
        let context = js_state.runtime.main_context();
        let isolate = js_state.runtime.v8_isolate();
        v8::scope_with_context!(scope, isolate, &context);
        let scope = scope;
        let context = v8::Local::new(scope, &context);
        let dispatch_fn = v8::Local::new(scope, dispatch_fn);
        let this = context.global(scope);
        let event_value = serde_v8::to_v8(scope, &event)?;
        let payload_value = serde_v8::to_v8(scope, &payload)?;

        dispatch_fn
            .call(scope, this.into(), &[event_value, payload_value])
            .ok_or_else(|| AnyError::msg("dispatch call failed"))?;
    }

    js_state.runtime.run_event_loop(PollEventLoopOptions::default()).await.map_err(AnyError::from)
}

fn extract_dispatch_fn(runtime: &mut JsRuntime) -> Result<Global<v8::Function>, AnyError> {
    let context = runtime.main_context();
    let isolate = runtime.v8_isolate();
    let _isolate_guard = IsolateEnterGuard::new(isolate);
    v8::scope_with_context!(scope, isolate, &context);
    let scope = scope;
    let context = v8::Local::new(scope, &context);
    let global = context.global(scope);
    let key = v8::String::new(scope, "__oakmossDispatch")
        .ok_or_else(|| AnyError::msg("failed to create dispatch name"))?;
    let value =
        global.get(scope, key.into()).ok_or_else(|| AnyError::msg("dispatch function missing"))?;
    let function = v8::Local::<v8::Function>::try_from(value)
        .map_err(|_| AnyError::msg("dispatch symbol is not a function"))?;
    Ok(Global::new(scope, function))
}

const RUNTIME_PRELUDE: &str = r#"
// Minimal runtime helpers exposed to user scripts.
const core = Deno.core;
globalThis.__oakmossHandlers = {};

globalThis.on = function on(event, handler) {
  if (!globalThis.__oakmossHandlers[event]) {
    globalThis.__oakmossHandlers[event] = [];
  }
  globalThis.__oakmossHandlers[event].push(handler);
};

globalThis.__oakmossDispatch = async function __oakmossDispatch(event, payload) {
  const handlers = globalThis.__oakmossHandlers[event] || [];
  for (const handler of handlers) {
    const context = {
      msg: payload,
      reply(message) {
        const options = normalizeReply(message, payload);
        if (options.interaction_id && options.token) {
          return core.ops.op_send_interaction_response(options);
        }
        return core.ops.op_send_message(options);
      },
    };
    await handler(context);
  }
};

globalThis.console = {
  log: (...args) => core.ops.op_log(args),
};

function normalizeReply(message, payload) {
  if (payload?.interaction_token) {
    return normalizeInteractionReply(message, payload);
  }

  const base = { channel_id: payload.channel_id };

  if (typeof message === "string") {
    return { ...base, message_id: payload.id, content: message };
  }

  if (message && typeof message === "object") {
    const normalized = { ...base, ...message };
    const explicitReplyTo = message.replyTo ?? message.reply_to;

    if (explicitReplyTo === null) {
      delete normalized.message_id;
    } else if (explicitReplyTo !== undefined) {
      normalized.message_id = explicitReplyTo;
    } else if (payload?.id) {
      normalized.message_id = payload.id;
    }

    delete normalized.replyTo;
    delete normalized.reply_to;
    return normalized;
  }

  return { ...base, message_id: payload.id, content: String(message) };
}

function normalizeInteractionReply(message, payload) {
  const base = {
    interaction_id: payload.interaction_id ?? payload.id,
    token: payload.interaction_token,
  };

  if (typeof message === "string") {
    return { ...base, content: message };
  }

  if (message && typeof message === "object") {
    const normalized = { ...base, ...message };
    if (message.ephemeral !== undefined) {
      normalized.ephemeral = message.ephemeral;
    }
    return normalized;
  }

  return { ...base, content: String(message) };
}
"#;

const SDK_BUNDLE_PATH: &str = "dist/sdk-bundle.js";
