//! Finality replaces actual death, while Thunderbolts returns exact owner objects.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
fn paid(g: &mut Game, p: Player, name: &str, target: Option<Target>) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &target.into_iter().collect::<Vec<_>>());
    settle(g);
    id
}
fn place(g: &mut Game, id: CardId, n: u64) {
    g.effect_counter(
        id,
        CounterKind::Finality,
        n,
        replacements::ResumeMode::Pending,
    );
    settle(g);
}
fn reject(g: &mut Game, a: Action) {
    let snapshot = g.snapshot();
    let bytes = g.state_bytes();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.state_bytes(), bytes);
}
fn pay(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. } | Action::FinishTargets | Action::ChooseCastCost { .. }
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("unpaid")
}
fn sacrifice_spell(g: &mut Game, p: Player, victim: CardId) {
    let spell = named(g, p, "Corrupted Conviction", Zone::Hand);
    checked(g, Action::AnnounceCast { card: spell });
    for _ in 0..20 {
        let action = Action::ChooseAdditionalCost {
            object: g.object_ref(victim),
        };
        if g.legal_actions().contains(&action) {
            checked(g, action);
            break;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. } | Action::FinishTargets | Action::ChooseCastCost { .. }
                )
            })
            .unwrap();
        checked(g, action);
    }
}
fn tokens(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
        .count()
}
#[test]
fn finality_thunderbolts_paid_flash_returns_hero_at_entry_then_exiles_without_dying() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let panther = paid(&mut g, p, "Black Panther, Vanguard", None);
        let villain = paid(&mut g, p, "Ultron Drone", None);
        assert_eq!(tokens(&g, p), 0);
        checked(&mut g, Action::Pass);
        let mana = g.state.players[p.other().index()].mana;
        let conspiracy = paid(&mut g, p.other(), "Thunderbolts Conspiracy", None);
        assert_ne!(g.state.players[p.other().index()].mana, mana);
        // A stolen observer watches the current controller's Villain.
        g.state.objects[conspiracy as usize].controller = p;
        let old = g.object_ref(villain);
        paid(&mut g, p, "Hour of Defeat", Some(Target::Card(villain)));
        assert_ne!(g.object_ref(villain), old);
        assert_eq!(g.object_view(villain).zone, Zone::Battlefield);
        assert_eq!(g.object_view(villain).controller, p);
        assert!(g.has_subtype(villain, "Villain") && g.has_subtype(villain, "Hero"));
        assert_eq!(g.counter_count(villain, CounterKind::Finality), 1);
        assert_eq!(tokens(&g, p), 1, "Hero must exist in the ETB event");
        assert_eq!(g.object_view(panther).zone, Zone::Battlefield);
        let deaths = g.state.creatures_died_this_turn;
        g.move_object(villain, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(villain).zone, Zone::Exile);
        assert_eq!(g.state.creatures_died_this_turn, deaths);
        assert_eq!(g.counter_count(villain, CounterKind::Finality), 0);
        assert!(!g.has_subtype(villain, "Hero"));
    }
}
#[test]
fn finality_thunderbolts_owner_return_source_departure_and_lasting_type_timeline() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let conspiracy = paid(&mut g, p, "Thunderbolts Conspiracy", None);
        let villain = named(&mut g, p.other(), "Ultron Drone", Zone::Battlefield);
        g.state.objects[villain as usize].controller = p;
        g.move_object(villain, Zone::Graveyard);
        g.move_object(conspiracy, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(villain).controller, p.other());
        assert!(g.has_subtype(villain, "Hero"));
        g.state.objects[villain as usize].controller = p;
        assert!(g.has_subtype(villain, "Hero"));
        let protection = paid(&mut g, p, "Witness Protection", Some(Target::Card(villain)));
        assert!(!g.has_subtype(villain, "Hero"));
        assert!(!g.printed_abilities_available(villain));
        g.move_object(protection, Zone::Graveyard);
        assert!(g.has_subtype(villain, "Hero"));
        g.move_object(villain, Zone::Hand);
        g.move_object(villain, Zone::Battlefield);
        settle(&mut g);
        assert!(!g.has_subtype(villain, "Hero"));
        assert_eq!(g.counter_count(villain, CounterKind::Finality), 0);
    }
}
#[test]
fn finality_thunderbolts_simultaneous_sources_duplicate_links_and_removed_counter_recur() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = paid(&mut g, p, "Thunderbolts Conspiracy", None);
        paid(&mut g, p, "Thunderbolts Conspiracy", None);
        let villain = named(&mut g, p, "Ultron Drone", Zone::Battlefield);
        let sources = g.death_trigger_sources();
        let events = g.move_objects_raw(&[first, villain], Zone::Graveyard);
        g.finish_zone_events(&events, &sources);
        settle(&mut g);
        assert_eq!(g.object_view(villain).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(villain, CounterKind::Finality), 1);
        let old = g.object_ref(villain);
        g.pay_counters(villain, &Counters::of(CounterKind::Finality, 1));
        g.move_object(villain, Zone::Graveyard);
        settle(&mut g);
        assert_ne!(g.object_ref(villain), old);
        assert_eq!(g.object_view(villain).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(villain, CounterKind::Finality), 1);
    }
}
#[test]
fn finality_thunderbolts_exact_graveyard_link_does_not_rebind_or_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Thunderbolts Conspiracy", None);
        let villain = named(&mut g, p, "Ultron Drone", Zone::Battlefield);
        g.move_object(villain, Zone::Graveyard);
        g.move_object(villain, Zone::Hand);
        g.move_object(villain, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(villain).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(villain, CounterKind::Finality), 0);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Ultron Drone", Zone::Battlefield);
        g.move_object(bear, Zone::Graveyard);
        g.move_object(enemy, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        // The predicate has no creature-only restriction (Kindred Villains qualify).
        let source = g.object_view(villain);
        let mut before = g.object_view(enemy);
        before.controller = p;
        before.kinds = 128;
        let mut event = g.move_object(enemy, Zone::Exile);
        event.before = before;
        assert!(g.return_death_matches(
            crate::cards::ReturnDeathFilter::ControlledSubtype("Villain"),
            &source,
            &event
        ));
    }
}
#[test]
fn finality_all_permanents_ignore_ability_loss_multiple_counts_and_non_graveyard_moves() {
    assert_eq!(CounterKind::Finality as usize, 29);
    assert_eq!(CounterKind::Vigilance as usize, 28);
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Forest", "Gilded Lotus"] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Battlefield);
            place(&mut g, id, 2);
            if name == "Bear Cub" {
                paid(&mut g, p, "Witness Protection", Some(Target::Card(id)));
            }
            g.pay_counters(id, &Counters::of(CounterKind::Finality, 1));
            g.move_object(id, Zone::Graveyard);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert_eq!(g.counter_count(id, CounterKind::Finality), 0);
            g.move_object(id, Zone::Battlefield);
            place(&mut g, id, 1);
            g.move_object(id, Zone::Hand);
            assert_eq!(g.object_view(id).zone, Zone::Hand);
            assert_eq!(g.counter_count(id, CounterKind::Finality), 0);
            g.move_object(id, Zone::Battlefield);
            place(&mut g, id, 1);
            g.pay_counters(id, &Counters::of(CounterKind::Finality, 1));
            g.move_object(id, Zone::Graveyard);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn finality_entry_doubling_and_source_suppression_apply_before_return_observers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Thunderbolts Conspiracy", None);
        paid(&mut g, p, "Doubling Season", None);
        let villain = named(&mut g, p, "Ultron Drone", Zone::Battlefield);
        g.move_object(villain, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(villain, CounterKind::Finality), 2);
        let mut movement = g.entry_move(villain, Zone::Battlefield, false);
        movement.return_modification = Some((
            crate::cards::ReturnKind::HeroFinality,
            g.object_ref(villain),
        ));
        assert!(Game::entry_choice_subtypes(&movement).has(movement.before.card, "Hero"));
        paid(&mut g, p, "Witness Protection", Some(Target::Card(villain)));
        g.move_object(villain, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(villain).zone, Zone::Exile);
    }
}
#[test]
fn finality_paid_sacrifice_pays_cost_draws_and_excludes_dies_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Thunderbolts Conspiracy", None);
        let villain = paid(&mut g, p, "Ultron Drone", None);
        place(&mut g, villain, 1);
        let deaths = g.state.creatures_died_this_turn;
        let draws = g.state.players[p.index()].cards_drawn_this_turn;
        sacrifice_spell(&mut g, p, villain);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(villain).zone, Zone::Exile);
        assert_eq!(g.state.creatures_died_this_turn, deaths);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 2);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn finality_competing_shuffle_uses_current_controller_choice_and_owned_checkpoint() {
    for p in [Player::P0, Player::P1] {
        for effect in [
            ZoneReplacementKind::Exile,
            ZoneReplacementKind::RevealShuffle,
        ] {
            let mut g = setup(p);
            let progenitus = named(&mut g, p.other(), "Progenitus", Zone::Battlefield);
            g.state.objects[progenitus as usize].controller = p;
            place(&mut g, progenitus, 1);
            sacrifice_spell(&mut g, p, progenitus);
            for _ in 0..20 {
                if g.legal_actions()
                    .contains(&Action::ChooseZoneReplacement { effect })
                {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                    .unwrap();
                checked(&mut g, a);
            }
            assert_eq!(g.actor(), Some(p));
            assert_eq!(g.object_view(progenitus).zone, Zone::Battlefield);
            reject(
                &mut g,
                Action::ChooseZoneReplacement {
                    effect: ZoneReplacementKind::Battlefield,
                },
            );
            checked(&mut g, Action::ChooseZoneReplacement { effect });
            pay(&mut g);
            settle(&mut g);
            assert_eq!(
                g.object_view(progenitus).zone,
                if effect == ZoneReplacementKind::Exile {
                    Zone::Exile
                } else {
                    Zone::Library
                }
            );
            assert_eq!(g.counter_count(progenitus, CounterKind::Finality), 0);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
            assert_eq!(g.state.creatures_died_this_turn, 0);
        }
    }
}
#[test]
fn finality_rest_in_peace_same_destination_does_not_offer_redundant_choice_or_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Thunderbolts Conspiracy", None);
        paid(&mut g, p, "Rest in Peace", None);
        let villain = named(&mut g, p, "Ultron Drone", Zone::Battlefield);
        place(&mut g, villain, 3);
        g.move_object(villain, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(villain).zone, Zone::Exile);
        assert!(g.state.pending_triggers.is_empty());
        assert_eq!(g.state.creatures_died_this_turn, 0);
    }
}
#[test]
fn finality_cold_storage_public_counter_wire_and_owned_snapshot_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Forest", Zone::Battlefield);
        let parent = g.clone();
        let bytes = g.state_bytes();
        place(&mut g, id, 1);
        assert!(g.state.objects.ptr_eq(&parent.state.objects));
        assert!(g.state_bytes() > bytes);
        let snapshot = g.snapshot();
        let counters = g.object_view(id).counters;
        let wire = serde_json::to_string(&counters).unwrap();
        assert_eq!(serde_json::from_str::<Counters>(&wire).unwrap(), counters);
        for viewer in [p, p.other()] {
            assert_eq!(
                g.observe(viewer)
                    .objects
                    .iter()
                    .find(|o| o.id == id)
                    .unwrap()
                    .counters
                    .get(CounterKind::Finality),
                1
            )
        }
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        branch.move_object(id, Zone::Graveyard);
        assert_eq!(branch.object_view(id).zone, Zone::Exile);
        assert_eq!(g.snapshot(), snapshot);
        g.restore(&snapshot).unwrap();
        g.pay_counters(id, &Counters::of(CounterKind::Finality, 1));
        assert!(g.state.named_counters.is_none());
        assert_eq!(parent.counter_count(id, CounterKind::Finality), 0);
    }
}

