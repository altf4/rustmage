use crate::engine::ability_world::CharacteristicWorld;
// Focused printed outcomes; shared engine invariants live in core tests.
// Whole-card BIG library rules, paid actions and owned branch/restore checks.
use super::*;
use crate::engine::testing::card_cases::big_helpers::{cast, checked, named, settle, setup, spawn};
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn reject(g: &mut Game, a: Action) {
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
}
fn activate(g: &mut Game, card: CardId, ability: u16, target: Option<Target>) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    until(g, |g| g.state.announcing.is_none());
}
fn select_library(g: &mut Game, id: CardId) {
    checked(g, Action::SelectLibrary { card: id });
    checked(g, Action::FinishLibrarySelection);
}
fn start_step(g: &mut Game, p: Player, step: Step) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = step;
    g.begin_step(step);
    g.prepare_triggers();
}
fn token(subtype: &str) -> Card {
    Card::catalogue()
        .find(|card| {
            card.token_info()
                .is_some_and(|info| info.subtypes == subtype)
        })
        .unwrap()
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(*id as CardId, subtype)
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn big_farmer_entry_attack_food_cost_exact_milled_recovery_and_decline_both_seats() {
    {
        let p = Player::P0;
        for pay in [false, true] {
            let mut g = setup(p);
            let farmer = named(&mut g, p, "Bristlebud Farmer", Zone::Hand);
            cast(&mut g, farmer, None, &[]);
            settle(&mut g);
            assert_eq!(tokens(&g, p, "Food").len(), 2);
            assert_eq!((g.stats(farmer).power, g.stats(farmer).toughness), (5, 5));

            let prior = named(&mut g, p, "Forest", Zone::Graveyard);
            let recovered = named(&mut g, p, "Forest", Zone::Library);
            let spell = named(&mut g, p, "Giant Growth", Zone::Library);
            named(&mut g, p, "Grizzly Bears", Zone::Library);
            let count = g.state.players[p.index()].library.len();
            g.state.objects[farmer as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: farmer });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            let opponent_food = spawn(&mut g, p.other(), token("Food"), Zone::Battlefield);
            let bad = g.object_ref(opponent_food);
            reject(&mut g, Action::Sacrifice { object: bad });
            if pay {
                let food = g.object_ref(tokens(&g, p, "Food")[0]);
                // A queued attack trigger survives the source's departure.
                g.move_object(farmer, Zone::Graveyard);
                checked(&mut g, Action::Sacrifice { object: food });
                assert_eq!(g.state.players[p.index()].library.len(), count - 3);
                assert_eq!(tokens(&g, p, "Food").len(), 1);
                let old = g.object_ref(prior);
                reject(&mut g, Action::RecoverMilled { object: Some(old) });
                let instant = g.object_ref(spell);
                reject(
                    &mut g,
                    Action::RecoverMilled {
                        object: Some(instant),
                    },
                );
                let object = g.object_ref(recovered);
                checked(
                    &mut g,
                    Action::RecoverMilled {
                        object: Some(object),
                    },
                );
                assert_eq!(g.object_view(recovered).zone, Zone::Hand);
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
                assert_eq!(g.state.players[p.index()].library.len(), count);
                assert_eq!(tokens(&g, p, "Food").len(), 2);
            }
        }
    }
}

#[test]
fn big_loot_counts_other_nonland_types_and_permission_survives_source() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let loot = named(&mut g, p, "Loot, the Key to Everything", Zone::Battlefield);
        named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "Omenpath Journey", Zone::Battlefield);
        spawn(&mut g, p, token("Food"), Zone::Battlefield);
        named(&mut g, p, "Forest", Zone::Battlefield);
        let count = g.state.players[p.index()].library.len();
        start_step(&mut g, p, Step::Upkeep);
        g.move_object(loot, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), count - 3);
        let permissions = &g.state.exile_permissions;
        assert_eq!(permissions.len(), 3);
        assert!(
            permissions
                .iter()
                .all(|x| x.player == p && x.expiry == ExilePlayExpiry::Turn(g.state.turn))
        );
    }
}

