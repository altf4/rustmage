//! Additional S–Z definitions: whole printed outcomes against accepted engine85.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
}
fn advance(g: &mut Game, targets: &[Target], done: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if done(g) {
            return;
        }
        let actions = g.legal_actions();
        if let Some(a) = targets
            .iter()
            .map(|&target| Action::ChooseTarget { target })
            .find(|a| actions.contains(a))
        {
            act(g, a);
            continue;
        }
        let a = actions
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishStackTargets
                        | Action::ChooseReplacement { .. }
                )
            })
            .unwrap_or_else(|| panic!("explicit choice needed: {:?}", g.state.pending));
        act(g, a);
    }
    panic!("96-action actual-card budget");
}
fn drain(g: &mut Game) {
    advance(g, &[], idle);
}
fn drain_to(g: &mut Game, targets: &[Target]) {
    advance(g, targets, idle);
}
fn candidate(g: &mut Game, p: Player, name: &str) -> CardId {
    put(g, p, name, Zone::Hand)
}
fn cast_candidate(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = candidate(g, p, name);
    cast(g, id, None);
    drain(g);
    id
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.object_view(id as CardId).has_subtype(subtype))
            .then_some(id as CardId)
        })
        .collect()
}
fn entrants(g: &Game, name: &str) -> Vec<CardId> {
    let card = Card::from_name(name).unwrap();
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.card == card).then_some(id as CardId)
        })
        .collect()
}
fn reset_main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
}
fn step(g: &mut Game, step: Step) {
    g.begin_step(step);
    g.advance_replacements(false);
    g.give_priority();
}
fn activation(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

#[test]
fn sage_copies_after_any_earlier_own_spell_but_not_as_first_and_copies_are_tokens() {
    for prior in [0, 1, 2] {
        let p = Player::P0;
        let mut g = scenario(p);
        for _ in 0..prior {
            let id = put(&mut g, p, "Grizzly Bears", Zone::Hand);
            cast(&mut g, id, None);
            drain(&mut g);
        }
        let source = cast_candidate(&mut g, p, "Sage of the Skies");
        let all = entrants(&g, "Sage of the Skies");
        assert_eq!(all.len(), if prior == 0 { 1 } else { 2 });
        assert_eq!(
            all.iter()
                .filter(|&&id| g.object_view(id).is_token())
                .count(),
            usize::from(prior != 0)
        );
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, prior + 1);
        for id in all {
            assert_eq!(stats(&g, id), (2, 3));
            assert!(
                g.keywords(id)
                    .contains(Keywords::FLYING.union(Keywords::LIFELINK))
            );
        }
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}
#[test]
fn stormscale_actual_storm_tokens_and_other_owned_dragon_anthem() {
    for prior in [0, 2] {
        let p = Player::P1;
        let mut g = scenario(p);
        let own = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        for _ in 0..prior {
            let id = put(&mut g, p, "Grizzly Bears", Zone::Hand);
            cast(&mut g, id, None);
            drain(&mut g);
        }
        let source = cast_candidate(&mut g, p, "Stormscale Scion");
        let all = entrants(&g, "Stormscale Scion");
        assert_eq!(all.len(), prior + 1);
        assert_eq!(
            all.iter()
                .filter(|&&id| g.object_view(id).is_token())
                .count(),
            prior
        );
        for &id in &all {
            assert_eq!(stats(&g, id), ((4 + prior) as i128, (4 + prior) as i128));
            assert!(g.keywords(id).contains(Keywords::FLYING));
        }
        assert_eq!(stats(&g, own), ((6 + prior) as i128, (6 + prior) as i128));
        assert_eq!(stats(&g, enemy), (5, 5));
        assert_eq!(stats(&g, bear), (2, 2));
        g.move_object(source, Zone::Graveyard);
        assert_eq!(stats(&g, own), ((5 + prior) as i128, (5 + prior) as i128));
    }
}
#[test]
fn sultai_monument_all_three_basic_filters_search_shuffle_and_exact_activation() {
    for (p, basic) in [
        (Player::P0, "Swamp"),
        (Player::P1, "Forest"),
        (Player::P0, "Island"),
    ] {
        let mut g = scenario(p);
        let found = put(&mut g, p, basic, Zone::Library);
        let excluded = put(&mut g, p, "Plains", Zone::Library);
        let source = candidate(&mut g, p, "Sultai Monument");
        cast(&mut g, source, None);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert!(g.observe(p).search_choice.is_some());
        reject(&mut g, Action::ChooseSearch { card: excluded });
        let before_shuffle = g.state.rng;
        act(&mut g, Action::ChooseSearch { card: found });
        if g.legal_actions().contains(&Action::FinishSearch) {
            act(&mut g, Action::FinishSearch);
        }
        drain(&mut g);
        assert_eq!(g.object_view(found).zone, Zone::Hand);
        assert_ne!(
            g.state.rng, before_shuffle,
            "search shuffles the remaining library"
        );
        g.state.step = Step::Upkeep;
        reject(&mut g, activation(source));
        g.state.step = Step::Main1;
        g.state.objects[source as usize].tapped = true;
        reject(&mut g, activation(source));
        g.state.objects[source as usize].tapped = false;
        g.state.players[p.index()].mana = [0, 1, 1, 0, 1, 2];
        act(&mut g, activation(source));
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        drain(&mut g);
        let zombies = tokens(&g, p, "Zombie");
        assert_eq!(zombies.len(), 2);
        for id in zombies {
            assert_eq!(stats(&g, id), (2, 2));
            assert_eq!(g.object_view(id).colors, 4);
            assert!(g.object_view(id).has_subtype("Druid"));
            assert!(!g.object_view(id).tapped);
        }
    }
}
#[test]
fn sibsig_forced_private_pick_and_owned_raw_index_restore_leave_other_card_in_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        let first = put(&mut parent, p, "Island", Zone::Library);
        let second = put(&mut parent, p, "Mountain", Zone::Library);
        let source = candidate(&mut parent, p, "Sibsig Appraiser");
        cast(&mut parent, source, None);
        advance(&mut parent, &[], |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        assert!(parent.observe(p).library_selection.is_some());
        assert!(parent.observe(p.other()).library_selection.is_none());
        assert_eq!(stats(&parent, source), (2, 1));
        reject(&mut parent, Action::FinishLibrarySelection);
        let action = Action::SelectLibrary { card: first };
        let saved = parent.snapshot();
        let revision = parent.revision();
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        assert_eq!(parent.snapshot(), saved);
        drop(parent);
        assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
        assert_eq!(raw.snapshot(), saved);
        reject(&mut raw, Action::SelectLibrary { card: source });
        act(&mut raw, action.clone());
        indexed.apply_index(p, revision, index).unwrap();
        for g in [&mut raw, &mut indexed] {
            if g.legal_actions().contains(&Action::FinishLibrarySelection) {
                act(g, Action::FinishLibrarySelection);
            }
            drain(g);
            assert_eq!(g.object_view(first).zone, Zone::Hand);
            assert_eq!(g.object_view(second).zone, Zone::Graveyard);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(
            serde_json::to_value(raw.observe(p)).unwrap(),
            serde_json::to_value(indexed.observe(p)).unwrap()
        );
        assert_eq!(
            serde_json::to_value(raw.observe(p.other())).unwrap(),
            serde_json::to_value(indexed.observe(p.other())).unwrap()
        );
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        if indexed
            .legal_actions()
            .contains(&Action::FinishLibrarySelection)
        {
            act(&mut indexed, Action::FinishLibrarySelection);
        }
        drain(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
#[test]
fn teeming_creates_two_white_soldiers_and_returns_only_for_own_dragon() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = cast_candidate(&mut g, p, "Teeming Dragonstorm");
    let soldiers = tokens(&g, p, "Soldier");
    assert_eq!(soldiers.len(), 2);
    for id in soldiers {
        assert_eq!(stats(&g, id), (2, 2));
        assert_eq!(g.object_view(id).colors, 1);
    }
    let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
    g.move_object(enemy, Zone::Battlefield);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    let own = put(&mut g, p, "Shivan Dragon", Zone::Hand);
    g.move_object(own, Zone::Battlefield);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Hand);
    reset_main(&mut g, p);
    g.state.players[p.index()].mana = [8; 6];
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(tokens(&g, p, "Soldier").len(), 4);
}
#[test]
fn kirin_flash_optional_bounce_obeys_other_owned_nonland_and_draws_only_for_tokens() {
    for mode in 0..3 {
        let p = Player::P1;
        let mut g = scenario(p);
        let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let token = put(
            &mut g,
            p,
            "@token/626a64b3de0602628359e30d",
            Zone::Battlefield,
        );
        let source = candidate(&mut g, p, "Sunpearl Kirin");
        g.state.active = p.other();
        g.state.step = Step::Upkeep;
        let hand = g.hand_count(p);
        cast(&mut g, source, None);
        advance(&mut g, &[], |g| {
            g.object_view(source).zone == Zone::Battlefield
                && matches!(g.state.pending, Pending::Targets { .. })
        });
        for id in [land, enemy, source] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        if mode != 0 {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(if mode == 1 { own } else { token }),
                },
            );
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        drain(&mut g);
        assert_eq!(stats(&g, source), (2, 1));
        assert!(
            g.keywords(source)
                .contains(Keywords::FLASH.union(Keywords::FLYING))
        );
        assert_eq!(
            g.hand_count(p),
            hand - 1 + usize::from(mode == 1) + usize::from(mode == 2)
        );
        assert_eq!(
            g.object_view(own).zone,
            if mode == 1 {
                Zone::Hand
            } else {
                Zone::Battlefield
            }
        );
        if mode == 2 {
            assert_ne!(g.object_view(token).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn sarkhan_optional_behold_hand_battlefield_or_decline_produces_actual_treasure() {
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let witness = put(
            &mut g,
            p,
            "Shivan Dragon",
            if mode == 1 {
                Zone::Battlefield
            } else {
                Zone::Hand
            },
        );
        let source = candidate(&mut g, p, "Sarkhan, Dragon Ascendant");
        cast(&mut g, source, None);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::HandCardChoice { .. })
        });
        assert!(g.observe(p.other()).hand_choice.is_none());
        let reference = g.object_ref(witness);
        act(
            &mut g,
            Action::ChooseHandCard {
                object: if mode == 2 { None } else { Some(reference) },
            },
        );
        drain(&mut g);
        assert_eq!(tokens(&g, p, "Treasure").len(), usize::from(mode != 2));
        assert_eq!(stats(&g, source), (2, 2));
        assert!(!g.keywords(source).contains(Keywords::FLYING));
    }
}
#[test]
fn sarkhan_dragon_entry_counter_and_additional_type_flying_expire_but_counter_remains() {
    let p = Player::P1;
    let mut g = scenario(p);
    let source = candidate(&mut g, p, "Sarkhan, Dragon Ascendant");
    cast(&mut g, source, None);
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::HandCardChoice { .. })
    });
    act(&mut g, Action::ChooseHandCard { object: None });
    drain(&mut g);
    let non = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    g.move_object(non, Zone::Battlefield);
    drain(&mut g);
    assert_eq!(stats(&g, source), (2, 2));
    let dragon = put(&mut g, p, "Shivan Dragon", Zone::Hand);
    g.move_object(dragon, Zone::Battlefield);
    drain(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(stats(&g, source), (3, 3));
    assert!(g.keywords(source).contains(Keywords::FLYING));
    let view = g.object_view(source);
    assert!(view.has_subtype("Dragon") && view.has_subtype("Human") && view.has_subtype("Druid"));
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    assert!(!g.object_view(source).has_subtype("Dragon"));
    assert!(!g.keywords(source).contains(Keywords::FLYING));
    assert_eq!(stats(&g, source), (3, 3));
}
#[test]
fn shocking_sharpshooter_other_own_creature_entries_target_opponent_and_ignore_self_enemy_land() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = cast_candidate(&mut g, p, "Shocking Sharpshooter");
    assert_eq!(stats(&g, source), (1, 3));
    assert!(g.keywords(source).contains(Keywords::REACH));
    assert_eq!(g.state.players[1].life, 20);
    for (owner, name) in [(p.other(), "Grizzly Bears"), (p, "Forest")] {
        let id = put(&mut g, owner, name, Zone::Hand);
        g.move_object(id, Zone::Battlefield);
        drain(&mut g);
    }
    assert_eq!(g.state.players[1].life, 20);
    let own = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    g.move_object(own, Zone::Battlefield);
    drain_to(&mut g, &[Target::Player(p.other())]);
    assert_eq!(g.state.players[1].life, 19);
    let teeming = candidate(&mut g, p, "Teeming Dragonstorm");
    cast(&mut g, teeming, None);
    drain_to(&mut g, &[Target::Player(p.other())]);
    assert_eq!(g.state.players[1].life, 17);
}
#[test]
fn shock_brigade_mobilizes_actual_attacking_warrior_then_sacrifices_next_end() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = cast_candidate(&mut g, p, "Shock Brigade");
    assert_eq!(stats(&g, source), (1, 3));
    assert!(g.keywords(source).contains(Keywords::MENACE));
    g.state.active = p;
    g.begin_turn();
    reset_main(&mut g, p);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: source });
    act(&mut g, Action::FinishAttackers);
    advance(&mut g, &[], |g| !tokens(g, p, "Warrior").is_empty());
    let all = tokens(&g, p, "Warrior");
    assert_eq!(all.len(), 1);
    let id = all[0];
    assert!(g.object_view(id).tapped);
    assert_eq!(stats(&g, id), (1, 1));
    assert_eq!(g.object_view(id).colors, 8);
    assert!(g.state.attackers.iter().any(|a| a.card == id));
    drain(&mut g);
    step(&mut g, Step::End);
    drain(&mut g);
    assert_ne!(g.object_view(id).zone, Zone::Battlefield);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
}
#[test]
fn stormplain_detainment_exiles_only_opponent_nonland_and_returns_exact_object_on_departure() {
    for early in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let own = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let source = candidate(&mut g, p, "Stormplain Detainment");
        cast(&mut g, source, None);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for id in [land, own] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        if early {
            g.move_object(source, Zone::Hand);
        }
        drain(&mut g);
        if early {
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        } else {
            assert_eq!(g.object_view(target).zone, Zone::Exile);
            g.move_object(source, Zone::Graveyard);
            drain(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        }
    }
}

