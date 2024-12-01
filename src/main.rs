use std::net::SocketAddr;
use std::sync::Arc;
use std::{env, fs, iter};

use anyhow::{anyhow, Context};
use axum::extract::{FromRef, MatchedPath, State};
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

#[derive(Clone, Serialize)]
struct Message {
    sender_name: String,
    video_id: Option<String>,
    message: String,
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
const NAME_LIST: [&str; 23] = [
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
    "buffybear",
    "mikururun",
    "Trildar",
];

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
        .route("/messages", get(messages))
        .route("/timeline", get(timeline))
        .route("/credits", get(credits))
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

async fn home(
    State(template_engine): State<Arc<AutoReloader>>,
    matched_path: MatchedPath,
) -> Result<Html<String>, AppError> {
    let env = template_engine.acquire_env()?;
    let base_url = env::var("DEPLOY_BASE_URL").ok();
    let ctx = context! { base_url };
    Ok(Html(env.get_template("home.html")?.render(ctx)?))
}

async fn credits(
    State(template_engine): State<Arc<AutoReloader>>,
    matched_path: MatchedPath,
) -> Result<Html<String>, AppError> {
    let env = template_engine.acquire_env()?;
    let base_url = env::var("DEPLOY_BASE_URL").ok();
    let ctx = context! { base_url };
    Ok(Html(env.get_template("credits.html")?.render(ctx)?))
}

async fn messages(
    State(template_engine): State<Arc<AutoReloader>>,
    State(videos_data): State<&[VideoInfo]>,
    matched_path: MatchedPath,
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

        let mut message = lorem_sentences[0..rng.gen_range(1..=5)].join(".");
        message.push('.');
        let video_id = if rng.gen_bool(0.2) {
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
    .take(2000)
    .collect::<Value>();
    let env = template_engine.acquire_env()?;
    let base_url = env::var("DEPLOY_BASE_URL").ok();
    let ctx = context! { messages => messages, base_url => base_url };
    return Ok(Html(env.get_template("messages.html")?.render(ctx)?));
}

async fn timeline(
    State(videos_data): State<&[VideoInfo]>,
    State(template_engine): State<Arc<AutoReloader>>,
    matched_path: MatchedPath,
) -> Result<Html<String>, AppError> {
    use time::macros::{format_description, time};

    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(80085);
    let lorem_sentences = "Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis"
        .split('.')
        .collect::<Vec<_>>();
    let grouped_videos = videos_data
        .iter()
        .filter(|video| video.playable_in_embed && video.availability == "public")
        .chunk_by(|video| {
            video
                .release_timestamp
                .replace_day(1)
                .unwrap()
                .replace_time(time!(00:00))
        })
        .into_iter()
        .map(|(key, chunk)| {
            Value::from_serialize(TimelineGroup {
                group_id: key.format(format_description!("g-[year]-[month]")).unwrap(),
                group_name: key
                    .format(format_description!("[month repr:long] [year]"))
                    .unwrap(),
                videos: chunk
                    .map(|video| {
                        let mut message =
                            lorem_sentences[0..rng.gen_range(1..=lorem_sentences.len())].join(".");
                        message.push('.');
                        TimelineVideo {
                            title: &video.title,
                            video_id: &video.id,
                            timestamp_display: video
                                .release_timestamp
                                .format(format_description!("[year]-[month]-[day]"))
                                .unwrap(),
                            timestamp_rfc3339: video
                                .release_timestamp
                                .format(&time::format_description::well_known::Rfc3339)
                                .unwrap(),
                            blurb: message,
                        }
                    })
                    .collect(),
            })
        })
        .collect::<Value>();
    let group_links = videos_data
        .iter()
        .filter(|video| video.playable_in_embed && video.availability == "public")
        .map(|video| video.release_timestamp)
        .chunk_by(|timestamp| timestamp.year())
        .into_iter()
        .map(|(year, chunk)| {
            Value::from_serialize(TimelineLinksGroup {
                year,
                links: chunk
                    .dedup_by(|a, b| a.month() == b.month())
                    .map(|timestamp| TimelineLink {
                        link_id: timestamp
                            .format(format_description!("g-[year]-[month]"))
                            .unwrap(),
                        display_string: timestamp
                            .format(format_description!("[month repr:short]"))
                            .unwrap(),
                    })
                    .collect(),
            })
        })
        .collect::<Value>();

    let env = template_engine.acquire_env()?;
    let base_url = env::var("DEPLOY_BASE_URL").ok();
    let ctx = context! { grouped_videos, group_links, base_url };
    Ok(Html(env.get_template("timeline.html")?.render(ctx)?))
}

async fn not_found(
    State(template_engine): State<Arc<AutoReloader>>,
) -> Result<(StatusCode, Html<String>), AppError> {
    let env = template_engine.acquire_env()?;
    let base_url = env::var("DEPLOY_BASE_URL").ok();
    let ctx = context! { base_url };

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

    let page_paths = ["/", "/messages", "/timeline", "/credits", "/404"];
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
