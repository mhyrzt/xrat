use super::*;
use xrat_support::net::KernelRuleAttribute;

#[test]
fn nftables_flowtable_blocks_gateway_preflight() {
    let report = inspect_nft_output(
        br#"{"nftables":[{"flowtable":{"family":"inet","table":"foreign","name":"fastpath"}}]}"#,
    )
    .unwrap();
    assert_eq!(report.state, "failed");
}

#[test]
fn nftables_reports_base_chain_and_host_guard_without_asserting_protection() {
    let data = br#"{"nftables":[{"table":{"family":"inet","name":"xrat_killswitch"}},{"chain":{"family":"inet","table":"foreign","name":"forward","hook":"forward","prio":0}}]}"#;
    let result = inspect_nft_output(data).unwrap();
    assert_eq!(result.state, "incomplete");
    assert!(
        result
            .detail
            .contains("inet/foreign/forward hook forward priority 0")
    );
    assert!(result.detail.contains("does not protect forwarded traffic"));
    assert!(inspect_nft_output(br#"{"nftables":[{"chain":{"hook":"forward"}}]}"#).is_err());
    assert!(inspect_nft_output(b"invalid").is_err());
}

#[test]
fn marks_unknown_when_rules_unavailable_and_do_not_claim_conflict_free() {
    assert_eq!(inspect_rule_marks(None).state, "unknown");
    let rule = KernelPolicyRule {
        family: 2,
        destination_prefix: 0,
        source_prefix: 0,
        tos: 0,
        table: 100,
        action: 1,
        flags: 0,
        attributes: vec![
            KernelRuleAttribute {
                kind: 10,
                value: 1u32.to_ne_bytes().to_vec(),
            },
            KernelRuleAttribute {
                kind: 6,
                value: 100u32.to_ne_bytes().to_vec(),
            },
        ],
    };
    let result = inspect_rule_marks(Some(&[rule]));
    assert_eq!(result.state, "incomplete");
    assert!(result.detail.contains("priorities 100"));
    assert!(result.detail.contains("conflicts cannot be ruled out"));
}
