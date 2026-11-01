//! Focused printed outcomes; shared engine invariants live in core tests.
//! Exact source identity for a craft ability's exile-cost return.
use super::*;

#[test]
fn jewel_mana_pays_ability_costs_but_not_spells() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        g.state.players[player.index()].mana = [0; 6];
        let jewel = put(&mut g, player, "The Enigma Jewel", Zone::Hand);
        let entry = g.entry_move(jewel, Zone::Battlefield, false);
        g.stage_entries(vec![entry], None);
        assert!(g.object_view(jewel).tapped);
        g.untap_objects(&[jewel], untap_rules::UntapCause::Effect);
        let bow = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, player, "Short Bow", Zone::Hand);
        choose(&mut g, |action| {
            matches!(action,
            Action::ActivateMana { card, production, .. }
                if *card == jewel && *production == [0, 0, 0, 0, 0, 2])
        });
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: bow,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(bow).attached_to, Some(g.object_ref(creature)));
    }
}

#[test]
fn locus_copies_your_nonmana_activation_and_allows_retargeting() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let locus = put(&mut g, player, "Locus of Enlightenment", Zone::Battlefield);
        let bow = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let first = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: bow,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        pay(&mut g);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(g.state.stack.iter())
                .any(|trigger| {
                    trigger.source.is_some_and(|source| source.id == locus)
                        && trigger.copy_event.is_some()
                })
        );
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::Retarget { .. }) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::Retarget { .. }));
        let alternative = g.legal_actions().into_iter().find(|action| {
            matches!(action, Action::SetStackTarget { target: Target::Card(card), .. } if *card == second)
        }).expect("copy offers a new target");
        big_helpers::checked(&mut g, alternative);
        big_helpers::checked(&mut g, Action::FinishStackTargets);
        assert!(g.state.stack.iter().any(|packet| packet.is_copy));
        settle(&mut g);
        assert_eq!(g.object_view(bow).attached_to, Some(g.object_ref(first)));
    }
}

pub(super) fn begin_craft(g: &mut Game, source: CardId, ability: u16) {
    big_helpers::checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    big_helpers::checked(g, Action::BeginOrderedCosts);
}
pub(super) fn craft_clause(g: &mut Game, clause: CastCostClause) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == clause && !row.paid)
        .unwrap() as u16;
    big_helpers::checked(g, Action::ChooseCastCost { clause: index });
}
pub(super) fn craft_material(g: &mut Game, material: CardId) {
    craft_clause(g, CastCostClause::Additional);
    let object = g.object_ref(material);
    big_helpers::checked(g, Action::ChooseAdditionalCost { object });
}
pub(super) fn craft_mana(g: &mut Game) {
    craft_clause(g, CastCostClause::Mana);
    pay(g);
}
pub(super) fn craft_source(g: &mut Game) {
    craft_clause(g, CastCostClause::CraftSource);
}

#[test]
fn jewel_crafts_with_four_or_more_and_links_exact_exiled_cards() {
    {
        let player = Player::P0;
        for count in [4, 5] {
            let mut g = scenario(player);
            let jewel = put(&mut g, player, "The Enigma Jewel", Zone::Battlefield);
            let mut materials = Vec::new();
            for n in 0..count {
                materials.push(put(
                    &mut g,
                    player,
                    "Short Bow",
                    if n % 2 == 0 {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    },
                ));
            }
            let invalid = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            let grant_only = put(&mut g, player, "Sanctum Lurker", Zone::Battlefield);
            begin_craft(&mut g, jewel, 1);
            craft_clause(&mut g, CastCostClause::Additional);
            assert!(!g.legal_actions().contains(&Action::ChooseAdditionalCost {
                object: g.object_ref(invalid)
            }));
            assert!(!g.legal_actions().contains(&Action::ChooseAdditionalCost {
                object: g.object_ref(grant_only)
            }));
            for (index, &material) in materials.iter().enumerate() {
                let object = g.object_ref(material);
                big_helpers::checked(&mut g, Action::ChooseAdditionalCost { object });
                assert_eq!(
                    g.legal_actions().contains(&Action::FinishCraftMaterials),
                    index >= 3
                );
            }
            big_helpers::checked(&mut g, Action::FinishCraftMaterials);
            craft_source(&mut g);
            craft_mana(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(jewel).card.name(), "Locus of Enlightenment");
            for material in &materials {
                assert_eq!(g.object_view(*material).zone, Zone::Exile);
            }
            let inherited = g.borrowed_ability_views();
            assert_eq!(inherited.len(), count);
            for row in inherited {
                assert!(materials.contains(&row.exiled.card));
            }
        }
    }
}
