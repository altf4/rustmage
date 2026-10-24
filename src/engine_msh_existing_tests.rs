//! Complete MSH cards using shared typed mechanics, paid actions and owned replay.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{checked, named, next_turn, settle, setup};
use super::*;

fn cast(g: &mut Game, card: CardId, mode: Option<u16>, targets: &[Target]) {
    let action = if g.legal_actions().contains(&Action::AnnounceExile { card }) {
        Action::AnnounceExile { card }
    } else {
        Action::AnnounceCast { card }
    };
    checked(g, action);
    if let Some(mode) = mode {
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..12 {
        if g.state
            .announcing
            .as_ref()
            .is_none_or(|a| a.item.card != card || a.item.kind != StackKind::Spell)
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid cast");
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            let id = id as CardId;
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id, subtype))
            .then_some(id)
        })
        .collect()
}
fn reject(g: &mut Game, action: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
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
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn resolve_target(g: &mut Game, target: Target) {
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget { target })
    });
    checked(g, Action::ChooseTarget { target });
    settle(g);
}
fn combat(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    g.begin_step(Step::BeginCombat);
    g.prepare_triggers();
}

#[test]
fn msh_ghost_hybrid_flash_and_evasion_paid_both_seats() {
    for p in [Player::P0, Player::P1] {
        for color in [1, 2] {
            let mut g = setup(p);
            g.state.active = p.other();
            g.state.priority = p;
            let ghost = named(&mut g, p, "Ghost, Spectral Saboteur", Zone::Hand);
            // Only one half of the hybrid symbol is fundable.
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = 3;
            cast(&mut g, ghost, None, &[]);
            settle(&mut g);
            assert_eq!(
                g.stats(ghost),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert!(g.keywords(ghost).contains(Keywords::FLASH));
            assert!(g.keywords(ghost).contains(Keywords::CANT_BE_BLOCKED));
            let blocker = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.active = p;
            g.state.priority = p;
            g.state.objects[ghost as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: ghost });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            reject(
                &mut g,
                Action::Block {
                    blocker,
                    attacker: ghost,
                },
            );
            checked(&mut g, Action::FinishBlockers);
        }
    }
}

#[test]
fn msh_training_regimen_controls_counter_trample_and_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = paid(&mut g, p, "Training Regimen");
        assert!(!g.keywords(own).contains(Keywords::TRAMPLE));
        combat(&mut g, p.other());
        assert!(g.state.pending_triggers.is_empty());
        combat(&mut g, p);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(own),
            })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
        assert!(g.keywords(own).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
        g.move_object(source, Zone::Graveyard);
        assert!(!g.keywords(own).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn msh_hire_a_crew_includes_new_villain_snapshot_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        paid(&mut g, p, "Hire a Crew");
        let token = tokens(&g, p, "Villain")[0];
        assert_eq!(
            g.stats(token),
            Stats {
                power: 3,
                toughness: 1
            }
        );
        assert!(g.keywords(token).contains(Keywords::MENACE));
        assert!(!g.object_view(token).tapped);
        assert_eq!(g.stats(own).power, 4);
        assert_eq!(g.stats(enemy).power, 3);
        let later = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        assert_eq!(g.stats(later).power, 3);
        next_turn(&mut g);
        assert_eq!(g.stats(token).power, 2);
        assert_eq!(g.stats(own).power, 3);
    }
}

