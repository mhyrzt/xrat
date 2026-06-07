use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

use crate::app::commands::output::{self, Align, Cell, Column};
use crate::app::context::AppContext;
use crate::cli::{ListFormat, LogLevel, LogSource, LogsArgs, LogsClearArgs, LogsCommand};
use crate::db::{EventFilter, EventRecord};

pub async fn run(context: &AppContext, args: &LogsArgs) -> crate::app::Result<()> {
    if let Some(LogsCommand::Clear(clear_args)) = &args.command {
        return clear(context, clear_args).await;
    }

    let feeds = Feeds::resolve(context, args.source).await?;
    let filter = EventFilter {
        source: None,
        levels: level_threshold(args.level),
        limit: args.lines as i64,
    };

    if args.follow {
        follow(context, args, &feeds, &filter).await
    } else {
        one_time(context, args, &feeds, &filter).await
    }
}

/// Permanently delete persisted app/runtime events. This is a database clear,
/// distinct from the TUI's view-only log clear. Engine log files are not
/// touched; they are tied to runtime sessions and rotate on their own.
async fn clear(context: &AppContext, args: &LogsClearArgs) -> crate::app::Result<()> {
    let probe = EventFilter {
        limit: i64::MAX,
        ..EventFilter::default()
    };
    let pending = context.db.list_events(&probe).await?.len();
    if pending == 0 {
        println!(
            "{}",
            output::notice("No persisted events to clear.", output::color_enabled())
        );
        return Ok(());
    }

    if !args.yes && !output::confirm("Permanently delete all persisted app events?")? {
        println!("{}", output::notice("Aborted.", output::color_enabled()));
        return Ok(());
    }

    let cleared = context.db.clear_events().await?;
    println!(
        "{}",
        output::success(
            format!("Cleared {cleared} event(s)."),
            output::color_enabled()
        )
    );
    Ok(())
}

async fn one_time(
    context: &AppContext,
    args: &LogsArgs,
    feeds: &Feeds,
    filter: &EventFilter,
) -> crate::app::Result<()> {
    let events = if feeds.events {
        let mut events = context.db.list_events(filter).await?;
        events.reverse(); // list_events returns newest-first; show oldest-first
        events
    } else {
        Vec::new()
    };

    if feeds.events && matches!(args.format, ListFormat::Json | ListFormat::Tsv) {
        println!("{}", render_events(&events, args.format)?);
        return Ok(());
    }

    if feeds.events {
        if events.is_empty() {
            println!("{}", output::empty_message("No events recorded."));
        } else {
            println!("{}", render_events(&events, ListFormat::Table)?);
        }
    }

    for file in &feeds.files {
        let lines = tail_lines(&file.path, args.lines).await?;
        if lines.is_empty() {
            continue;
        }
        println!(
            "\n{}",
            output::format_list(
                &format!("{} ({})", file.label, file.path.display()),
                &lines,
                output::color_enabled()
            )
        );
    }

    Ok(())
}

async fn follow(
    context: &AppContext,
    args: &LogsArgs,
    feeds: &Feeds,
    filter: &EventFilter,
) -> crate::app::Result<()> {
    let color = output::color_enabled();

    let mut cursor = 0i64;
    if feeds.events {
        let mut initial = context.db.list_events(filter).await?;
        initial.reverse();
        for event in &initial {
            println!("{}", format_event_line(event, color));
            cursor = cursor.max(event.id);
        }
    }

    let mut offsets: Vec<u64> = Vec::with_capacity(feeds.files.len());
    for file in &feeds.files {
        let lines = tail_lines(&file.path, args.lines).await?;
        for line in &lines {
            println!("{}", format_file_line(&file.label, line, color));
        }
        offsets.push(current_len(&file.path));
    }

    let mut ticker = tokio::time::interval(Duration::from_millis(500));
    let follow_filter = EventFilter {
        limit: 0,
        ..filter.clone()
    };

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                return Ok(());
            }
            _ = ticker.tick() => {
                if feeds.events {
                    let new_events = context.db.events_after(cursor, &follow_filter).await?;
                    for event in &new_events {
                        println!("{}", format_event_line(event, color));
                        cursor = cursor.max(event.id);
                    }
                }

                for (index, file) in feeds.files.iter().enumerate() {
                    let offset = offsets[index];
                    let (lines, new_offset) = read_from(&file.path, offset).await?;
                    for line in lines {
                        println!("{}", format_file_line(&file.label, &line, color));
                    }
                    offsets[index] = new_offset;
                }
            }
        }
    }
}

