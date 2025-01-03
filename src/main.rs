use std::collections::HashSet;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use itertools::Itertools;
use minijinja::{context, Value};
use minijinja_autoreload::AutoReloader;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_http::services::ServeDir;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod app_error;

use app_error::AppError;

#[derive(Deserialize)]
struct MessageInput {
    id: String,
    sender_name: String,
    sender_title: Option<String>,
    media: Option<MessageMedia>,
    message: String,
}

#[derive(Clone, Serialize)]
struct Message {
    id: String,
    sender_name: String,
    sender_title: Option<String>,
    media: Option<MessageMedia>,
    message: String,
    decal_variant: u64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum MessageMedia {
    Image {
        path: String,
        width: u32,
        height: u32,
        thumbnail: Option<Thumbnail>,
    },
    YouTube {
        path: String,
        width: u32,
        height: u32,
        video_id: String,
    },
    YouTubeClip {
        path: String,
        width: u32,
        height: u32,
        video_id: String,
        clip_id: String,
        clipt: String,
    },
}

#[derive(Clone, Serialize, Deserialize)]
struct Thumbnail {
    path: String,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct TimelineVideo<'a> {
    title: &'a str,
    video_id: &'a str,
    timestamp_display: String,
    timestamp_rfc3339: String,
    blurb: String,
}

#[derive(Serialize)]
struct TimelineGroup<'a> {
    group_id: String,
    group_name: String,
    videos: Vec<TimelineVideo<'a>>,
}

#[derive(Serialize)]
struct TimelineLink {
    display_string: String,
    link_id: String,
}

#[derive(Serialize)]
struct TimelineLinksGroup {
    year: i32,
    links: Vec<TimelineLink>,
}

#[derive(Deserialize)]
struct VideoInfo {
    id: String,
    title: String,
    #[serde(with = "time::serde::timestamp")]
    release_timestamp: OffsetDateTime,
    was_live: bool,
    playable_in_embed: bool,
    availability: String,
}

#[derive(Clone)]
struct AppState {
    template_engine: Arc<AutoReloader>,
    videos_data: &'static [VideoInfo],
}

impl FromRef<AppState> for Arc<AutoReloader> {
    fn from_ref(input: &AppState) -> Self {
        input.template_engine.clone()
    }
}

impl FromRef<AppState> for &'static [VideoInfo] {
    fn from_ref(input: &AppState) -> Self {
        input.videos_data
    }
}

const TEMPLATE_PATH: &str = "templates";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = std::env::args().collect::<Vec<_>>();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();

    let template_engine = AutoReloader::new(|notifier| {
        let mut env = minijinja::Environment::new();
        #[cfg(deploy_env = "dev")]
        {
            env.set_loader(minijinja::path_loader(TEMPLATE_PATH));
            notifier.set_fast_reload(true);
            notifier.watch_path(TEMPLATE_PATH, true);
        }
        // Load all templates
        #[cfg(deploy_env = "prod")]
        {
            let path_prefix = TEMPLATE_PATH.to_string() + "/";
            for file_path in fs_extra::dir::get_dir_content(TEMPLATE_PATH)
                .expect("Could not load templates directory")
                .files
            {
                let file_path = file_path.replace('\\', "/");
                let contents =
                    fs::read_to_string(&file_path).expect("Could not read template file");
                let name = file_path
                    .strip_prefix(&path_prefix)
                    .ok_or_else(|| {
                        anyhow!(
                            "File path \"{}\" is missing expected prefix \"{}\"",
                            file_path,
                            TEMPLATE_PATH
                        )
                    })
                    .unwrap();
                env.add_template_owned(name.to_string(), contents)
                    .with_context(|| format!("Template \"{}\" is invalid", file_path))
                    .unwrap();
            }
        }
        return Ok(env);
    });
    // Trigger initial environment creation to load templates
    #[cfg(deploy_env = "prod")]
    {
        template_engine
            .acquire_env()
            .context("Could not preload templates")?;
    }
    let mut videos_data = {
        let videos_json_file = std::fs::File::open("bae-videos.json")?;
        serde_json::from_reader::<_, Vec<VideoInfo>>(std::io::BufReader::new(videos_json_file))?
    };
    videos_data.sort_by_key(|v| v.release_timestamp);
    let videos_data: &'static [VideoInfo] = videos_data.leak();
    let file_service = ServeDir::new("public").precompressed_br();
    let mut app = Router::new()
        .route("/", get(home))
        .route("/index.html", get(home))
        .route("/messages", get(messages))
        .route("/messages.html", get(messages))
        .route("/timeline", get(timeline))
        .route("/timeline.html", get(timeline))
        .route("/credits", get(credits))
        .route("/credits.html", get(credits))
        .nest_service("/public", file_service)
        .fallback(not_found)
        .with_state(AppState {
            template_engine: Arc::new(template_engine),
            videos_data,
        });

    match args.get(1) {
        Some(a) if a == "--build" => build_static(&mut app, "./dist").await,
        _ => {
            let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            tracing::info!(
                "Starting server v{} listening on http://{}",
                env!("CARGO_PKG_VERSION"),
                &addr
            );

            axum::serve(listener, app).await.map_err(|e| anyhow!(e))
        }
    }
}