#[test]
fn msh_hydra_troopers_checks_graveyard_at_resolution_and_mills_otherwise() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1, 2] {
            let mut g = setup(p);
            for _ in 0..count {
                named(&mut g, p, "Hill Giant", Zone::Graveyard);
            }
            // Opponent creatures and our noncreature cards do not satisfy the threshold.
            named(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
            named(&mut g, p, "Forest", Zone::Graveyard);
            let before = g.state.players[p.index()].library.len();
            paid(&mut g, p, "HYDRA Troopers");
            if count == 2 {
                let token = tokens(&g, p, "Villain")[0];
                assert!(g.object_view(token).tapped);
                assert!(g.keywords(token).contains(Keywords::MENACE));
                assert_eq!(
                    g.stats(token),
                    Stats {
                        power: 2,
                        toughness: 1
                    }
                );
                assert_eq!(g.state.players[p.index()].library.len(), before);
            } else {
                assert!(tokens(&g, p, "Villain").is_empty());
                assert_eq!(g.state.players[p.index()].library.len(), before - 2);
            }
        }
        let mut g = setup(p);
        let source = named(&mut g, p, "HYDRA Troopers", Zone::Hand);
        cast(&mut g, source, None, &[]);
        until(&mut g, |g| g.object_view(source).zone == Zone::Battlefield);
        named(&mut g, p, "Hill Giant", Zone::Graveyard);
        named(&mut g, p, "Hill Giant", Zone::Graveyard);
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Villain").len(), 1);
    }
}

#[test]
fn msh_ant_mans_army_resolution_choice_and_food_payment() {
    for p in [Player::P0, Player::P1] {
        for (token, subtype) in [
            (
                Card::from_name("@token/283fd83a9aec83a51b736926").unwrap(),
                "Food",
            ),
            (
                Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap(),
                "Treasure",
            ),
        ] {
            let mut g = setup(p);
            let army = named(&mut g, p, "Ant-Man's Army", Zone::Hand);
            cast(&mut g, army, None, &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::TokenChoice { .. })
            });
            reject(
                &mut g,
                Action::ChooseToken {
                    token: Card::from_name("@token/cf0227c0984ee9114e6c311f").unwrap(),
                },
            );
            checked(&mut g, Action::ChooseToken { token });
            settle(&mut g);
            let id = tokens(&g, p, subtype)[0];
            assert!(
                tokens(
                    &g,
                    p,
                    if subtype == "Food" {
                        "Treasure"
                    } else {
                        "Food"
                    }
                )
                .is_empty()
            );
            assert_eq!(
                g.stats(army),
                Stats {
                    power: 3,
                    toughness: 2
                }
            );
            if subtype == "Food" {
                let life = g.life(p);
                checked(
                    &mut g,
                    Action::AnnounceActivate {
                        card: id,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                until(&mut g, |g| g.state.announcing.is_none());
                settle(&mut g);
                assert_eq!(g.object_view(id).zone, Zone::Outside);
                assert_eq!(g.life(p), life + 3);
            }
        }
    }
}

#[test]
fn msh_crossbones_villain_filter_once_turn_and_departed_source_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Crossbones, Malicious Mercenary");
        assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        let life = g.life(p.other());
        paid(&mut g, p, "Ant-Man's Army");
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        paid(&mut g, p, "Ghost, Spectral Saboteur");
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(g.life(p.other()), life - 2);
        paid(&mut g, p, "HYDRA Troopers");
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(g.life(p.other()), life - 2);
        next_turn(&mut g);
        // Instant creates a second-turn Villain and the trigger survives Crossbones leaving.
        g.state.priority = p;
        g.state.players[p.index()].mana = [30; 6];
        let spell = named(&mut g, p, "Hire a Crew", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.card == source && matches!(s.kind, StackKind::TriggeredAbility { .. }))
        });
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.life(p.other()), life - 4);
    }
}

#[test]
fn msh_mister_fantastic_optional_once_per_token_batch_and_privacy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Mister Fantastic, Reed Richards");
        let secret = named(&mut g, p, "Hill Giant", Zone::Library);
        let hand = g.hand_count(p);
        let spell = named(&mut g, p, "Hire a Crew", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand);
        let spell = named(&mut g, p, "Hire a Crew", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.reference.card == secret)
        );
        let before = g.hand_count(p);
        // One simultaneous batch of two tokens yields one optional draw.
        g.create_tokens(
            p,
            Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap(),
            2,
            false,
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
        g.create_tokens(
            p.other(),
            Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap(),
            2,
            false,
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}

#[test]
fn msh_war_machine_other_own_target_live_power_lki_and_expiry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = paid(&mut g, p, "War Machine, Legacy of Iron");
        assert!(g.keywords(source).contains(Keywords::FLYING));
        combat(&mut g, p.other());
        assert!(g.state.pending_triggers.is_empty());
        combat(&mut g, p);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(target),
            })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        // The source's current power is captured when the trigger resolves, with departure LKI.
        g.add_counter(source, CounterKind::PlusOne, 2);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(
            g.stats(target),
            Stats {
                power: 6,
                toughness: 3
            }
        );
        next_turn(&mut g);
        assert_eq!(g.stats(target).power, 3);
    }
}

