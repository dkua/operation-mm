use std::iter;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::anyhow;
use axum::extract::{FromRef, State};
use axum::response::Html;
use axum::routing::get;
use axum::Router;
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

#[derive(Clone, Serialize)]
struct Message {
    sender_name: String,
    video_id: Option<String>,
    message: String,
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
const NAME_LIST: [&str; 21] = [
    "AshScar",
    "saltedbread",
    "reki",
    "MurphLAZ3R",
    "Zyrob",
    "Alphaetus",
    "xing255",
    "TheRocki",
    "やよい軒",
    "TensuTensu",
    "Avros",
    "Vayne Darkness",
    "taco tom",
    "Kagecherou",
    "alkusanagi",
    "WallyWW",
    "WolkenKatz",
    "Avros",
    "PuffyOwlGod",
    "DiaGuy",
    "Trildar",
];

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();

    let template_engine = AutoReloader::new(|notifier| {
        let mut env = minijinja::Environment::new();
        env.set_loader(minijinja::path_loader(TEMPLATE_PATH));
        #[cfg(deploy_env = "dev")]
        {
            notifier.set_fast_reload(true);
            notifier.watch_path(TEMPLATE_PATH, true);
        }
        return Ok(env);
    });
    let mut videos_data = {
        let videos_json_file = std::fs::File::open("bae-videos.json")?;
        serde_json::from_reader::<_, Vec<VideoInfo>>(std::io::BufReader::new(videos_json_file))?
    };
    videos_data.sort_by_key(|v| v.release_timestamp);
    let videos_data: &'static [VideoInfo] = videos_data.leak();
    let file_service = ServeDir::new("public").precompressed_br();
    let app = Router::new()
        .route("/", get(home))
        .with_state(AppState {
            template_engine: Arc::new(template_engine),
            videos_data,
        })
        .nest_service("/public", file_service);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(
        "Starting server v{} listening on http://{}",
        env!("CARGO_PKG_VERSION"),
        &addr
    );

    axum::serve(listener, app).await
}

async fn home(
    State(template_engine): State<Arc<AutoReloader>>,
    State(videos_data): State<&[VideoInfo]>,
) -> Result<Html<String>, AppError> {
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(80085);
    let lorem_sentences = "Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis"
        .split('.')
        .collect::<Vec<_>>();
    let mut video_ids_iter = videos_data
        .iter()
        .filter_map(|video| {
            if video.playable_in_embed && video.availability == "public" {
                Some(&video.id)
            } else {
                None
            }
        })
        .peekable();
    let messages = iter::from_fn(|| {
        video_ids_iter.peek()?;

        let mut message = lorem_sentences[0..rng.gen_range(1..=lorem_sentences.len())].join(".");
        message.push('.');
        let video_id = if rng.gen_bool(0.8) {
            Some(video_ids_iter.next().unwrap().to_owned())
        } else {
            None
        };
        return Some(Value::from_serialize(Message {
            sender_name: NAME_LIST[rng.gen_range(0..NAME_LIST.len())].to_owned(),
            video_id,
            message,
        }));
    })
    .collect::<Value>();
    let env = template_engine.acquire_env()?;
    let ctx = context! {messages};
    return Ok(Html(env.get_template("home.html")?.render(ctx)?));
}
