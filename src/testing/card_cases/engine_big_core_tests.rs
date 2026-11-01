//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid interactions for the Big Score's mana, lock, and zone-replacement cards.
use super::*;
use crate::cards::ChoiceColor;
use crate::engine::testing::card_cases::big_helpers::spawn;
use crate::engine::testing::card_cases::big_helpers::{cast, checked, settle, setup};

fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    let id = spawn(g, p, card, zone);
    g.register_zone_rule_card(card);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    id
}
fn reject(g: &mut Game, action: Action) {
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
}

#[test]
fn big_abolisher_stops_opposing_spell_and_mana_abilities_allows_lands_and_releases() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let abolisher = paid(&mut g, p, "Grand Abolisher", &[]);
        settle(&mut g);
        let q = p.other();
        let elf = fixture(&mut g, q, "Llanowar Elves", Zone::Battlefield);
        let artifact = fixture(&mut g, q, "Ancient Cornucopia", Zone::Battlefield);
        let forest = fixture(&mut g, q, "Forest", Zone::Battlefield);
        let spell = fixture(&mut g, q, "Giant Growth", Zone::Hand);
        g.state.players[q.index()].mana = [0; 6];
        checked(&mut g, Action::Pass);
        reject(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::TapMana { card: elf });
        assert!(!g.legal_actions().iter().any(|a| matches!(a, Action::ActivateMana { card, .. } | Action::AnnounceMana { card, .. } if *card == artifact)));
        checked(&mut g, Action::TapMana { card: forest });
        assert!(!g.can_fund(funding::FundingGoal {
            actor: q,
            required: [0, 0, 0, 0, 2, 0],
            generic: 0,
            reserve: None,
            source: None
        }));
        g.move_object(abolisher, Zone::Exile);
        assert!(g.legal_actions().contains(&Action::TapMana { card: elf }));
        assert!(g.can_fund(funding::FundingGoal {
            actor: q,
            required: [0, 0, 0, 0, 2, 0],
            generic: 0,
            reserve: None,
            source: None
        }));
        checked(&mut g, Action::TapMana { card: elf });
    }
}

#[test]
fn big_vista_paid_entry_chosen_color_and_distinct_monocolor_mana_in_funding() {
    {
        let p = Player::P0;
        for chosen in ChoiceColor::ALL {
            let mut g = setup(p);
            let vista = fixture(&mut g, p, "Tarnation Vista", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: vista });
            checked(
                &mut g,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::Color(chosen),
                },
            );
            assert!(g.object_view(vista).tapped);
            assert_eq!(
                g.object_view(vista).entry_choice,
                Some(EntryChoiceValue::Color(chosen))
            );
            g.state.objects[vista as usize].tapped = false;
            let mut production = [0; 6];
            production[chosen.index()] = 1;
            let a = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, ability: 0, production: got, .. } if *card == vista && *got == production)).unwrap();
            checked(&mut g, a);
            g.state.objects[vista as usize].tapped = false;
            fixture(&mut g, p, "Savannah Lions", Zone::Battlefield);
            fixture(&mut g, p, "Savannah Lions", Zone::Battlefield);
            fixture(&mut g, p, "Memory Vessel", Zone::Battlefield);
            fixture(&mut g, p, "Kutzil, Malamet Exemplar", Zone::Battlefield);
            fixture(&mut g, p.other(), "Ancient Cornucopia", Zone::Battlefield);
            assert_eq!(
                ManaProduction::MonocoloredPermanentColors.in_world(&g, vista),
                ManaProduction::Fixed([1, 0, 0, 1, 0, 0])
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            assert!(g.can_fund(funding::FundingGoal {
                actor: p,
                required: [1, 0, 0, 1, 0, 0],
                generic: 0,
                reserve: None,
                source: None
            }));
            let before = g.state.players[p.index()].mana;
            let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::AnnounceMana { card, ability: 1, .. } if *card == vista)).unwrap();
            checked(&mut g, action);
            for _ in 0..20 {
                if g.state.announcing.is_none() {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                    .unwrap();
                checked(&mut g, a);
            }
            assert!(g.object_view(vista).tapped);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                before.iter().sum::<u32>() + 1
            );
        }
    }
}
