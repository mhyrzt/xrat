use super::super::*;
use super::test_support::{test_context, test_node, test_node_with, test_source};
use crate::app::daemon::server::RotationTrigger;
use crate::xray::runtime as xray_runtime;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Child, Command, Stdio};

mod fake_runtime;
mod handoff_cases;
mod rejection_cases;
mod spawn_cases;
mod support;
