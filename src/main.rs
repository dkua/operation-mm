use std::iter;
use std::sync::Arc;

use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use minijinja::{context, Value};
use minijinja_autoreload::AutoReloader;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use tower_http::services::ServeDir;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Clone, Serialize)]
struct Message {
    sender_name: String,
    video_id: Option<String>,
    message: String,
}

const TEMPLATE_PATH: &str = "templates";
const NAME_LIST: [&str; 15] = [
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
    "Trildar",
];

#[tokio::main]
async fn main() {
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
    let file_service = ServeDir::new("public").precompressed_br();
    let app = Router::new()
        .route("/", get(home))
        .with_state(Arc::new(template_engine))
        .fallback_service(file_service);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Starting server");
    axum::serve(listener, app).await.unwrap();
}

async fn home(State(template_engine): State<Arc<AutoReloader>>) -> Html<String> {
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(80085);
    let lorem_sentences = "Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis"
        .split('.')
        .collect::<Vec<_>>();
    let video_ids = iter::repeat("oCOGTtxq24k").take(750);
    let messages = video_ids
        .map(|id| {
            let mut message =
                lorem_sentences[0..rng.gen_range(1..=lorem_sentences.len())].join(".");
            message.push('.');
            let video_id = if rng.gen_bool(0.8) {
                Some(id.to_owned())
            } else {
                None
            };
            Value::from_serialize(Message {
                sender_name: NAME_LIST[rng.gen_range(0..NAME_LIST.len())].to_owned(),
                video_id,
                message,
            })
        })
        .collect::<Value>();
    let env = template_engine.acquire_env().unwrap();
    let ctx = context! {messages};
    return Html(env.get_template("home.html").unwrap().render(ctx).unwrap());
}