#[test]
fn msh_quake_noncreature_cast_creature_or_land_target_restrictions() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Quake, Agent of S.H.I.E.L.D.");
        paid(&mut g, p, "Ghost, Spectral Saboteur");
        assert!(g.state.pending_triggers.is_empty());
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let spell = named(&mut g, p, "Hire a Crew", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(land),
            })
        });
        let artifact = named(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        settle(&mut g);
        assert!(g.object_view(land).tapped);
        let spell = named(&mut g, p, "Hire a Crew", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        resolve_target(&mut g, Target::Card(source));
        assert!(g.object_view(source).tapped);
    }
}

#[test]
fn msh_luke_cage_alone_declaration_indestructible_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for alone in [true, false] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Luke Cage, Power Man");
            let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.state.objects[source as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: source });
            if !alone {
                checked(&mut g, Action::Attack { card: other });
            }
            checked(&mut g, Action::FinishAttackers);
            settle(&mut g);
            assert_eq!(g.stats(source).power, if alone { 4 } else { 2 });
            assert_eq!(g.keywords(source).contains(Keywords::INDESTRUCTIBLE), alone);
            next_turn(&mut g);
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 2,
                    toughness: 5
                }
            );
            assert!(!g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}

fn activate(g: &mut Game, card: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    until(g, |g| g.state.announcing.is_none());
}
#[test]
fn msh_modok_lord_life_cost_connive_land_nonland_and_timing() {
    for p in [Player::P0, Player::P1] {
        for nonland in [false, true] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let source = paid(&mut g, p, "M.O.D.O.K.");
            assert_eq!(g.stats(own).power, 3);
            assert_eq!(
                g.stats(enemy),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert!(
                g.keywords(source)
                    .contains(Keywords::FLYING.union(Keywords::LIFELINK))
            );
            let discarded = named(
                &mut g,
                p,
                if nonland { "Hill Giant" } else { "Forest" },
                Zone::Hand,
            );
            let hand = g.hand_count(p);
            let life = g.life(p);
            activate(&mut g, source);
            assert_eq!(g.life(p), life - 3);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.hand_count(p), hand + 1);
            checked(&mut g, Action::Discard { card: discarded });
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                u64::from(nonland)
            );
            assert_eq!(g.object_view(discarded).zone, Zone::Graveyard);
            g.state.active = p.other();
            g.state.priority = p;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.active = p;
            g.state.players[p.index()].life = 2;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.move_object(source, Zone::Graveyard);
            assert_eq!(
                g.stats(enemy),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
        }
    }
}

#[test]
fn msh_modok_connives_for_current_or_last_controller_without_new_object_counters() {
    for p in [Player::P0, Player::P1] {
        for departure in 0..3 {
            let mut g = setup(p);
            let source = paid(&mut g, p, "M.O.D.O.K.");
            let original = g.object_ref(source);
            let discard = named(&mut g, p.other(), "Hill Giant", Zone::Hand);
            let old_hand = g.hand_count(p);
            let new_hand = g.hand_count(p.other());
            let life = g.life(p);
            activate(&mut g, source);
            assert_eq!(g.life(p), life - 3);
            g.change_controller(original, p.other());
            if departure > 0 {
                g.move_object(source, Zone::Graveyard);
                assert_ne!(g.object_ref(source), original);
            }
            if departure == 2 {
                g.move_object(source, Zone::Battlefield);
                assert_ne!(g.object_ref(source), original);
                assert_eq!(g.object_view(source).controller, p);
            }
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.actor(), Some(p.other()));
            assert_eq!(g.hand_count(p), old_hand);
            assert_eq!(g.hand_count(p.other()), new_hand + 1);
            assert!(
                !g.observe(p)
                    .objects
                    .iter()
                    .any(|o| o.reference.card == discard)
            );
            checked(&mut g, Action::Discard { card: discard });
            settle(&mut g);
            assert_eq!(g.hand_count(p.other()), new_hand);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                u64::from(departure == 0)
            );
        }
    }
}

