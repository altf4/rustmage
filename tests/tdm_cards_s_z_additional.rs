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

fn choose_targets(g: &mut Game, ids: &[CardId]) {
    for &id in ids {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn attack(g: &mut Game, p: Player, ids: &[CardId]) {
    reset_main(g, p);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in ids {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
}
fn finish_costs(g: &mut Game, material: Option<CardId>) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        if matches!(g.state.pending, Pending::CastCosts { .. }) {
            let slot = g
                .state
                .announcing
                .as_ref()
                .unwrap()
                .cast_costs
                .as_ref()
                .unwrap()
                .clauses
                .iter()
                .position(|c| !c.paid)
                .unwrap();
            act(
                g,
                Action::ChooseCastCost {
                    clause: slot as u16,
                },
            );
        } else if matches!(g.state.pending, Pending::AdditionalCost { .. }) {
            let object = g.object_ref(material.expect("explicit sacrificed creature"));
            act(g, Action::ChooseAdditionalCost { object });
        } else {
            pay(g);
        }
    }
    panic!("12-step cost budget");
}
#[test]
fn salt_road_destroys_creature_creates_two_hasty_warriors_and_next_end_sacrifices_only_them() {
    for invalidated in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let existing = put(
            &mut g,
            p,
            Card::test_index(5255).identity(),
            Zone::Battlefield,
        );
        let source = candidate(&mut g, p, "Salt Road Skirmish");
        g.state.players[0].mana = [0, 0, 1, 0, 0, 3];
        cast(&mut g, source, Some(Target::Card(target)));
        assert_eq!(g.state.players[0].mana, [0; 6]);
        if invalidated {
            g.move_object(target, Zone::Hand);
        }
        drain(&mut g);
        let warriors = tokens(&g, p, "Warrior");
        assert_eq!(warriors.len(), if invalidated { 1 } else { 3 });
        assert_eq!(
            g.object_view(target).zone,
            if invalidated {
                Zone::Hand
            } else {
                Zone::Graveyard
            }
        );
        for &id in warriors.iter().filter(|&&id| id != existing) {
            assert_eq!(stats(&g, id), (1, 1));
            assert_eq!(g.object_view(id).colors, 8);
            assert!(g.keywords(id).contains(Keywords::HASTE));
            assert!(!g.object_view(id).tapped);
            assert!(!g.state.attackers.iter().any(|a| a.card == id));
        }
        step(&mut g, Step::End);
        drain(&mut g);
        assert_eq!(tokens(&g, p, "Warrior"), vec![existing]);
    }
}
#[test]
fn severance_reveals_opponent_hand_may_exile_nonland_and_its_owner_gets_sized_white_spirit() {
    for accept in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        let land = put(&mut g, p.other(), "Forest", Zone::Hand);
        let card = put(&mut g, p.other(), "Hill Giant", Zone::Hand);
        let own = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        let source = candidate(&mut g, p, "Severance Priest");
        cast(&mut g, source, None);
        advance(&mut g, &[Target::Player(p.other())], |g| {
            matches!(g.state.pending, Pending::HandCardChoice { .. })
        });
        assert_eq!(stats(&g, source), (3, 3));
        assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
        for viewer in [p, p.other()] {
            assert!(g.observe(viewer).known_hand.contains(&g.object_ref(land)));
            assert!(g.observe(viewer).known_hand.contains(&g.object_ref(card)));
        }
        for id in [land, own] {
            let object = g.object_ref(id);
            reject(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(object),
                },
            );
        }
        let object = if accept {
            Some(g.object_ref(card))
        } else {
            None
        };
        act(&mut g, Action::ChooseHandCard { object });
        drain(&mut g);
        assert_eq!(
            g.object_view(card).zone,
            if accept { Zone::Exile } else { Zone::Hand }
        );
        g.change_controller(g.object_ref(source), p.other());
        g.move_object(source, Zone::Graveyard);
        drain(&mut g);
        assert_eq!(tokens(&g, p.other(), "Spirit").len(), usize::from(accept));
        assert!(tokens(&g, p, "Spirit").is_empty());
        if accept {
            let token = tokens(&g, p.other(), "Spirit")[0];
            assert_eq!(stats(&g, token), (4, 4));
            assert_eq!(g.object_view(token).colors, 1);
            assert_eq!(g.object_view(card).zone, Zone::Exile);
        }
    }
}
#[test]
fn shiko_exiles_only_own_small_nonland_then_free_casts_real_spell_or_permanent_copy_or_declines() {
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let name = if mode == 1 {
            "Grizzly Bears"
        } else {
            "Divination"
        };
        let target = put(&mut g, p, name, Zone::Graveyard);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        let large = put(&mut g, p, "Hill Giant", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Graveyard);
        let source = candidate(&mut g, p, "Shiko, Paragon of the Way");
        cast(&mut g, source, None);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for id in [land, large, enemy] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        choose_targets(&mut g, &[target]);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::ResolutionCasts { .. })
        });
        assert_eq!(g.object_view(target).zone, Zone::Exile);
        let offered = g
            .state
            .resolution_free_casts
            .as_ref()
            .unwrap()
            .remaining
            .clone();
        assert_eq!(offered.len(), 1);
        let object = offered[0];
        assert_ne!(object, g.object_ref(target));
        let hand = g.hand_count(p);
        g.state.players[0].mana = [0; 6];
        if mode != 2 {
            act(
                &mut g,
                Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal,
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
        }
        act(&mut g, Action::FinishResolutionCasts);
        drain(&mut g);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert_eq!(stats(&g, source), (4, 5));
        assert!(
            g.keywords(source)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        if mode == 0 {
            assert_eq!(g.hand_count(p), hand + 2);
        } else if mode == 1 {
            let copies = entrants(&g, "Grizzly Bears");
            assert_eq!(copies.len(), 1);
            assert!(g.object_view(copies[0]).is_token());
            assert_eq!(stats(&g, copies[0]), (2, 2));
        } else {
            assert_eq!(g.hand_count(p), hand);
        }
    }
}
#[test]
fn sidisi_nonmana_x_exact_sacrifice_and_x_plus_one_return_obey_tap_and_sorcery() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = cast_candidate(&mut g, p, "Sidisi, Regent of the Mire");
        assert_eq!(stats(&g, source), (1, 3));
        g.state.active = p;
        g.begin_turn();
        reset_main(&mut g, p);
        let material = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut g, p, "Vampire Nighthawk", Zone::Graveyard);
        let wrong = put(&mut g, p, "Shivan Dragon", Zone::Graveyard);
        g.state.step = Step::Upkeep;
        reject(&mut g, activation(source));
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, activation(source));
        reject(&mut g, Action::ChooseX { value: 0 });
        act(&mut g, Action::ChooseX { value: 2 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(wrong),
            },
        );
        choose_targets(&mut g, &[target]);
        finish_costs(&mut g, Some(material));
        assert!(g.object_view(source).tapped);
        assert_eq!(g.object_view(material).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.object_view(wrong).zone, Zone::Graveyard);
    }
}
#[test]
fn smile_own_upkeep_up_to_two_power_two_or_less_return_then_counters_allow_partial_survivors() {
    for count in [0, 1, 2] {
        let p = Player::P1;
        let mut g = scenario(p);
        let source = cast_candidate(&mut g, p, "Smile at Death");
        let a = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        let b = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let strong = put(&mut g, p, "Hill Giant", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Graveyard);
        g.state.active = p.other();
        step(&mut g, Step::Upkeep);
        drain(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        g.state.active = p;
        step(&mut g, Step::Upkeep);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for id in [strong, enemy] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        choose_targets(&mut g, &[a, b][..count]);
        if count == 2 {
            g.move_object(a, Zone::Hand);
        }
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        if count == 1 {
            assert_eq!(g.object_view(a).zone, Zone::Battlefield);
            assert_eq!(stats(&g, a), (3, 3));
        }
        if count == 2 {
            assert_eq!(g.object_view(a).zone, Zone::Hand);
            assert_eq!(g.object_view(b).zone, Zone::Battlefield);
            assert_eq!(stats(&g, b), (2, 2));
        }
        if count == 0 {
            assert_eq!(g.object_view(a).zone, Zone::Graveyard);
            assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn sonic_shrieker_damage_life_and_actual_damaged_player_discard_are_separate_from_creature_hits() {
    for player_target in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let discard = put(&mut g, p.other(), "Forest", Zone::Hand);
        let source = candidate(&mut g, p, "Sonic Shrieker");
        cast(&mut g, source, None);
        let target = if player_target {
            Target::Player(p.other())
        } else {
            Target::Card(victim)
        };
        advance(&mut g, &[target], |g| {
            if player_target {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            } else {
                idle(g)
            }
        });
        if player_target {
            assert_eq!(g.actor(), Some(p.other()));
            act(&mut g, Action::Discard { card: discard });
            drain(&mut g);
        }
        assert_eq!(g.state.players[0].life, 22);
        assert_eq!(g.state.players[1].life, if player_target { 18 } else { 20 });
        assert_eq!(
            g.object_view(victim).damage,
            if player_target { 0 } else { 2 }
        );
        assert_eq!(
            g.object_view(discard).zone,
            if player_target {
                Zone::Graveyard
            } else {
                Zone::Hand
            }
        );
        assert_eq!(stats(&g, source), (4, 4));
        assert!(g.keywords(source).contains(Keywords::FLYING));
    }
}
#[test]
fn stalwart_first_any_counter_on_each_own_creature_is_extra_once_per_turn_and_tokens_enter_with_counters()
 {
    fn renew_keywords(g: &mut Game, p: Player, target: CardId) {
        // add_counter is storage-only. Real paid renew enters the replacement
        // and placement-event pipeline used by this printed trigger.
        let renew = put(g, p, "Qarsi Revenant", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        act(g, activation(renew));
        choose_targets(g, &[target]);
        pay(g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(renew).zone, Zone::Exile);
        drain(g);
    }
    let p = Player::P0;
    let mut g = scenario(p);
    let source = cast_candidate(&mut g, p, "Stalwart Successor");
    assert_eq!(stats(&g, source), (3, 2));
    assert!(g.keywords(source).contains(Keywords::MENACE));
    let a = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    for id in [a, b, source, enemy] {
        renew_keywords(&mut g, p, id);
    }
    assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(b, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
    renew_keywords(&mut g, p, a);
    assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
    g.state.active = p;
    g.begin_turn();
    reset_main(&mut g, p);
    renew_keywords(&mut g, p, a);
    assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
    g.stage_copy_creation(
        p,
        CopyRecipe::new(Card::from_name("Host of the Hereafter").unwrap()),
        1,
    );
    g.advance_replacements(false);
    g.give_priority();
    drain(&mut g);
    let copies = entrants(&g, "Host of the Hereafter");
    let host = copies[0];
    assert!(g.object_view(host).is_token());
    assert_eq!(g.counter_count(host, CounterKind::PlusOne), 3);
    assert_eq!(stats(&g, host), (5, 5));
}
#[test]
fn starry_skyrider_attack_target_and_attacking_token_flying_have_distinct_durations() {
    let p = Player::P1;
    let mut g = scenario(p);
    let source = cast_candidate(&mut g, p, "Starry-Eyed Skyrider");
    let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let token = put(
        &mut g,
        p,
        Card::test_index(5255).identity(),
        Zone::Battlefield,
    );
    let idle_token = put(
        &mut g,
        p,
        Card::test_index(5255).identity(),
        Zone::Battlefield,
    );
    assert_eq!(stats(&g, source), (1, 3));
    assert!(g.keywords(source).contains(Keywords::FLYING));
    assert!(!g.keywords(token).contains(Keywords::FLYING));
    g.state.active = p;
    g.begin_turn();
    reset_main(&mut g, p);
    attack(&mut g, p, &[source, token]);
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    for id in [source, enemy] {
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    choose_targets(&mut g, &[own]);
    drain(&mut g);
    assert!(g.keywords(own).contains(Keywords::FLYING));
    assert!(g.keywords(token).contains(Keywords::FLYING));
    assert!(!g.keywords(idle_token).contains(Keywords::FLYING));
    assert!(!g.keywords(enemy).contains(Keywords::FLYING));
    g.state.attackers.clear();
    assert!(!g.keywords(token).contains(Keywords::FLYING));
    assert!(g.keywords(own).contains(Keywords::FLYING));
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    assert!(!g.keywords(own).contains(Keywords::FLYING));
    assert!(g.keywords(source).contains(Keywords::FLYING));
}
#[test]
fn snowmelt_stag_turn_base_stats_and_exact_paid_unblockability_expire_independently() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = cast_candidate(&mut g, p, "Snowmelt Stag");
    assert_eq!(
        Card::from_name("Snowmelt Stag").unwrap().effect(),
        Effect::Creature(2, 5)
    );
    assert_eq!(stats(&g, source), (5, 2));
    assert!(g.keywords(source).contains(Keywords::VIGILANCE));
    g.state.active = p.other();
    assert_eq!(stats(&g, source), (2, 5));
    g.state.players[0].mana = [0, 2, 0, 0, 0, 5];
    act(&mut g, activation(source));
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert!(g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    assert!(!g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
    assert!(g.keywords(source).contains(Keywords::VIGILANCE));
}
#[test]
fn stadium_mobilize_and_sacrifice_damage_counts_current_creatures_after_source_is_paid() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = cast_candidate(&mut g, p, "Stadium Headliner");
    assert_eq!(stats(&g, source), (1, 1));
    let ally = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.state.active = p;
    g.begin_turn();
    reset_main(&mut g, p);
    attack(&mut g, p, &[source]);
    advance(&mut g, &[], |g| !tokens(g, p, "Warrior").is_empty());
    drain(&mut g);
    let token = tokens(&g, p, "Warrior")[0];
    assert!(g.state.attackers.iter().any(|a| a.card == token));
    assert!(g.object_view(token).tapped);
    assert_eq!(stats(&g, token), (1, 1));
    g.state.players[0].mana = [0, 0, 0, 1, 0, 1];
    act(&mut g, activation(source));
    choose_targets(&mut g, &[victim]);
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    g.move_object(ally, Zone::Hand);
    drain(&mut g);
    assert_eq!(
        g.object_view(victim).damage,
        1,
        "only surviving Warrior counted at resolution"
    );
    step(&mut g, Step::End);
    drain(&mut g);
    assert!(tokens(&g, p, "Warrior").is_empty());
}
#[test]
fn static_snare_flash_counts_attackers_from_both_players_and_exiles_artifact_or_creature_until_leaving()
 {
    for artifact in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        let a = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let target = put(
            &mut g,
            p.other(),
            if artifact {
                "Short Bow"
            } else {
                "Shivan Dragon"
            },
            Zone::Battlefield,
        );
        let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enchant = put(&mut g, p.other(), "Roiling Dragonstorm", Zone::Battlefield);
        attack(&mut g, p.other(), &[a, b]);
        drain(&mut g);
        if g.actor() != Some(p) {
            act(&mut g, Action::Pass);
        }
        let source = put(&mut g, p, "Static Snare", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        cast(&mut g, source, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for id in [own, enchant] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        choose_targets(&mut g, &[target]);
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Exile);
        g.move_object(source, Zone::Hand);
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert!(g.keywords(source).contains(Keywords::FLASH));
    }
}
#[test]
fn stormbeacon_equip_two_only_own_creature_as_sorcery_and_draw_threshold_uses_own_attackers() {
    for attacking in [2, 3] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = cast_candidate(&mut g, p, "Stormbeacon Blade");
        let body = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let c = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.step = Step::Upkeep;
        reject(&mut g, activation(source));
        g.state.step = Step::Main1;
        g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
        act(&mut g, activation(source));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        choose_targets(&mut g, &[body]);
        pay(&mut g);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        drain(&mut g);
        assert_eq!(g.object_view(source).attached_to, Some(g.object_ref(body)));
        assert_eq!(stats(&g, body), (5, 2));
        let hand = g.hand_count(p);
        attack(&mut g, p, &[body, b, c][..attacking]);
        drain(&mut g);
        assert_eq!(g.hand_count(p), hand + usize::from(attacking == 3));
        g.move_object(source, Zone::Graveyard);
        assert_eq!(stats(&g, body), (2, 2));
    }
}
#[test]
fn stillness_mills_three_then_refills_only_empty_library_with_private_ordered_five_or_all_available()
 {
    for (library, grave) in [(4, 0), (3, 5), (0, 2), (0, 0)] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = cast_candidate(&mut g, p, "Stillness in Motion");
        let old = g.state.players[0].library.clone();
        for id in old {
            g.move_object(id, Zone::Hand);
        }
        for _ in 0..grave {
            put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        }
        for _ in 0..library {
            put(&mut g, p, "Forest", Zone::Library);
        }
        g.state.active = p;
        step(&mut g, Step::Upkeep);
        if library <= 3 && library + grave > 0 {
            advance(&mut g, &[], |g| {
                matches!(g.state.pending, Pending::MilledSelection { .. })
            });
            assert!(g.observe(p.other()).milled_choice.is_none());
            reject(&mut g, Action::FinishMilledSelection);
            let candidates = g.observe(p).milled_choice.unwrap().candidates;
            let required = (library + grave).min(5);
            let order = candidates
                .iter()
                .rev()
                .take(required)
                .copied()
                .collect::<Vec<_>>();
            for &object in &order {
                act(&mut g, Action::SelectMilled { object });
            }
            if g.legal_actions().contains(&Action::FinishMilledSelection) {
                act(&mut g, Action::FinishMilledSelection);
            }
            drain(&mut g);
            let actual = g.state.players[0]
                .library
                .iter()
                .rev()
                .copied()
                .collect::<Vec<_>>();
            assert_eq!(actual, order.iter().map(|r| r.card).collect::<Vec<_>>());
        } else {
            drain(&mut g);
        }
        assert_eq!(
            g.object_view(source).zone,
            if library <= 3 {
                Zone::Exile
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.library_card_count(p),
            if library <= 3 {
                (library + grave).min(5)
            } else {
                library - 3
            }
        );
        assert!(!g.state.players[0].attempted_empty_draw);
    }
}
#[test]
fn synchronized_distributes_two_one_or_split_then_grants_all_current_own_counter_bearers_until_cleanup()
 {
    for split in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        let a = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let existing = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let unmarked = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.add_counter(existing, CounterKind::Soul, 1);
        g.add_counter(enemy, CounterKind::Soul, 1);
        let source = candidate(&mut g, p, "Synchronized Charge");
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        act(&mut g, Action::AnnounceCast { card: source });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        let targets = [a, b];
        choose_targets(&mut g, &targets[..if split { 2 } else { 1 }]);
        reject(&mut g, Action::AllocateTargetAmount { amount: 0 });
        for &amount in if split { &[1, 1][..] } else { &[2][..] } {
            act(&mut g, Action::AllocateTargetAmount { amount });
        }
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.counter_count(a, CounterKind::PlusOne),
            if split { 1 } else { 2 }
        );
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), u64::from(split));
        let k = Keywords::VIGILANCE.union(Keywords::TRAMPLE);
        for id in [a, existing] {
            assert!(g.keywords(id).contains(k));
        }
        assert_eq!(g.keywords(b).contains(k), split);
        for id in [unmarked, enemy] {
            assert!(!g.keywords(id).intersects(k));
        }
        g.state.step = Step::Cleanup;
        g.finish_cleanup();
        assert!(!g.keywords(a).intersects(k));
        assert!(g.counter_count(a, CounterKind::PlusOne) > 0);
    }
}
#[test]
fn synchronized_harmonize_four_generic_green_uses_actual_power_tap_then_exiles() {
    let p = Player::P0;
    let mut g = scenario(p);
    let helper = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let target = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let source = candidate(&mut g, p, "Synchronized Charge");
    g.move_object(source, Zone::Graveyard);
    g.state.players[0].mana = [0, 0, 0, 0, 1, 1];
    act(&mut g, Action::AnnounceHarmonize { card: source });
    let object = g.object_ref(helper);
    act(
        &mut g,
        Action::ChooseHarmonizer {
            object: Some(object),
        },
    );
    choose_targets(&mut g, &[target]);
    act(&mut g, Action::AllocateTargetAmount { amount: 2 });
    finish_costs(&mut g, None);
    assert!(g.object_view(helper).tapped);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
    assert_eq!(stats(&g, target), (4, 4));
    assert!(
        g.keywords(target)
            .contains(Keywords::VIGILANCE.union(Keywords::TRAMPLE))
    );
}
#[test]
fn taigam_second_cast_copy_suspend_four_owner_upkeeps_optional_free_recast_and_creature_haste() {
    for accept in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = cast_candidate(&mut g, p, "Taigam, Master Opportunist");
        assert_eq!(stats(&g, source), (2, 2));
        let spell = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        cast(&mut g, spell, None);
        drain(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.counter_count(spell, CounterKind::Time), 4);
        let copies = entrants(&g, "Grizzly Bears");
        assert_eq!(copies.len(), 1);
        assert!(g.object_view(copies[0]).is_token());
        let third = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, third, None);
        drain(&mut g);
        assert_eq!(g.object_view(third).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(third, CounterKind::Time), 0);
        g.move_object(source, Zone::Graveyard);
        g.state.active = p.other();
        step(&mut g, Step::Upkeep);
        drain(&mut g);
        assert_eq!(g.counter_count(spell, CounterKind::Time), 4);
        g.state.active = p;
        for remaining in [3, 2, 1] {
            step(&mut g, Step::Upkeep);
            drain(&mut g);
            assert_eq!(g.counter_count(spell, CounterKind::Time), remaining);
        }
        step(&mut g, Step::Upkeep);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::ResolutionCasts { .. })
        });
        assert_eq!(g.counter_count(spell, CounterKind::Time), 0);
        g.state.players[0].mana = [0; 6];
        if accept {
            let object = g.object_ref(spell);
            act(
                &mut g,
                Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal,
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
        }
        act(&mut g, Action::FinishResolutionCasts);
        drain(&mut g);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert_eq!(
            g.object_view(spell).zone,
            if accept {
                Zone::Battlefield
            } else {
                Zone::Exile
            }
        );
        if accept {
            assert!(g.keywords(spell).contains(Keywords::HASTE));
            assert!(!g.object_view(spell).is_token());
        }
    }
}
