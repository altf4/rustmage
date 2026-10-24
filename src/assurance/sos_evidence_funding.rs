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

#[test]
fn sos_evidence_funding_returned_courtyard_uses_projected_chosen_type_quality() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        // A bounded live-state fixture: the Reliquary has an exact linked land
        // (previously animated while exiled), is animated itself, and can receive
        // equipment. All subsequent return, entry and mana decisions are paid.
        let prison = put(&mut g, p, "Dusk Rose Reliquary", Zone::Battlefield);
        let land = put(&mut g, p, "Secluded Courtyard", Zone::Exile);
        g.state.exile_links.push(ExileLink {
            source: g.object_ref(prison),
            exiled: g.object_ref(land),
            return_zone: Zone::Battlefield,
        });
        let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let mut animation = g.activation_item(ring, 0);
        animation.card = prison;
        animation.object = g.object_ref(prison);
        g.animate_source(
            &animation,
            crate::cards::AnimationSpec {
                artifact: true,
                creature_types: crate::cards::AnimationCreatureTypes::Named(&["Golem"]),
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
        );
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ring,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(prison),
            },
        );
        pay_checked(&mut g);
        finish(&mut g);
        let spell = put(&mut g, p, "Fractal Tender", Zone::Hand);
        let instant_sorcery = put(&mut g, p, "Prismari Charm", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        // Three mana of a single color from the Ring cannot pay both colors.
        // The projected new Courtyard must supply the other creature-spell color.
        reject(
            &mut g,
            Action::AnnounceCast {
                card: instant_sorcery,
            },
        );
        checked(&mut g, Action::AnnounceCast { card: spell });
        let grantor = g.object_ref(ring);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: prison,
                grantor,
                production: [0, 3, 0, 0, 0, 0],
            },
        );
        let elf = crate::cards::CreatureType::from_name("Elf").unwrap();
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(elf),
            },
        );
        checked(
            &mut g,
            Action::ActivateMana {
                card: land,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        pay_checked(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(prison).zone, Zone::Graveyard);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        assert_eq!(
            g.entry_value(g.object_ref(land)),
            Some(EntryChoiceValue::CreatureType(elf))
        );
    }
}