#[test]
fn msh_masters_villain_lord_and_paid_revealed_hand_tutor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ghost = paid(&mut g, p, "Ghost, Spectral Saboteur");
        let hero = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(
            &mut g,
            p.other(),
            "Ghost, Spectral Saboteur",
            Zone::Battlefield,
        );
        let lord = paid(&mut g, p, "The Masters of Evil");
        assert_eq!(
            g.stats(lord),
            Stats {
                power: 5,
                toughness: 6
            }
        );
        assert_eq!(
            g.stats(ghost),
            Stats {
                power: 4,
                toughness: 3
            }
        );
        assert_eq!(g.stats(hero).power, 3);
        assert_eq!(g.stats(enemy).power, 2);
        let tutor = named(&mut g, p, "The Masters of Evil", Zone::Hand);
        // Only Plan characteristics are needed here; its own gated behavior is not exercised.
        let plan = named(&mut g, p, "Forest", Zone::Library);
        g.state.objects[plan as usize].card = Card::from_name("Construct a Cosmic Cube").unwrap();
        let ordinary = named(&mut g, p, "Forest", Zone::Library);
        let rng = g.state.rng;
        activate(&mut g, tutor);
        assert_eq!(g.object_view(tutor).zone, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert!(g.observe(p.other()).search_choice.is_none());
        reject(&mut g, Action::ChooseSearch { card: ordinary });
        checked(&mut g, Action::ChooseSearch { card: plan });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(plan).zone, Zone::Hand);
        assert_ne!(g.state.rng, rng);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.reference.card == plan)
        );
        g.move_object(lord, Zone::Graveyard);
        assert_eq!(g.stats(ghost).power, 2);
    }
}

#[test]
fn msh_wiccan_excludes_self_lands_tokens_and_blinks_stolen_card_to_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Wiccan, Rising Magician");
        let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let reference = g.object_ref(victim);
        g.change_controller(reference, p);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let spell = named(&mut g, p, "Hire a Crew", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(victim),
            })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        g.move_object(source, Zone::Graveyard);
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
        assert_eq!(g.object_view(victim).controller, p.other());
        assert_ne!(g.object_ref(victim), reference);
    }
}

#[test]
fn msh_thor_graveyard_eligibility_play_permission_and_cast_mana_damage() {
    for p in [Player::P0, Player::P1] {
        for name in ["Hire a Crew", "Divination", "Swiftfoot Boots"] {
            let mut g = setup(p);
            let returned = named(&mut g, p, name, Zone::Graveyard);
            let wrong = named(&mut g, p, "Hill Giant", Zone::Graveyard);
            let enemy = named(&mut g, p.other(), name, Zone::Graveyard);
            let source = named(&mut g, p, "Thor, God of Thunder", Zone::Hand);
            cast(&mut g, source, None, &[]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(returned),
                })
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(wrong),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(returned),
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(returned).zone, Zone::Exile);
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceExile { card: returned })
            );
            let life = g.life(p.other());
            let mv = g.state.objects[returned as usize].card.fixed_mana_value();
            cast(&mut g, returned, None, &[]);
            resolve_target(&mut g, Target::Player(p.other()));
            assert_eq!(g.life(p.other()), life - mv as i128);
        }
    }
}

