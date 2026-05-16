use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TestStatus {
    Ok,
    Failed,
    Skipped,
}

impl TestStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

pub(crate) fn overall_status(
    result: &TestResult,
    ran_icmp: bool,
    ran_tcp: bool,
    ran_real_delay: bool,
    ran_download: bool,
    ran_upload: bool,
) -> TestStatus {
    if !ran_icmp && !ran_tcp && !ran_real_delay && !ran_download && !ran_upload {
        return TestStatus::Skipped;
    }

    let success = if ran_upload {
        result.upload_ok
    } else if ran_download {
        result.download_ok
    } else if ran_real_delay {
        result.real_delay_ok
    } else if ran_tcp {
        result.tcp_ok
    } else {
        result.icmp_ok
    };

    if success {
        TestStatus::Ok
    } else {
        TestStatus::Failed
    }
}