async fn home(State(template_engine): State<Arc<AutoReloader>>) -> Result<Html<String>, AppError> {
    let env = template_engine.acquire_env()?;
    let ctx = context! {};

    Ok(Html(env.get_template("home.html")?.render(ctx)?))
}

async fn credits(
    State(template_engine): State<Arc<AutoReloader>>,
) -> Result<Html<String>, AppError> {
    let env = template_engine.acquire_env()?;
    let ctx = context! {};

    Ok(Html(env.get_template("credits.html")?.render(ctx)?))
}

async fn messages(
    State(template_engine): State<Arc<AutoReloader>>,
) -> Result<Html<String>, AppError> {
    let messages_input_file = fs::File::open("data/messages.json")?;
    let mut messages_input = serde_json::from_reader::<_, serde_json::Value>(
        std::io::BufReader::new(messages_input_file),
    )?;
    let mut charset_sans = HashSet::new();
    let messages = messages_input["messages"]
        .as_array_mut()
        .ok_or(anyhow!(
            "Could not read messages field in messages.json as array"
        ))?
        .iter_mut()
        .map(|v| {
            let message_input = serde_json::from_value::<MessageInput>(v.take())?;
            let sender_name = message_input.sender_name;
            let decal_variant = if sender_name == "Mikururun" {
                10
            } else {
                let mut bytes = [0u8; 16];
                hex::decode_to_slice(&message_input.id, &mut bytes)?;

                (u128::from_be_bytes(bytes) % 5) as u64
            };
            charset_sans.extend(sender_name.chars());
            charset_sans.extend(message_input.message.chars());

            Ok(Value::from_serialize(Message {
                id: message_input.id,
                sender_name,
                sender_title: message_input.sender_title,
                media: message_input.media,
                message: message_input.message,
                decal_variant,
            }))
        })
        .try_collect::<_, Value, AppError>()?;
    let mut charset_sans_jp = String::new();
    let mut charset_sans_kr = String::new();
    for c in charset_sans {
        match c {
            // Hangul syllables block
            '\u{ac00}'..='\u{d7af}' => charset_sans_kr.push(c),
            // Hiragana, Katakana, CJK ideographs, halfwidth & fullwidth
            '\u{3000}'..='\u{9fff}' | '\u{ff00}'..='\u{ffef}' => charset_sans_jp.push(c),
            // Mathematical operators (for kaomoji)
            '\u{2200}'..='\u{22ff}' => charset_sans_jp.push(c),
            _ => {}
        }
    }
    let env = template_engine.acquire_env()?;
    let ctx = context! {messages, charset_sans_jp, charset_sans_kr};
    return Ok(Html(env.get_template("messages.html")?.render(ctx)?));
}