#[test]
fn msh_truck_toss_vehicle_discount_any_target_and_atomic_illegal_payment() {
    for p in [Player::P0, Player::P1] {
        for own_vehicle in [false, true] {
            let mut g = setup(p);
            named(
                &mut g,
                if own_vehicle { p } else { p.other() },
                "Dependable Quinjet",
                Zone::Battlefield,
            );
            let source = named(&mut g, p, "Truck Toss", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 0];
            if !own_vehicle {
                assert!(
                    !g.legal_actions()
                        .contains(&Action::AnnounceCast { card: source })
                );
                reject(&mut g, Action::AnnounceCast { card: source });
                g.state.players[p.index()].mana[5] = 2;
            }
            let life = g.life(p.other());
            cast(&mut g, source, None, &[Target::Player(p.other())]);
            settle(&mut g);
            assert_eq!(g.life(p.other()), life - 4);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = named(&mut g, p, "Truck Toss", Zone::Hand);
        cast(&mut g, source, None, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}

#[test]
fn msh_thor_permission_survives_departure_through_next_turn_then_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let returned = named(&mut g, p, "Hire a Crew", Zone::Graveyard);
        let source = named(&mut g, p, "Thor, God of Thunder", Zone::Hand);
        cast(&mut g, source, None, &[]);
        resolve_target(&mut g, Target::Card(returned));
        g.move_object(source, Zone::Graveyard);
        for _ in 0..2 {
            next_turn(&mut g);
            g.state.priority = p;
            g.state.players[p.index()].mana = [30; 6];
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceExile { card: returned })
            );
        }
        next_turn(&mut g);
        g.state.priority = p;
        g.state.players[p.index()].mana = [30; 6];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceExile { card: returned })
        );
        reject(&mut g, Action::AnnounceExile { card: returned });
    }
}

#[test]
fn msh_thor_cast_context_uses_printed_value_after_discount_counter_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Thor, God of Thunder");
        named(&mut g, p, "Dependable Quinjet", Zone::Battlefield);
        let spell = named(&mut g, p, "Truck Toss", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 0];
        cast(&mut g, spell, None, &[Target::Player(p.other())]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        let cast_item = g
            .state
            .stack
            .iter()
            .find(|s| s.card == spell && s.kind == StackKind::Spell)
            .unwrap();
        assert_eq!(cast_item.cast.unwrap().payment, [0, 0, 0, 2, 0, 0]);
        g.move_object(source, Zone::Graveyard);
        checked(&mut g, Action::Pass);
        let negate = named(&mut g, p.other(), "Negate", Zone::Hand);
        cast(&mut g, negate, None, &[Target::Spell(spell)]);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(
            g.life(p.other()),
            16,
            "captured mana value is four, despite two mana paid and original spell countered"
        );
    }
}

#[test]
fn msh_thor_spell_copies_are_not_cast_and_do_not_emit_extra_damage_triggers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Thor, God of Thunder", Zone::Battlefield);
        named(&mut g, p, "Prismari, the Inspiration", Zone::Battlefield);
        paid(&mut g, p, "Llanowar Elves");
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        resolve_target(&mut g, Target::Player(p.other()));
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
        assert_eq!(
            g.life(p.other()),
            15,
            "two Shock resolutions plus exactly one Thor trigger"
        );
    }
}

