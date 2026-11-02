//! Whole-card speed enchantments and controller-life Vehicle.
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
fn grant(g: &Game, card: CardId) -> Action {
    Action::AnnounceSourceGrant {
        source: g.object_ref(card),
        ability: 0,
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
#[test]
fn four_speed_enchantments_start_engines_on_actual_entry_and_keep_existing_speed() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Hour of Victory",
            "Kickoff Celebrations",
            "Momentum Breaker",
            "Outpace Oblivion",
        ] {
            for old in [0, 3] {
                let mut g = scenario(p);
                g.state.players[p.index()].speed = old;
                let source = put(&mut g, p, name, Zone::Hand);
                cast(&mut g, source, None);
                for _ in 0..30 {
                    g.prepare_triggers();
                    if g.state.pending == Pending::Priority
                        && g.state.stack.is_empty()
                        && g.state.pending_triggers.is_empty()
                    {
                        break;
                    }
                    choose(&mut g, |a| {
                        matches!(
                            a,
                            Action::Pass
                                | Action::PutTrigger { .. }
                                | Action::FinishTargets
                                | Action::DeclineEffectPayment
                        )
                    });
                }
                assert_eq!(g.object_view(source).zone, Zone::Battlefield);
                assert_eq!(g.state.players[p.index()].speed, old.max(1), "{name}");
                assert_eq!(g.state.players[p.other().index()].speed, 0);
                g.move_object(source, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].speed, old.max(1));
            }
        }
    }
}
#[test]
fn hour_of_victory_creates_zombie_and_its_max_speed_sorcery_search_is_required_and_private() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Hour of Victory", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        let z: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(i, o)| {
                o.zone == Zone::Battlefield && o.is_token() && g.has_subtype(*i as CardId, "Zombie")
            })
            .map(|(i, _)| i as CardId)
            .collect();
        assert_eq!(z.len(), 1);
        assert_eq!(stats(&g, z[0]), (2, 2));
        assert_eq!(g.object_view(z[0]).colors, 4);
        assert!(!g.object_view(z[0]).tapped);
        let request = grant(&g, source);
        reject(&mut g, request);
        g.state.players[p.index()].speed = 4;
        g.state.step = Step::Upkeep;
        let request = grant(&g, source);
        reject(&mut g, request);
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        let found = put(&mut g, p, "Shock", Zone::Library);
        let foreign = put(&mut g, p.other(), "Shock", Zone::Library);
        let request = grant(&g, source);
        act(&mut g, request);
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.players[p.index()].speed = 0;
        reach(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
        reject(&mut g, Action::FinishSearch);
        reject(&mut g, Action::ChooseSearch { card: foreign });
        let hidden = g.object_ref(found);
        assert!(!g.observe(p.other()).known_hand.contains(&hidden));
        let before = g.state.rng;
        act(&mut g, Action::ChooseSearch { card: found });
        act(&mut g, Action::FinishSearch);
        drain(&mut g);
        assert_eq!(g.object_view(found).zone, Zone::Hand);
        assert!(
            !g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(found))
        );
        assert_ne!(g.state.rng, before);
    }
}
#[test]
fn kickoff_optional_discard_draws_two_only_when_paid_and_max_speed_freezes_creature_vehicle_union()
{
    for accept in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let material = put(&mut g, p, "Mountain", Zone::Hand);
        let source = put(&mut g, p, "Kickoff Celebrations", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::DeclineEffectPayment));
        if accept {
            act(&mut g, Action::Discard { card: material });
        } else {
            act(&mut g, Action::DeclineEffectPayment);
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(material).zone,
            if accept { Zone::Graveyard } else { Zone::Hand }
        );
        assert_eq!(
            g.state.players[0].cards_drawn_this_turn,
            if accept { 2 } else { 0 }
        );
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let artifact = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let request = grant(&g, source);
        reject(&mut g, request);
        g.state.players[0].speed = 4;
        g.state.step = Step::Upkeep;
        let request = grant(&g, source);
        act(&mut g, request);
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(&mut g);
        }
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        g.state.players[0].speed = 0;
        drain(&mut g);
        for id in [bear, vehicle] {
            assert!(g.keywords(id).contains(Keywords::HASTE));
        }
        for id in [artifact, enemy] {
            assert!(!g.keywords(id).contains(Keywords::HASTE));
        }
        let late = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(!g.keywords(late).contains(Keywords::HASTE));
        g.finish_cleanup();
        for id in [bear, vehicle] {
            assert!(!g.keywords(id).contains(Keywords::HASTE));
        }
    }
}
#[test]
fn momentum_breaker_opponent_must_sacrifice_creature_or_vehicle_instead_of_discarding() {
    for p in [Player::P0, Player::P1] {
        for vehicle in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(
                &mut g,
                p.other(),
                if vehicle {
                    "Clamorous Ironclad"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            let invalid = put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
            let hand = put(&mut g, p.other(), "Shock", Zone::Hand);
            let source = put(&mut g, p, "Momentum Breaker", Zone::Hand);
            cast(&mut g, source, None);
            reach(&mut g, |a| matches!(a, Action::Sacrifice { .. }));
            assert_eq!(g.actor(), Some(p.other()));
            for id in [own, invalid] {
                let object = g.object_ref(id);
                reject(&mut g, Action::Sacrifice { object });
            }
            reject(&mut g, Action::Discard { card: hand });
            let object = g.object_ref(enemy);
            act(&mut g, Action::Sacrifice { object });
            drain(&mut g);
            assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
            assert_eq!(g.object_view(hand).zone, Zone::Hand);
            assert_eq!(g.object_view(invalid).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn momentum_breaker_fallback_is_discard_and_life_ability_samples_its_controllers_speed_at_resolution()
 {
    for speed in [0, 1, 4] {
        let p = Player::P1;
        let mut g = scenario(p);
        let hand = put(&mut g, p.other(), "Shock", Zone::Hand);
        let source = put(&mut g, p, "Momentum Breaker", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::Discard { .. }));
        assert_eq!(g.actor(), Some(p.other()));
        act(&mut g, Action::Discard { card: hand });
        drain(&mut g);
        assert_eq!(g.object_view(hand).zone, Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        g.state.players[p.index()].speed = 2;
        act(&mut g, activation(source, 0));
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.players[p.index()].speed = speed;
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20 + i128::from(speed));
    }
}
#[test]
fn outpace_oblivion_optional_entry_hits_creature_or_planeswalker_and_excludes_players_and_artifacts()
 {
    for target_kind in [0, 1, 2] {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = put(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let walker = put(
            &mut g,
            p.other(),
            "Chandra, Spark Hunter",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 8;
        let artifact = put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let source = put(&mut g, p, "Outpace Oblivion", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| {
            matches!(a, Action::ChooseTarget { .. } | Action::FinishTargets)
        });
        for target in [Target::Card(artifact), Target::Player(p.other())] {
            reject(&mut g, Action::ChooseTarget { target });
        }
        match target_kind {
            0 => act(&mut g, Action::FinishTargets),
            1 => act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            ),
            _ => act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(walker),
                },
            ),
        };
        drain(&mut g);
        assert_eq!(
            g.object_view(bear).zone,
            if target_kind == 1 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(walker).counters.loyalty,
            if target_kind == 2 { 3 } else { 8 }
        );
        assert_eq!(g.state.players[1].life, 20);
    }
}
#[test]
fn outpace_oblivion_sacrifice_damages_each_nonmax_player_using_independent_resolution_speeds() {
    for p in [Player::P0, Player::P1] {
        for (ours, theirs) in [(0, 0), (3, 4), (4, 3), (4, 4)] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Outpace Oblivion", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            g.state.players[p.index()].speed = 4;
            g.state.players[p.other().index()].speed = 4;
            act(&mut g, activation(source, 0));
            pay(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.state.players[p.index()].speed = ours;
            g.state.players[p.other().index()].speed = theirs;
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if ours == 4 { 20 } else { 18 }
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if theirs == 4 { 20 } else { 18 }
            );
        }
    }
}
#[test]
fn last_ride_live_life_reduction_and_paid_life_draw_follow_controller_and_crew_two() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.players[p.index()].life = 10;
        let source = put(&mut g, p, "The Last Ride", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(stats(&g, source), (3, 3));
        assert!(!g.is_creature(source));
        let crewer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        act(&mut g, activation(source, 1));
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(&mut g);
        }
        let object = g.object_ref(crewer);
        act(&mut g, Action::ChooseCrewer { object });
        act(&mut g, Action::FinishCrew);
        drain(&mut g);
        assert!(g.is_creature(source));
        assert_eq!(stats(&g, source), (3, 3));
        g.state.players[p.index()].life = 11;
        assert_eq!(stats(&g, source), (2, 2));
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        act(&mut g, activation(source, 0));
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].life, 9);
        assert_eq!(stats(&g, source), (4, 4));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        g.state.players[p.other().index()].life = 6;
        g.state.objects[source as usize].controller = p.other();
        assert_eq!(stats(&g, source), (7, 7));
        g.state.players[p.other().index()].life = 1;
        g.state.active = p.other();
        g.state.priority = p.other();
        reject(&mut g, activation(source, 0));
    }
}