#[test]
fn finality_thunderbolts_token_villain_triggers_but_cannot_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Thunderbolts Conspiracy", None);
        let drone = paid(&mut g, p, "Ultron Drone", None);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: drone,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token() && o.controller == p)
            .map(|(id, _)| id as CardId)
            .unwrap();
        assert!(g.has_subtype(token, "Villain"));
        g.move_object(token, Zone::Graveyard);
        g.state_based_actions();
        settle(&mut g);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
        assert_eq!(tokens(&g, p), 0);
    }
}

#[test]
fn finality_thunderbolts_provider_return_assigns_deferred_type_timestamp_and_preserves_later_order()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let conspiracy = paid(&mut g, p, "Thunderbolts Conspiracy", None);
        let provider = paid(&mut g, p, "Doc Ock, Sinister Scientist", None);
        let old = g.object_ref(provider);
        let earlier = g.next_role_timestamp();
        paid(&mut g, p, "Hour of Defeat", Some(Target::Card(provider)));
        let returned = g.object_ref(provider);
        assert_ne!(returned, old);
        assert_eq!(g.object_view(provider).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(provider, CounterKind::Finality), 1);
        assert!(g.has_subtype(provider, "Hero"));
        let type_time = |game: &Game| {
            game.state
                .roles
                .as_ref()
                .unwrap()
                .creature_types
                .iter()
                .find(|row| row.recipient == returned && row.types == ["Hero"])
                .unwrap()
                .timestamp
        };
        let provider_time = |game: &Game| {
            game.state
                .roles
                .as_ref()
                .unwrap()
                .base_stats
                .iter()
                .find(|row| row.source == returned)
                .unwrap()
                .timestamp
        };
        let returned_time = type_time(&g);
        assert!(
            returned_time > earlier,
            "a deferred Hero effect cannot retain timestamp zero"
        );
        assert_eq!(
            returned_time,
            provider_time(&g),
            "the returning provider and pending subtype share its assigned entry timestamp"
        );
        assert!(!g.state.roles.as_ref().unwrap().deferred_timestamps);
        // Refreshing the provider later must not move a lasting return grant.
        g.assign_static_source_timestamp(returned);
        assert!(provider_time(&g) > returned_time);
        assert_eq!(type_time(&g), returned_time);
        let aura = paid(
            &mut g,
            p,
            "Witness Protection",
            Some(Target::Card(provider)),
        );
        assert!(!g.has_subtype(provider, "Hero"));
        assert_eq!(type_time(&g), returned_time);
        g.move_object(aura, Zone::Graveyard);
        g.move_object(conspiracy, Zone::Graveyard);
        assert!(g.has_subtype(provider, "Hero"));
        assert_eq!(type_time(&g), returned_time);
        g.move_object(provider, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(provider).zone, Zone::Exile);
        assert!(
            g.state
                .roles
                .as_ref()
                .unwrap()
                .creature_types
                .iter()
                .all(|row| row.recipient != returned)
        );
    }
}

