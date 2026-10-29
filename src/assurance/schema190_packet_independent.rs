//! Independent byte-level packet projection checks; link under regressions.
use super::*;

// Build the published schema-189 field order from actual serialized source
// fields and the unaffected packet bytes. This is a separate test oracle from
// the projector's recursive object parser.
fn schema189_packet(packet: &SpellView) -> String {
    let source = packet.source.expect("object-backed fixture");
    let mut shallow = packet.clone();
    let event = shallow.copy_event.take();
    let json = serde_json::to_string(&shallow).unwrap();
    let prefix = format!("{{\"source\":{},", serde_json::to_string(&source).unwrap());
    let rest = json.strip_prefix(&prefix).expect("source is the first packet field");
    let mut old = format!("{{{rest}");
    let anchor = if shallow.split_second { "\"split_second\":" } else { "\"source_keywords\":" };
    old = old.replacen(anchor, &format!("\"source_card\":{},{}", serde_json::to_string(&source.card).unwrap(), anchor), 1);
    old = old.replacen("\"related_stack\":", &format!("\"source_facts\":{},\"related_stack\":", serde_json::to_string(&source.facts).unwrap()), 1);
    old = old.replacen("\"targets\":", &format!("\"card\":{},\"object\":{},\"targets\":", source.id, serde_json::to_string(&source.object).unwrap()), 1);
    if let Some(event) = event {
        let copy = format!("\"copy_event\":{{\"spell\":{},\"amount\":{}}}", schema189_packet(&event.spell), event.amount);
        old = old.replacen("\"copy_event\":null", &copy, 1);
    }
    old
}

#[test]
fn actual_packets_project_to_schema189_bytes_with_optional_and_nested_fields() {
    for player in [Player::P0, Player::P1] {
        for split in [false, true] {
            let mut g = scenario(player);
            let id = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            let mut inner = g.activation_item(player, id, 0);
            inner.loyalty_x = Some(u64::from(u32::MAX) + 7);
            inner.reflexive_target_limit = Some(2);
            inner.source_transform = Some(9);
            inner.split_second = split;
            let mut outer = g.activation_item(player, id, 0);
            outer.source_transform = Some(4);
            outer.reflexive_target_limit = Some(3);
            outer.split_second = split;
            outer.copy_event = Some(Box::new(stack_copies::CopyEvent { spell: inner, amount: 2 }));
            let raw = serde_json::to_string(&outer).unwrap();
            assert_eq!(project_prior_schema190(&raw).unwrap(), schema189_packet(&outer));
            g.state.stack.push(outer);
            let saved = g.snapshot();
            assert!(prior_schema190_state_supported(&g.state));
            for viewer in [Player::P0, Player::P1] {
                let view = g.observe(viewer);
                let packet = view.stack.last().unwrap();
                let actual = serde_json::to_string(packet).unwrap();
                assert_eq!(project_prior_schema190(&actual).unwrap(), schema189_packet(packet));
            }
            g.begin_attack_rollback(player);
            g.state.stack.clear();
            assert!(prior_schema190_state_supported(&g.state));
            let rollback = g.snapshot();
            g.state.cast_rollback = None;
            g.restore(&rollback).unwrap();
            assert!(prior_schema190_state_supported(&g.state));
            g.restore(&saved).unwrap();
            assert_eq!(g.snapshot(), saved);
            for viewer in [Player::P0, Player::P1] {
                let view = g.observe(viewer);
                let packet = view.stack.last().unwrap();
                assert_eq!(project_prior_schema190(&serde_json::to_string(packet).unwrap()).unwrap(), schema189_packet(packet));
            }
        }
    }
}
