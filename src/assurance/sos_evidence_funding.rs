//! Evidence and mana costs share the exact graveyard incarnations they consume.
use super::*;
use big_helpers::{checked, settle as finish};

fn rubble(source: CardId) -> Action {
    Action::AnnounceMana {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
        production: [0, 0, 0, 1, 0, 0],
    }
}
fn fixture(p: Player) -> (Game, CardId, CardId, CardId, CardId) {
    let mut g = scenario(p);
    let source = put(&mut g, p, "Rubble Rouser", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let evidence = put(&mut g, p, "Colossus of the Blood Age", Zone::Graveyard);
    let spell = put(&mut g, p, "Bite Down on Crime", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
    (g, source, enemy, evidence, spell)
}
fn targets(g: &mut Game, source: CardId, enemy: CardId) {
    for card in [source, enemy] {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
}
fn pay_checked(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
#[test]
fn sos_evidence_funding_cannot_exile_its_only_evidence_to_fund_the_discount() {
    for p in [Player::P0, Player::P1] {
        let (mut g, _, _, evidence, spell) = fixture(p);
        assert_eq!(g.object_view(evidence).card.fixed_mana_value(), 6);
        let before = g.snapshot();
        for _ in 0..2 {
            assert!(
                !g.legal_actions()
                    .contains(&Action::AnnounceCast { card: spell })
            );
        }
        assert_eq!(g.snapshot(), before);
        reject(&mut g, Action::AnnounceCast { card: spell });
        // A real alternative source of generic mana makes the declaration legal.
        let forest = put(&mut g, p, "Forest", Zone::Battlefield);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseCollectEvidence { collect: true });
        assert!(!g.object_view(forest).tapped);
    }
}
#[test]
fn sos_evidence_funding_declared_intent_uses_the_same_virtual_graveyard() {
    for p in [Player::P0, Player::P1] {
        let (mut g, _, _, _, spell) = fixture(p);
        // Establish the declaration boundary with legal independent mana, then
        // isolate the intent query with its sole graveyard-consuming producer.
        let forest = put(&mut g, p, "Forest", Zone::Battlefield);
        checked(&mut g, Action::AnnounceCast { card: spell });
        g.state.objects[forest as usize].tapped = true;
        reject(&mut g, Action::ChooseCollectEvidence { collect: true });
        g.state.objects[forest as usize].tapped = false;
        checked(&mut g, Action::ChooseCollectEvidence { collect: true });
    }
}
#[test]
fn sos_evidence_funding_nested_rubble_preserves_selected_exact_evidence() {
    for p in [Player::P0, Player::P1] {
        for complete in [false, true] {
            for fuel_name in ["Forest", "Colossus of the Blood Age"] {
                let (mut g, source, enemy, evidence, spell) = fixture(p);
                let fuel = put(&mut g, p, fuel_name, Zone::Graveyard);
                checked(&mut g, Action::AnnounceCast { card: spell });
                checked(&mut g, Action::ChooseCollectEvidence { collect: true });
                targets(&mut g, source, enemy);
                let selected = g.object_ref(evidence);
                checked(&mut g, Action::ChooseEvidence { object: selected });
                assert!(g.observe(p.other()).evidence.is_none());
                if complete {
                    checked(&mut g, Action::FinishEvidence);
                }
                checked(&mut g, rubble(source));
                let own = g.observe(p).suspended_mana_announcements;
                let other = g.observe(p.other()).suspended_mana_announcements;
                assert_eq!(own.len(), 1);
                assert!(other[0].item.targets.is_empty());
                reject(&mut g, Action::ChooseAdditionalCost { object: selected });
                let fuel_ref = g.object_ref(fuel);
                checked(&mut g, Action::ChooseAdditionalCost { object: fuel_ref });
                pay_checked(&mut g);
                assert_eq!(g.object_view(fuel).zone, Zone::Exile);
                assert_eq!(g.object_ref(evidence), selected);
                assert_eq!(g.object_view(evidence).zone, Zone::Graveyard);
                if !complete {
                    checked(&mut g, Action::FinishEvidence);
                }
                pay_checked(&mut g);
                finish(&mut g);
                assert_eq!(g.object_view(evidence).zone, Zone::Exile);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
            }
        }
    }
}