#[test]
fn finality_return_modifiers_share_chosen_entry_order_for_plain_and_provider_creatures() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Thunderbolts Conspiracy", Zone::Battlefield);
            let ids = [
                named(&mut g, p, "Doc Ock, Sinister Scientist", Zone::Graveyard),
                named(&mut g, p, "Ultron Drone", Zone::Graveyard),
            ];
            let earlier = g.next_role_timestamp();
            let moves = ids
                .into_iter()
                .map(|id| {
                    let mut movement = g.entry_move(id, Zone::Battlefield, false);
                    movement.effect_counter = Some((CounterKind::Finality, 1));
                    movement.return_modification =
                        Some((crate::cards::ReturnKind::HeroFinality, g.object_ref(source)));
                    movement
                })
                .collect();
            g.stage_entries(moves, None);
            g.advance_entries();
            assert!(matches!(g.state.pending, Pending::EntryTimestamp { .. }));
            let ordered = if reverse { [ids[1], ids[0]] } else { ids };
            let first = g.object_ref(ordered[0]);
            checked(&mut g, Action::ChooseEntryTimestamp { object: first });
            settle(&mut g);
            let roles = g.state.roles.as_ref().unwrap();
            let times = ordered.map(|id| {
                let row = roles
                    .creature_types
                    .iter()
                    .find(|row| row.recipient == g.object_ref(id) && row.types == ["Hero"])
                    .unwrap();
                assert!(g.has_subtype(id, "Hero"));
                assert_eq!(g.counter_count(id, CounterKind::Finality), 1);
                row.timestamp
            });
            assert!(times[0] > earlier && times[1] > times[0]);
            let base = roles
                .base_stats
                .iter()
                .find(|row| row.source == g.object_ref(ids[0]) && row.static_source)
                .unwrap();
            assert_eq!(base.timestamp, times[usize::from(reverse)]);
        }
    }
}
