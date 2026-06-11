use std::path::{Path, PathBuf};

use tokio::io::{AsyncBufReadExt, BufReader};

use crate::app::context::AppContext;
use crate::db::{EventFilter, EventRecord};
use crate::tui::data::{ProxyStream, TuiProxyLogRow};

const EVENT_LIMIT: i64 = 200;
const PROXY_LOG_LIMIT: usize = 200;

#[derive(Debug, Clone, Default)]
pub struct TuiLogs {
    pub events: Vec<TuiEventLogRow>,
    pub proxy: Vec<TuiProxyLogRow>,
}

impl TuiLogs {
    pub async fn load(context: &AppContext) -> crate::app::Result<Self> {
        let (events, proxy) = tokio::join!(load_events(context), load_proxy_logs(context));
        Ok(Self {
            events: events?,
            proxy: proxy?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiEventLogRow {
    pub id: i64,
    pub time: String,
    pub level: String,
    pub source: String,
    pub kind: String,
    pub message: String,
}

impl From<EventRecord> for TuiEventLogRow {
    fn from(value: EventRecord) -> Self {
        Self {
            id: value.id,
            time: value.created_at,
            level: value.level,
            source: value.source,
            kind: value.kind,
            message: value.message,
        }
    }
}

async fn load_events(context: &AppContext) -> crate::app::Result<Vec<TuiEventLogRow>> {
    let filter = EventFilter {
        limit: EVENT_LIMIT,
        ..EventFilter::default()
    };
    Ok(context
        .db
        .list_events(&filter)
        .await?
        .into_iter()
        .map(TuiEventLogRow::from)
        .collect())
}

async fn load_proxy_logs(context: &AppContext) -> crate::app::Result<Vec<TuiProxyLogRow>> {
    let Some(session) = context.db.get_latest_runtime_session().await? else {
        return Ok(Vec::new());
    };

    let mut files = Vec::new();
    let runtime_dir = &context.runtime_paths.runtime_dir;
    let id = session.id;
    push_existing(
        &mut files,
        "xray",
        ProxyStream::Stdout,
        runtime_dir.join(format!("session-{id}.out.log")),
    );
    push_existing(
        &mut files,
        "xray",
        ProxyStream::Stderr,
        runtime_dir.join(format!("session-{id}.err.log")),
    );
    push_existing(
        &mut files,
        "sing-box",
        ProxyStream::Stdout,
        runtime_dir.join(format!("session-{id}.singbox.out.log")),
    );
    push_existing(
        &mut files,
        "sing-box",
        ProxyStream::Stderr,
        runtime_dir.join(format!("session-{id}.singbox.err.log")),
    );

    let mut rows = Vec::new();
    for file in files {
        let lines = tail_lines(&file.path, PROXY_LOG_LIMIT).await?;
        rows.extend(
            lines
                .into_iter()
                .map(|line| TuiProxyLogRow::parse(&file.engine, file.stream, &line)),
        );
    }

    if rows.len() > PROXY_LOG_LIMIT {
        let start = rows.len() - PROXY_LOG_LIMIT;
        rows.drain(0..start);
    }
    Ok(rows)
}

struct LogFile {
    engine: String,
    stream: ProxyStream,
    path: PathBuf,
}

fn push_existing(files: &mut Vec<LogFile>, engine: &str, stream: ProxyStream, path: PathBuf) {
    if path.exists() {
        files.push(LogFile {
            engine: engine.to_string(),
            stream,
            path,
        });
    }
}

async fn tail_lines(path: &Path, max_lines: usize) -> crate::app::Result<Vec<String>> {
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };

    let mut reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut buffer = String::new();
    loop {
        buffer.clear();
        let read = reader.read_line(&mut buffer).await?;
        if read == 0 {
            break;
        }
        lines.push(buffer.trim_end_matches(['\n', '\r']).to_string());
    }

    if lines.len() > max_lines {
        let start = lines.len() - max_lines;
        lines.drain(0..start);
    }
    Ok(lines)
}