#[test]
fn big_font_chooses_token_on_resolution_and_pays_three_distinct_token_names_atomically() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let font = named(&mut g, p, "Transmutation Font", Zone::Battlefield);
        let found = named(&mut g, p, "Solemn Simulacrum", Zone::Library);
        for (card, subtype) in [
            (token("Blood"), "Blood"),
            (token("Clue"), "Clue"),
            (token("Food"), "Food"),
        ] {
            g.state.objects[font as usize].tapped = false;
            activate(&mut g, font, 0, None);
            assert!(
                g.state
                    .stack
                    .iter()
                    .any(|s| s.source.is_some_and(|source| source.id == font))
            );
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::TokenChoice { .. })
            });
            assert_eq!(tokens(&g, p, subtype).len(), 0);
            checked(&mut g, Action::ChooseToken { token: card });
            settle(&mut g);
            assert_eq!(tokens(&g, p, subtype).len(), 1);
        }
        let duplicate = spawn(&mut g, p, token("Food"), Zone::Battlefield);
        g.state.objects[font as usize].tapped = false;
        let chosen = [
            tokens(&g, p, "Blood")[0],
            tokens(&g, p, "Clue")[0],
            tokens(&g, p, "Food")[0],
        ];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: font,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        let first = g.object_ref(chosen[2]);
        checked(&mut g, Action::ChooseAdditionalCost { object: first });
        let same = g.object_ref(duplicate);
        reject(&mut g, Action::ChooseAdditionalCost { object: same });
        for id in &chosen[..2] {
            let object = g.object_ref(*id);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        assert!(
            chosen
                .iter()
                .all(|&id| g.object_view(id).zone == Zone::Battlefield)
        );
        until(&mut g, |g| g.state.announcing.is_none());
        assert!(
            chosen
                .iter()
                .all(|&id| g.object_view(id).zone == Zone::Outside)
        );
        assert!(g.object_view(font).tapped);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert!(g.observe(p.other()).search_choice.is_none());
        checked(&mut g, Action::ChooseSearch { card: found });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(found).zone, Zone::Battlefield);
        assert_eq!(g.object_view(duplicate).zone, Zone::Battlefield);
    }
}

#[test]
fn big_cage_free_land_keeps_tarnation_color_choice_and_obeys_land_allowance() {
    {
        let p = Player::P0;
        for already_played in [false, true] {
            let mut g = setup(p);
            let land = named(&mut g, p, "Tarnation Vista", Zone::Library);
            let cage = named(&mut g, p, "Collector's Cage", Zone::Hand);
            cast(&mut g, cage, None, &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
            });
            select_library(&mut g, land);
            named(&mut g, p, "Helpful Hunter", Zone::Battlefield);
            named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let target = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.state.land_played = already_played;
            activate(&mut g, cage, 0, Some(Target::Card(target)));
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionCasts { .. })
            });
            if already_played {
                reject(&mut g, Action::PlayLand { card: land });
                assert_eq!(g.object_view(land).zone, Zone::Exile);
            } else {
                checked(&mut g, Action::PlayLand { card: land });
                assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .any(|o| o.reference.card == land)
                );
                checked(
                    &mut g,
                    Action::ChooseEntryValue {
                        value: EntryChoiceValue::Color(crate::ChoiceColor::Green),
                    },
                );
                assert!(matches!(g.state.pending, Pending::ResolutionCasts { .. }));
                assert_eq!(g.object_view(land).zone, Zone::Battlefield);
                assert!(g.object_view(land).tapped);
                assert_eq!(
                    g.entry_value(g.object_ref(land)),
                    Some(EntryChoiceValue::Color(crate::ChoiceColor::Green))
                );
                assert!(g.state.land_played);
            }
            checked(&mut g, Action::FinishResolutionCasts);
            settle(&mut g);
        }
    }
}

#[test]
fn big_font_helm_adds_map_to_chosen_token_event_and_map_can_pay_distinct_cost() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let font = named(&mut g, p, "Transmutation Font", Zone::Battlefield);
        named(&mut g, p, "Worldwalker Helm", Zone::Battlefield);
        activate(&mut g, font, 0, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::TokenChoice { .. })
        });
        checked(
            &mut g,
            Action::ChooseToken {
                token: token("Blood"),
            },
        );
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Blood").len(), 1);
        assert_eq!(tokens(&g, p, "Map").len(), 1);
        let food = spawn(&mut g, p, token("Food"), Zone::Battlefield);
        let blood = tokens(&g, p, "Blood")[0];
        let map = tokens(&g, p, "Map")[0];
        g.state.objects[font as usize].tapped = false;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: font,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        for id in [food, blood, map] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        until(&mut g, |g| g.state.announcing.is_none());
        assert!(
            [food, blood, map]
                .into_iter()
                .all(|id| g.object_view(id).zone == Zone::Outside)
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
    }
}
