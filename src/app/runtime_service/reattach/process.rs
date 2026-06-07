use super::*;

pub(super) struct SystemProcessInspector;

impl ProcessInspector for SystemProcessInspector {
    fn is_running(&self, pid: i64) -> bool {
        xray_runtime::process_is_running(pid)
    }

    fn exec_matches_runtime_engine(&self, context: &AppContext, session_id: i64, pid: i64) -> bool {
        process_exec_matches_runtime_engine(context, session_id, pid)
    }

    fn cmdline_matches_session_config(
        &self,
        context: &AppContext,
        pid: i64,
        session_id: i64,
    ) -> bool {
        process_cmdline_matches_session_config(context, pid, session_id)
    }
}

#[cfg(unix)]
fn process_exec_matches_runtime_engine(context: &AppContext, session_id: i64, pid: i64) -> bool {
    let process_exe = std::fs::read_link(format!("/proc/{pid}/exe")).ok();
    let Some(process_exe) = process_exe else {
        return false;
    };

    let expected =
        if process_cmdline_contains_session_config(context, pid, session_id, "singbox.json") {
            &context.runtime_paths.sing_box_path
        } else {
            match context.app_config.runtime.engine.as_str() {
                "xray" => &context.runtime_paths.xray_path,
                "v2ray" => &context.runtime_paths.v2ray_path,
                "sing-box" => &context.runtime_paths.sing_box_path,
                _ => &context.runtime_paths.xray_path,
            }
        };
    process_exe
        .file_name()
        .map(|name| name == expected.file_name().unwrap_or_default())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn process_exec_matches_runtime_engine(_context: &AppContext, _session_id: i64, _pid: i64) -> bool {
    false
}

#[cfg(unix)]
fn process_cmdline_matches_session_config(context: &AppContext, pid: i64, session_id: i64) -> bool {
    process_cmdline_contains_session_config(context, pid, session_id, "json")
        || process_cmdline_contains_session_config(context, pid, session_id, "singbox.json")
}

#[cfg(not(unix))]
fn process_cmdline_matches_session_config(
    _context: &AppContext,
    _pid: i64,
    _session_id: i64,
) -> bool {
    false
}

#[cfg(unix)]
fn process_cmdline_contains_session_config(
    context: &AppContext,
    pid: i64,
    session_id: i64,
    suffix: &str,
) -> bool {
    let cmdline_bytes = std::fs::read(format!("/proc/{pid}/cmdline")).ok();
    let Some(cmdline_bytes) = cmdline_bytes else {
        return false;
    };
    let expected = context
        .runtime_paths
        .runtime_dir
        .join(format!("session-{session_id}.{suffix}"));
    let expected = expected.to_string_lossy();
    let cmdline = cmdline_bytes
        .split(|byte| *byte == 0)
        .filter(|segment| !segment.is_empty())
        .map(|segment| String::from_utf8_lossy(segment).to_string())
        .collect::<Vec<_>>();
    cmdline.iter().any(|arg| arg.contains(expected.as_ref()))
}
