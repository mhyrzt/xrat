use crate::app::context::AppContext;
use crate::tui::app::{QrModalState, TuiApp};

pub async fn open_qr_for_config(
    context: &AppContext,
    app: &mut TuiApp,
    config_id: i64,
    config_name: String,
) {
    match context.db.get_config_by_id(config_id).await {
        Ok(Some(record)) => {
            app.qr_modal = Some(QrModalState::new(config_name, record.raw_config));
        }
        Ok(None) => app.set_status(format!("config {config_id} not found")),
        Err(err) => app.set_status(format!("failed to load config URI: {err}")),
    }
}

pub async fn copy_config_uri(context: &AppContext, app: &mut TuiApp, config_id: i64) {
    match context.db.get_config_by_id(config_id).await {
        Ok(Some(record)) => {
            set_clipboard(app, record.raw_config);
        }
        Ok(None) => app.set_status(format!("config {config_id} not found")),
        Err(err) => app.set_status(format!("failed to load config URI: {err}")),
    }
}

pub async fn copy_selected_uris(context: &AppContext, app: &mut TuiApp, ids: Vec<i64>) {
    let mut uris: Vec<String> = Vec::new();
    for id in &ids {
        match context.db.get_config_by_id(*id).await {
            Ok(Some(record)) => uris.push(record.raw_config),
            Ok(None) => {}
            Err(err) => {
                app.set_status(format!("failed to load config {id}: {err}"));
                return;
            }
        }
    }
    if uris.is_empty() {
        app.set_status("no URIs found for selected configs".to_string());
        return;
    }
    let text = uris.join("\n");
    set_clipboard(app, text);
}

pub fn open_qr_for_source(app: &mut TuiApp, source_name: String, source_url: String) {
    app.qr_modal = Some(QrModalState::new(source_name, source_url));
}

pub fn copy_source_uri(app: &mut TuiApp, source_url: String) {
    set_clipboard(app, source_url);
}

pub fn open_qr_for_api_url(app: &mut TuiApp) {
    let url = app.data.api_b64_url.clone();
    if url.is_empty() {
        app.set_status("API subscription URL not available");
        return;
    }
    app.qr_modal = Some(QrModalState::new("API /b64 subscription", url));
}

pub fn copy_api_url(app: &mut TuiApp) {
    let url = app.data.api_b64_url.clone();
    if url.is_empty() {
        app.set_status("API subscription URL not available");
        return;
    }
    set_clipboard(app, url);
}

fn set_clipboard(app: &mut TuiApp, text: String) {
    match arboard::Clipboard::new() {
        Ok(mut cb) => match cb.set_text(&text) {
            Ok(()) => {
                let preview = if text.len() > 40 {
                    format!("{}…", &text[..40])
                } else {
                    text.clone()
                };
                app.set_status(format!("copied: {preview}"));
            }
            Err(err) => app.set_status(format!("clipboard write failed: {err}")),
        },
        Err(err) => app.set_status(format!("clipboard unavailable: {err}")),
    }
}