async fn timeline(
    State(template_engine): State<Arc<AutoReloader>>,
) -> Result<Html<String>, AppError> {
    let timeline_input_file = fs::File::open("data/timeline.json")?;
    let timeline_input = serde_json::from_reader::<_, serde_json::Value>(std::io::BufReader::new(
        timeline_input_file,
    ))?;
    let events = timeline_input["events"]
        .as_array()
        .ok_or(anyhow!(
            "Could not read events field in timeline.json as array"
        ))?
        .iter()
        .map(Value::from_serialize)
        .collect::<Value>();

    let env = template_engine.acquire_env()?;
    let ctx = context! {events};
    Ok(Html(env.get_template("timeline.html")?.render(ctx)?))
}

async fn not_found(
    State(template_engine): State<Arc<AutoReloader>>,
) -> Result<(StatusCode, Html<String>), AppError> {
    let env = template_engine.acquire_env()?;
    let ctx = context! {};

    Ok((
        StatusCode::NOT_FOUND,
        Html(env.get_template("404.html")?.render(ctx)?),
    ))
}

async fn build_static(
    router: &mut axum::routing::Router,
    output_dir: &str,
) -> Result<(), anyhow::Error> {
    use http_body_util::BodyExt;
    use std::fs;
    use std::path::Path;
    use tower_service::Service;

    let output_dir_path = Path::new(output_dir);
    if output_dir_path
        .try_exists()
        .context("Could not verify if output dir exists")?
    {
        loop {
            let mut input = String::new();
            println!(
                "Output directory \"{}\" exists. Overwrite directory? Y/N",
                output_dir
            );
            std::io::stdin()
                .read_line(&mut input)
                .context("Could not read input for overwrite confirmation")?;
            match input.as_str().trim_end() {
                "Y" | "y" => {
                    fs::remove_dir_all(output_dir).context("Could not clean output directory")?;
                    break;
                }
                "N" | "n" => break,
                _ => {}
            }
        }
    }
    fs::create_dir_all(output_dir).context("Could not create output directory")?;

    let copy_options = fs_extra::dir::CopyOptions::new();
    fs_extra::dir::copy("./public", output_dir, &copy_options)
        .context("Could not copy static assets directory")?;

    let page_paths = ["/", "/messages", "/credits", "/404"];
    for path in page_paths {
        let request = http::Request::get(path)
            .body(http_body_util::Empty::new())
            .with_context(|| format!("Could not create request for {}", path))?;
        let response = router
            .call(request)
            .await
            .with_context(|| format!("Could not get response for {}", path))?;
        let content_type = response
            .headers()
            .get(http::header::CONTENT_TYPE)
            .ok_or_else(|| anyhow!("No Content-Type for response {}", path))
            .and_then(|v| {
                v.to_str()
                    .with_context(|| format!("Content-Type for {} is not valid text", path))
            })?;
        if !content_type.contains("text/html") {
            return Err(anyhow!(
                "Unexpected Content-Type {:?} for response {}",
                content_type,
                path
            ));
        }
        let response_bytes = response
            .into_body()
            .collect()
            .await
            .map(|x| x.to_bytes())
            .with_context(|| format!("Could not read body for response {}", path))?;
        let output_filepath = if path == "/" {
            "index.html".to_owned()
        } else {
            // Want to create relative paths for the output, so remove the leading slash
            path[1..].to_owned() + ".html"
        };
        let output_fullpath = output_dir_path.join(output_filepath);
        if let Some(output_parent_dir) = output_fullpath.parent() {
            if !output_parent_dir.try_exists().with_context(|| {
                format!(
                    "Could not verify if directory for {:?} exists",
                    output_fullpath
                )
            })? {
                fs::create_dir_all(output_parent_dir).with_context(|| {
                    format!("Could not create directory for {:?}", output_fullpath)
                })?;
            }
        }
        fs::write(&output_fullpath, response_bytes)
            .with_context(|| format!("Could not write output file {:?}", output_fullpath))?;
    }

    Ok(())
}