// Only this external subtype provider uses a fixture definition. Both copied
// Dragonstorms retain their actual immutable definition and complete program.
const DRAGON_PROVIDER: rustmage_core::cards::authored::CardRules =
    rustmage_core::cards::authored::CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::ControlledCreatureSubtype {
                grant: CreatureSubtypeGrant::Named("Dragon"),
                required_subtype: None,
            }],
        }],
        ..rustmage_core::cards::authored::CardRules::EMPTY
    };
#[test]
fn actual_dragonstorm_creature_copies_trigger_for_self_only_under_their_own_dragon_provider() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &DRAGON_PROVIDER)]);
    for name in ["Roiling Dragonstorm", "Teeming Dragonstorm"] {
        for provider_is_own in [false, true] {
            let p = Player::P0;
            let mut g = scenario(p);
            put(
                &mut g,
                if provider_is_own { p } else { p.other() },
                "Ghost Vacuum",
                Zone::Battlefield,
            );
            let discard = put(&mut g, p, "Forest", Zone::Hand);
            let actual = Card::from_name(name).unwrap();
            let mut recipe = CopyRecipe::new(actual);
            recipe.exceptions.base = Some((2, 2));
            recipe.overrides = Some(CopyOverrides {
                kinds: Some(33),
                ..CopyOverrides::NONE
            });
            g.stage_copy_creation(p, recipe, 1);
            g.advance_replacements(false);
            g.give_priority();
            let source = *entrants(&g, name).first().unwrap();
            assert_eq!(g.object_view(source).card, actual);
            assert_eq!(g.object_view(source).has_subtype("Dragon"), provider_is_own);
            let returns = g
                .state
                .stack
                .iter()
                .chain(&g.state.pending_triggers)
                .chain(&g.state.ordering_triggers)
                .filter(|s| {
                    s.source.is_some_and(|s| s.id == source)
                        && s.kind == StackKind::TriggeredAbility { ability: 1 }
                })
                .count();
            assert_eq!(
                returns,
                usize::from(provider_is_own),
                "{name}: exact self-entry qualification"
            );
            if provider_is_own {
                // Put the entry payoff below the self-return, so later Soldier
                // Dragons cannot mask a missing self-entry trigger.
                for _ in 0..4 {
                    if !matches!(g.state.pending, Pending::OrderTriggers { .. }) {
                        break;
                    }
                    let id = g
                        .state
                        .ordering_triggers
                        .iter()
                        .find(|s| {
                            s.source.is_some_and(|s| s.id == source)
                                && s.kind == StackKind::TriggeredAbility { ability: 0 }
                        })
                        .or_else(|| g.state.ordering_triggers.first())
                        .unwrap()
                        .id;
                    act(&mut g, Action::PutTrigger { id });
                }
                advance(&mut g, &[], |g| {
                    g.object_view(source).zone != Zone::Battlefield
                });
                assert!(
                    tokens(&g, p, "Soldier").is_empty(),
                    "self-return preceded ETB Soldier creation"
                );
            }
            for _ in 0..32 {
                if idle(&g) {
                    break;
                }
                if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
                    act(&mut g, Action::Discard { card: discard });
                } else {
                    choose(&mut g, |a| {
                        matches!(a, Action::Pass | Action::PutTrigger { .. })
                    });
                }
            }
            assert!(idle(&g));
            if !provider_is_own {
                assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            }
            if name == "Teeming Dragonstorm" {
                assert_eq!(tokens(&g, p, "Soldier").len(), 2);
            } else {
                assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
            }
        }
    }
}
