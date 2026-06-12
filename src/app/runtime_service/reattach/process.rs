use std::path::PathBuf;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

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

fn refresh_process(pid: i64) -> Option<(System, Pid)> {
    let pid = u32::try_from(pid).ok().map(Pid::from_u32)?;
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::nothing()
            .with_exe(UpdateKind::Always)
            .with_cmd(UpdateKind::Always),
    );
    Some((system, pid))
}

fn process_exe_path(pid: i64) -> Option<PathBuf> {
    let (system, pid) = refresh_process(pid)?;
    system
        .process(pid)
        .and_then(|process| process.exe().map(|exe| exe.to_path_buf()))
}

fn process_cmd_args(pid: i64) -> Option<Vec<String>> {
    let (system, pid) = refresh_process(pid)?;
    let process = system.process(pid)?;
    Some(
        process
            .cmd()
            .iter()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect(),
    )
}

fn process_exec_matches_runtime_engine(context: &AppContext, session_id: i64, pid: i64) -> bool {
    let Some(process_exe) = process_exe_path(pid) else {
        return false;
    };
    let args = process_cmd_args(pid).unwrap_or_default();

    let expected = if cmdline_contains_session_config(&args, context, session_id, "singbox.json") {
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

fn process_cmdline_matches_session_config(context: &AppContext, pid: i64, session_id: i64) -> bool {
    let Some(args) = process_cmd_args(pid) else {
        return false;
    };
    cmdline_contains_session_config(&args, context, session_id, "json")
        || cmdline_contains_session_config(&args, context, session_id, "singbox.json")
}

fn cmdline_contains_session_config(
    args: &[String],
    context: &AppContext,
    session_id: i64,
    suffix: &str,
) -> bool {
    let expected = context
        .runtime_paths
        .runtime_dir
        .join(format!("session-{session_id}.{suffix}"));
    let expected = expected.to_string_lossy();
    args.iter().any(|arg| arg.contains(expected.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::{process_cmd_args, process_exe_path};

    #[test]
    fn resolves_exe_and_cmd_for_spawned_process() {
        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep");
        let pid = child.id() as i64;

        let exe = process_exe_path(pid);
        let args = process_cmd_args(pid);

        let _ = child.kill();
        let _ = child.wait();

        let exe = exe.expect("expected to resolve exe for spawned process");
        assert!(exe.file_name().is_some());

        let args = args.expect("expected cmd args for spawned process");
        assert!(
            args.iter().any(|arg| arg.contains("30")),
            "spawned process args missing argument: {args:?}"
        );
    }
}