struct Feeds {
    events: bool,
    files: Vec<LogFile>,
}

struct LogFile {
    label: String,
    path: PathBuf,
}

impl Feeds {
    async fn resolve(context: &AppContext, source: LogSource) -> crate::app::Result<Self> {
        let runtime_dir = &context.runtime_paths.runtime_dir;
        let events = matches!(source, LogSource::All | LogSource::App);

        let mut files = Vec::new();

        if matches!(source, LogSource::All | LogSource::Daemon) {
            push_existing(&mut files, "daemon", runtime_dir.join("daemon.log"));
        }

        if matches!(
            source,
            LogSource::All | LogSource::Xray | LogSource::Singbox
        ) && let Some(session) = context.db.get_latest_runtime_session().await?
        {
            let id = session.id;
            if matches!(source, LogSource::All | LogSource::Xray) {
                push_existing(
                    &mut files,
                    "xray",
                    runtime_dir.join(format!("session-{id}.out.log")),
                );
                push_existing(
                    &mut files,
                    "xray",
                    runtime_dir.join(format!("session-{id}.err.log")),
                );
            }
            if matches!(source, LogSource::All | LogSource::Singbox) {
                push_existing(
                    &mut files,
                    "singbox",
                    runtime_dir.join(format!("session-{id}.singbox.out.log")),
                );
                push_existing(
                    &mut files,
                    "singbox",
                    runtime_dir.join(format!("session-{id}.singbox.err.log")),
                );
            }
        }

        Ok(Self { events, files })
    }
}

fn push_existing(files: &mut Vec<LogFile>, label: &str, path: PathBuf) {
    if path.exists() {
        files.push(LogFile {
            label: label.to_string(),
            path,
        });
    }
}

fn level_threshold(level: Option<LogLevel>) -> Option<Vec<String>> {
    match level {
        None | Some(LogLevel::Info) => None,
        Some(LogLevel::Warn) => Some(vec!["warn".to_string(), "error".to_string()]),
        Some(LogLevel::Error) => Some(vec!["error".to_string()]),
    }
}

fn render_events(events: &[EventRecord], format: ListFormat) -> crate::app::Result<String> {
    match format {
        ListFormat::Json => Ok(serde_json::to_string_pretty(
            &events.iter().map(event_json).collect::<Vec<_>>(),
        )?),
        ListFormat::Tsv => {
            let mut lines = Vec::with_capacity(events.len() + 1);
            lines.push("time\tlevel\tsource\tkind\tconfig_id\tsession_id\tmessage".to_string());
            for event in events {
                lines.push(format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    event.created_at,
                    event.level,
                    event.source,
                    event.kind,
                    event.config_id.map(|id| id.to_string()).unwrap_or_default(),
                    event
                        .session_id
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                    event.message.replace(['\t', '\r', '\n'], " "),
                ));
            }
            Ok(lines.join("\n"))
        }
        ListFormat::Table => Ok(format_event_table(events)),
    }
}

const EVENT_TIME_WIDTH: usize = 19;
const EVENT_LEVEL_WIDTH: usize = 5;
const EVENT_SOURCE_WIDTH: usize = 12;
const EVENT_KIND_WIDTH: usize = 32;
const EVENT_MESSAGE_WIDTH: usize = 96;

