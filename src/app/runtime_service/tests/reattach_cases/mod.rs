use super::super::reattach::ProcessInspector;
use super::super::*;
use super::test_support::{test_context, test_node, test_source};

mod accept_cases;
mod reject_cmdline_case;
mod reject_pid_exec_cases;

struct AcceptingInspector;

impl ProcessInspector for AcceptingInspector {
    fn is_running(&self, _pid: i64) -> bool {
        true
    }

    fn exec_matches_runtime_engine(
        &self,
        _context: &AppContext,
        _session_id: i64,
        _pid: i64,
    ) -> bool {
        true
    }

    fn cmdline_matches_session_config(
        &self,
        _context: &AppContext,
        _pid: i64,
        _session_id: i64,
    ) -> bool {
        true
    }
}

struct CmdlineMismatchInspector;

impl ProcessInspector for CmdlineMismatchInspector {
    fn is_running(&self, _pid: i64) -> bool {
        true
    }

    fn exec_matches_runtime_engine(
        &self,
        _context: &AppContext,
        _session_id: i64,
        _pid: i64,
    ) -> bool {
        true
    }

    fn cmdline_matches_session_config(
        &self,
        _context: &AppContext,
        _pid: i64,
        _session_id: i64,
    ) -> bool {
        false
    }
}
