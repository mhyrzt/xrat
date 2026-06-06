mod desktop;
mod endpoints;
mod pac;
mod shell;

use crate::app::context::AppContext;
use crate::cli::{ProxyAction, ProxyArgs, ProxyPacAction};

pub async fn run(context: &AppContext, args: &ProxyArgs) -> crate::app::Result<()> {
    match &args.action {
        ProxyAction::Endpoints(endpoints_args) => {
            endpoints::run(context, endpoints_args.json).await
        }
        ProxyAction::Pac(pac_args) => match &pac_args.action {
            ProxyPacAction::Url(_) => {
                pac::print_pac_url(context);
                Ok(())
            }
            ProxyPacAction::Print(_) => pac::print_pac_file(context).await,
        },
        ProxyAction::Shell(shell_args) => shell::run(context, &shell_args.action).await,
        ProxyAction::Desktop(desktop_args) => desktop::run(context, &desktop_args.action).await,
    }
}

/// Active local proxy endpoints resolved from the running runtime session.
/// Hosts/ports only; never includes Shadowsocks credentials.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ActiveEndpoints {
    pub http: Option<(String, u16)>,
    pub socks: Option<(String, u16)>,
    pub shadowsocks: Option<(String, u16)>,
}

pub(crate) async fn resolve_active_endpoints(
    context: &AppContext,
) -> crate::app::Result<ActiveEndpoints> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        return Ok(ActiveEndpoints::default());
    };

    Ok(ActiveEndpoints {
        http: endpoint(session.http_host, session.http_port),
        socks: endpoint(session.socks_host, session.socks_port),
        shadowsocks: endpoint(session.shadowsocks_host, session.shadowsocks_port),
    })
}

fn endpoint(host: Option<String>, port: Option<i64>) -> Option<(String, u16)> {
    match (host, port) {
        (Some(host), Some(port)) if port > 0 && port <= i64::from(u16::MAX) => {
            Some((host, port as u16))
        }
        _ => None,
    }
}
