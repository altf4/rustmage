//! Four complete Vehicles, including real tagged mana and additional costs.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("bounded resolution");
}
fn reach(g: &mut Game, wanted: impl Fn(&Action) -> bool) {
    for _ in 0..24 {
        g.prepare_triggers();
        if g.legal_actions().iter().any(&wanted) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("choice missing {:?}", g.state.pending);
}
fn crew(g: &mut Game, source: CardId, index: u16, pilot: CardId) {
    act(g, activation(source, index));
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
    let object = g.object_ref(pilot);
    act(g, Action::ChooseCrewer { object });
    act(g, Action::FinishCrew);
    drain(g);
}
#[test]
fn boommobile_entry_chooses_exactly_four_of_one_color_and_that_mana_cannot_cast_any_spell() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Boommobile", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 2];
            cast(&mut g, source, None);
            reach(&mut g, |a| matches!(a, Action::ChooseMana { .. }));
            reject(
                &mut g,
                Action::ChooseMana {
                    production: [1, 1, 1, 1, 0, 0],
                },
            );
            reject(
                &mut g,
                Action::ChooseMana {
                    production: [0, 0, 0, 0, 0, 4],
                },
            );
            let mut production = [0; 6];
            production[color] = 4;
            act(&mut g, Action::ChooseMana { production });
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].mana, production);
            assert!(!g.is_creature(source));
            let artifact = put(&mut g, p, "Short Bow", Zone::Hand);
            let spell = put(&mut g, p, "Shock", Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: artifact });
            reject(&mut g, Action::AnnounceCast { card: spell });
            let quality = g.state.mana_lots.as_ref().unwrap().pools[p.index()][0].quality;
            assert!(!quality.eligible(mana_lots::Purpose::Other));
            assert!(
                quality.eligible(mana_lots::Purpose::Activation(mana_lots::Source::in_world(
                    &g, source
                )))
            );
        }
    }
}
#[test]
fn boommobile_restricted_red_mana_pays_its_x_exhaust_and_counter_matters_when_crewed() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Boommobile", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 2, 0, 2];
    cast(&mut g, source, None);
    reach(&mut g, |a| matches!(a, Action::ChooseMana { .. }));
    act(
        &mut g,
        Action::ChooseMana {
            production: [0, 0, 0, 4, 0, 0],
        },
    );
    drain(&mut g);
    act(&mut g, activation(source, 0));
    act(&mut g, Action::ChooseX { value: 1 });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    pay(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    drain(&mut g);
    assert_eq!(g.state.players[1].life, 19);
    assert_eq!(g.object_view(source).counters.plus_one, 1);
    assert!(!g.is_creature(source));
    reject(&mut g, activation(source, 0));
    let pilot = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    crew(&mut g, source, 1, pilot);
    assert_eq!(stats(&g, source), (6, 6));
    g.finish_cleanup();
    assert!(!g.is_creature(source));
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.players[0].mana = [8; 6];
    reject(&mut g, activation(source, 0));
}
#[test]
fn boommobile_x_zero_and_creature_or_planeswalker_targets_are_independent_of_source_survival() {
    for x in [0, 2] {
        for walker in [false, true] {
            let p = Player::P0;
            let mut g = scenario(p);
            let source = put(&mut g, p, "Boommobile", Zone::Battlefield);
            let target = put(
                &mut g,
                p.other(),
                if walker {
                    "Chandra, Spark Hunter"
                } else {
                    "Aegis Turtle"
                },
                Zone::Battlefield,
            );
            if walker {
                g.state.objects[target as usize].counters.loyalty = 8;
            }
            act(&mut g, activation(source, 0));
            act(&mut g, Action::ChooseX { value: x });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            g.move_object(source, Zone::Graveyard);
            drain(&mut g);
            assert_eq!(g.object_view(source).counters.plus_one, 0);
            if walker {
                assert_eq!(g.object_view(target).counters.loyalty, 8 - u64::from(x));
            } else {
                assert_eq!(g.object_view(target).damage, i128::from(x));
            }
        }
    }
}
#[test]
fn demonic_junker_affinity_counts_only_controlled_artifacts_reduces_generic_to_zero_and_keeps_black()
 {
    for count in [0, 2, 7] {
        let p = Player::P0;
        let mut g = scenario(p);
        for _ in 0..count {
            put(&mut g, p, "Short Bow", Zone::Battlefield);
        }
        put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, p, "Demonic Junker", Zone::Hand);
        let generic = 6u32.saturating_sub(count);
        g.state.players[0].mana = [0, 0, 0, 0, 0, generic];
        reject(&mut g, Action::AnnounceCast { card: source });
        g.state.players[0].mana = [0, 0, 1, 0, 0, generic];
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::FinishTargets));
        act(&mut g, Action::FinishTargets);
        drain(&mut g);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert_eq!(g.object_view(source).counters.plus_one, 0);
        assert!(!g.is_creature(source));
    }
}
#[test]
fn demonic_junker_targets_at_most_one_per_player_and_rewards_actual_own_destruction_once() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = scenario(p);
            let ours = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let another = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let theirs = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = put(&mut g, p, "Demonic Junker", Zone::Hand);
            cast(&mut g, source, None);
            reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
            let ids = if reverse {
                [theirs, ours]
            } else {
                [ours, theirs]
            };
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(ids[0]),
                },
            );
            if !reverse {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(another),
                    },
                );
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(ids[1]),
                },
            );
            drain(&mut g);
            assert_eq!(g.object_view(ours).zone, Zone::Graveyard);
            assert_eq!(g.object_view(theirs).zone, Zone::Graveyard);
            assert_eq!(g.object_view(source).counters.plus_one, 2);
            crew(&mut g, source, 0, another);
            assert_eq!(stats(&g, source), (6, 5));
        }
    }
}
#[test]
fn demonic_junker_does_not_reward_indestructible_or_stale_own_targets_but_still_destroys_legal_other()
 {
    for stale in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let ours = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let theirs = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        if !stale {
            g.state.objects[ours as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        }
        let source = put(&mut g, p, "Demonic Junker", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        for id in [ours, theirs] {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        if stale {
            g.move_object(ours, Zone::Hand);
            g.move_object(ours, Zone::Battlefield);
        }
        drain(&mut g);
        assert_eq!(g.object_view(ours).zone, Zone::Battlefield);
        assert_eq!(g.object_view(theirs).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).counters.plus_one, 0);
    }
}
#[test]
fn gastal_thrillroller_actual_entry_animates_with_haste_trample_then_crew_two_is_needed_next_turn()
{
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Gastal Thrillroller", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 1, 0, 2];
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert!(g.is_creature(source));
    assert_eq!(stats(&g, source), (4, 2));
    assert!(
        g.keywords(source)
            .contains(Keywords::HASTE.union(Keywords::TRAMPLE))
    );
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: source });
    act(&mut g, Action::FinishAttackers);
    drain(&mut g);
    g.finish_cleanup();
    assert!(!g.is_creature(source));
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.objects[source as usize].tapped = false;
    let pilot = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    crew(&mut g, source, 0, pilot);
    assert!(g.is_creature(source));
    assert_eq!(stats(&g, source), (4, 2));
}
#[test]
fn gastal_thrillroller_sorcery_graveyard_return_discards_a_card_enters_with_finality_and_exiles_on_death()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Gastal Thrillroller", Zone::Graveyard);
        reject(&mut g, activation(source, 1));
        let card = put(&mut g, p, "Mountain", Zone::Hand);
        g.state.step = Step::Upkeep;
        reject(&mut g, activation(source, 1));
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        act(&mut g, activation(source, 1));
        let object = g.object_ref(card);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(source, CounterKind::Finality), 1);
        assert!(!g.object_view(source).tapped);
        assert!(g.is_creature(source));
        let spell = put(&mut g, p, "Lightning Strike", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        cast(&mut g, spell, Some(Target::Card(source)));
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}
#[test]
fn spire_mechcycle_exhaust_cost_is_other_untapped_controlled_mount_or_vehicle_and_union_count_is_live()
 {
    for choose_mount in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Spire Mechcycle", Zone::Battlefield);
        reject(&mut g, activation(source, 0));
        let mount = put(&mut g, p, "Gilded Ghoda", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let ordinary = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Gilded Ghoda", Zone::Battlefield);
        let dual = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let packet = g.activation_packet(p, dual, 0);
        g.add_creature_types(&packet, Subject::Source, &["Mount"]);
        assert!(g.has_subtype(dual, "Mount") && g.has_subtype(dual, "Vehicle"));
        g.state.objects[dual as usize].tapped = true;
        act(&mut g, activation(source, 0));
        for id in [source, ordinary, foreign, dual] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let selected = if choose_mount { mount } else { vehicle };
        let object = g.object_ref(selected);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert!(g.object_view(selected).tapped);
        assert!(!g.object_view(source).tapped);
        g.move_object(if choose_mount { vehicle } else { mount }, Zone::Hand);
        put(&mut g, p, "Gilded Ghoda", Zone::Battlefield);
        drain(&mut g);
        assert_eq!(
            g.object_view(source).counters.plus_one,
            3,
            "selected + dual once + late mount, excluding source"
        );
        assert_eq!(stats(&g, source), (8, 7));
        assert!(g.is_creature(source));
        assert!(g.keywords(source).contains(Keywords::HASTE));
        g.finish_cleanup();
        assert!(g.is_creature(source));
        assert_eq!(stats(&g, source), (8, 7));
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        reject(&mut g, activation(source, 0));
    }
}
#[test]
fn spire_mechcycle_ordinary_crew_two_is_temporary_and_owned_exhaust_payment_branches_agree() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Spire Mechcycle", Zone::Battlefield);
    let pilot = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    crew(&mut g, source, 1, pilot);
    assert!(g.is_creature(source));
    assert_eq!(stats(&g, source), (5, 4));
    g.finish_cleanup();
    assert!(!g.is_creature(source));
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    let mount = put(&mut g, p, "Gilded Ghoda", Zone::Battlefield);
    act(&mut g, activation(source, 0));
    let object = g.object_ref(mount);
    act(&mut g, Action::ChooseAdditionalCost { object });
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let a = raw
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    let at = indexed
        .legal_actions()
        .iter()
        .position(|x| *x == a)
        .unwrap();
    act(&mut raw, a);
    indexed.apply_index(p, indexed.revision(), at).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    drop(g);
    pay(&mut indexed);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(stats(&indexed, source), (6, 5));
}