#[test]
fn msh_avengers_disassembled_all_modes_optional_controller_search_and_sweep() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1, 2] {
            for controller in [p, p.other()] {
                for search in [false, true] {
                    let mut g = setup(p);
                    let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
                    let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                    let large = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
                    let land = named(&mut g, controller, "Forest", Zone::Battlefield);
                    let basic = *g.state.players[controller.index()].library.last().unwrap();
                    let spell = named(&mut g, p, "Avengers Disassembled", Zone::Hand);
                    let targets = if mode == 0 {
                        vec![]
                    } else {
                        vec![Target::Card(land)]
                    };
                    cast(&mut g, spell, Some(mode), &targets);
                    if mode != 0 {
                        until(&mut g, |g| {
                            matches!(g.state.pending, Pending::EffectOptional { .. })
                        });
                        assert_eq!(g.actor(), Some(controller));
                        checked(&mut g, Action::ChooseOptional { accept: search });
                        if search {
                            until(&mut g, |g| {
                                matches!(g.state.pending, Pending::EffectSearch { .. })
                            });
                            assert!(g.observe(controller.other()).search_choice.is_none());
                            checked(&mut g, Action::ChooseSearch { card: basic });
                            checked(&mut g, Action::FinishSearch);
                        }
                    }
                    settle(&mut g);
                    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                    assert_eq!(
                        g.object_view(land).zone,
                        if mode == 0 {
                            Zone::Battlefield
                        } else {
                            Zone::Graveyard
                        }
                    );
                    for id in [own, enemy] {
                        assert_eq!(
                            g.object_view(id).zone,
                            if mode == 1 {
                                Zone::Battlefield
                            } else {
                                Zone::Graveyard
                            }
                        );
                    }
                    assert_eq!(g.object_view(large).damage, if mode == 1 { 0 } else { 3 });
                    if mode != 0 && search {
                        assert_eq!(g.object_view(basic).zone, Zone::Battlefield);
                        assert!(g.object_view(basic).tapped);
                        assert_eq!(g.object_view(basic).controller, controller);
                    }
                }
            }
        }
    }
}

#[test]
fn msh_avengers_disassembled_indestructible_land_still_searches_but_invalid_target_counters_both_modes()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        g.state.objects[land as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        let spell = named(&mut g, p, "Avengers Disassembled", Zone::Hand);
        cast(&mut g, spell, Some(1), &[Target::Card(land)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        checked(&mut g, Action::ChooseOptional { accept: true });
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        g.state.players[p.index()].mana = [30; 6];
        let creature = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let spell = named(&mut g, p, "Avengers Disassembled", Zone::Hand);
        cast(&mut g, spell, Some(2), &[Target::Card(land)]);
        g.move_object(land, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(g.object_view(creature).damage, 0);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}

#[test]
fn msh_reptil_paid_repeatable_forms_keep_counters_and_combine_keywords_then_expire() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Reptil, Dinomorpher");
        g.add_counter(source, CounterKind::PlusOne, 1);
        for (slot, stats, keywords) in [
            (
                0,
                Stats {
                    power: 4,
                    toughness: 6,
                },
                Keywords::REACH.union(Keywords::VIGILANCE),
            ),
            (
                1,
                Stats {
                    power: 7,
                    toughness: 7,
                },
                Keywords::REACH
                    .union(Keywords::VIGILANCE)
                    .union(Keywords::TRAMPLE),
            ),
            (
                0,
                Stats {
                    power: 4,
                    toughness: 6,
                },
                Keywords::REACH
                    .union(Keywords::VIGILANCE)
                    .union(Keywords::TRAMPLE),
            ),
        ] {
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: slot,
                    counter_payment: Counters::default(),
                },
            );
            until(&mut g, |g| g.state.announcing.is_none());
            settle(&mut g);
            assert_eq!(g.stats(source), stats);
            assert!(g.keywords(source).contains(keywords));
            assert!(g.has_subtype(source, "Dinosaur"));
            assert!(g.has_subtype(source, "Hero"));
            assert!(!g.has_subtype(source, "Human"));
            assert_eq!(g.object_view(source).colors, 16);
        }
        next_turn(&mut g);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 2,
                toughness: 3
            }
        );
        assert!(g.has_subtype(source, "Human"));
        assert!(!g.has_subtype(source, "Dinosaur"));
        assert!(!g.keywords(source).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(source).contains(Keywords::REACH));
        g.state.priority = p;
        g.state.players[p.index()].mana = [30; 6];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 1,
                toughness: 2
            }
        );
        assert!(!g.has_subtype(source, "Dinosaur"));
    }
}
