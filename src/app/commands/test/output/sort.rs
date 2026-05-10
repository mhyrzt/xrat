use super::*;

pub(crate) fn sort_results(outputs: &mut [TestOutputRow], sort_by: TestSortBy) {
    outputs
        .sort_by(|left, right| compare_results(left, right, sort_by).then(left.id.cmp(&right.id)));
}

fn compare_results(left: &TestOutputRow, right: &TestOutputRow, sort_by: TestSortBy) -> Ordering {
    match sort_by {
        TestSortBy::Status => left.status.cmp(&right.status),
        TestSortBy::Icmp => compare_optional_u32(left.icmp_ms, right.icmp_ms),
        TestSortBy::RealDelay => compare_optional_u32(left.real_delay_ms, right.real_delay_ms),
        TestSortBy::DownloadSpeed => {
            compare_optional_f64_desc(left.download_mbps, right.download_mbps)
        }
        TestSortBy::Protocol => left.protocol.cmp(&right.protocol),
        TestSortBy::Address => left.address.cmp(&right.address),
    }
}

fn compare_optional_u32(left: Option<u32>, right: Option<u32>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_optional_f64_desc(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.partial_cmp(&left).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}