fn format_event_table(events: &[EventRecord]) -> String {
    let columns = [
        Column {
            header: "TIME",
            align: Align::Left,
        },
        Column {
            header: "LEVEL",
            align: Align::Left,
        },
        Column {
            header: "SOURCE",
            align: Align::Left,
        },
        Column {
            header: "KIND",
            align: Align::Left,
        },
        Column {
            header: "MESSAGE",
            align: Align::Left,
        },
    ];
    let rows = events
        .iter()
        .map(|event| {
            vec![
                Cell::plain(format!(
                    "{:<EVENT_TIME_WIDTH$}",
                    output::truncate(&event.created_at, EVENT_TIME_WIDTH)
                )),
                Cell::styled(
                    format!(
                        "{:<EVENT_LEVEL_WIDTH$}",
                        output::truncate(&event.level, EVENT_LEVEL_WIDTH)
                    ),
                    level_style(&event.level),
                ),
                Cell::plain(format!(
                    "{:<EVENT_SOURCE_WIDTH$}",
                    output::truncate(&event.source, EVENT_SOURCE_WIDTH)
                )),
                Cell::plain(format!(
                    "{:<EVENT_KIND_WIDTH$}",
                    output::truncate(&event.kind, EVENT_KIND_WIDTH)
                )),
                Cell::plain(output::truncate(&event.message, EVENT_MESSAGE_WIDTH)),
            ]
        })
        .collect::<Vec<_>>();
    output::format_table(&columns, &rows, output::color_enabled())
}

fn event_json(event: &EventRecord) -> serde_json::Value {
    serde_json::json!({
        "id": event.id,
        "level": event.level,
        "source": event.source,
        "kind": event.kind,
        "config_id": event.config_id,
        "session_id": event.session_id,
        "message": event.message,
        "detail": event.detail,
        "created_at": event.created_at,
    })
}

fn level_style(level: &str) -> output::Style {
    match level {
        "error" => output::Style::Red,
        "warn" => output::Style::Yellow,
        _ => output::Style::Cyan,
    }
}

fn format_event_line(event: &EventRecord, color: bool) -> String {
    let level_text = format!(
        "{:<EVENT_LEVEL_WIDTH$}",
        output::truncate(&event.level, EVENT_LEVEL_WIDTH)
    );
    let level = output::style_text(&level_text, level_style(&event.level), color);
    let time = format!(
        "{:<EVENT_TIME_WIDTH$}",
        output::truncate(&event.created_at, EVENT_TIME_WIDTH)
    );
    let source = format!(
        "{:<EVENT_SOURCE_WIDTH$}",
        output::truncate(&event.source, EVENT_SOURCE_WIDTH)
    );
    let kind = format!(
        "{:<EVENT_KIND_WIDTH$}",
        output::truncate(&event.kind, EVENT_KIND_WIDTH)
    );
    let message = output::truncate(&event.message, EVENT_MESSAGE_WIDTH);
    format!("{time}  {level}  {source}  {kind}  {message}",)
}

fn format_file_line(label: &str, line: &str, color: bool) -> String {
    let tag = output::style_text(label, output::Style::Dim, color);
    format!("{tag}  {line}")
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

fn current_len(path: &Path) -> u64 {
    std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}

async fn read_from(path: &Path, offset: u64) -> crate::app::Result<(Vec<String>, u64)> {
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((Vec::new(), 0)),
        Err(error) => return Err(error.into()),
    };

    let len = file.metadata()?.len();
    // A shorter file than our offset means it was truncated/rotated; restart.
    let start = if len < offset { 0 } else { offset };
    if len == start {
        return Ok((Vec::new(), start));
    }

    file.seek(SeekFrom::Start(start))?;
    let mut tokio_file = tokio::fs::File::from_std(file);
    let mut contents = String::new();
    tokio_file.read_to_string(&mut contents).await?;

    let lines = contents
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    Ok((lines, len))
}
