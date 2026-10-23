//! Paid-action audit of inherited TMT admissions. Every checked action compares
//! raw/index owned branches, both observations, rejected actors and restore.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, settle_on, setup};
use super::*;
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        let choice = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
            .unwrap();
        checked(g, choice);
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    let original = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..8 {
        if g.state.announcing.as_ref().map(|a| a.item.id) != original
            || g.state.announcing.is_none()
        {
            break;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    id
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(before, g.snapshot());
    assert_eq!(revision, g.revision());
}
fn activate_ability(g: &mut Game, id: CardId, ability: u16, targets: &[Target]) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid activation");
}
fn until(g: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..100 {
        if condition(g) {
            return;
        }
        g.prepare_triggers();
        if condition(g) {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("condition not reached {:?}", g.state.pending);
}
fn combat(g: &mut Game, id: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    until(g, |g| g.state.pending == Pending::Blockers);
}
fn own_main(g: &mut Game, p: Player) {
    until(g, |g| {
        g.state.active == p
            && g.state.step == Step::Main1
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
    });
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
}
fn activate(g: &mut Game, id: CardId, targets: &[Target]) {
    activate_ability(g, id, 0, targets);
}
fn count_subtype(g: &Game, p: Player, subtype: &str) -> usize {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && g.has_subtype(*id as CardId, subtype)
        })
        .count()
}
#[test]
fn tmt_inherited_two_east_wind_alliance_each_creature_including_tokens_and_eot_expiration() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let avatar = paid(&mut g, p, "East Wind Avatar", &[]);
        settle(&mut g);
        assert_eq!((g.stats(avatar).power, g.stats(avatar).toughness), (2, 4));
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        paid(&mut g, p, "Mechanized Ninja Cavalry", &[]);
        settle(&mut g);
        assert_eq!((g.stats(avatar).power, g.stats(avatar).toughness), (4, 4));
        combat(&mut g, avatar);
        assert!(!g.state.objects[avatar as usize].tapped);
        let ground = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: avatar,
            },
        );
        next_turn(&mut g);
        own_main(&mut g, p.other());
        assert_eq!(g.stats(avatar).power, 2);
        paid(&mut g, p.other(), "Grizzly Bears", &[]);
        settle(&mut g);
        assert_eq!(g.stats(avatar).power, 2);
    }
}
#[test]
fn tmt_inherited_two_foot_elite_hybrid_attack_other_own_target_indestructible_expires() {
    for color in [0, 2] {
        let p = Player::P0;
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.index()].mana[color] = 3;
        let elite = paid(&mut g, p, "Foot Elite", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: elite });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for id in [elite, enemy] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        settle(&mut g);
        assert_eq!(g.stats(own).power, 3);
        assert!(g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Murder", &[Target::Card(own)]);
        settle(&mut g);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Battlefield);
        next_turn(&mut g);
        assert_eq!(g.stats(own).power, 2);
        assert!(!g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
    }
}
#[test]
fn tmt_inherited_two_frog_butler_summoning_sickness_and_each_colored_mana_immediate() {
    for color in 0..5 {
        let p = Player::P0;
        let mut g = setup(p);
        let frog = paid(&mut g, p, "Frog Butler", &[]);
        settle(&mut g);
        let mut production = [0; 6];
        production[color] = 1;
        let action = Action::ActivateMana {
            card: frog,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production,
        };
        reject(&mut g, action.clone());
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, action.clone());
        assert_eq!(g.state.players[p.index()].mana, production);
        assert!(g.state.objects[frog as usize].tapped);
        assert!(g.state.stack.is_empty());
        reject(&mut g, action);
    }
}
#[test]
fn tmt_inherited_two_frog_butler_paid_reach_blocks_flyer_and_deathtouch_kills() {
    let p = Player::P0;
    let mut g = setup(p);
    let frog = paid(&mut g, p, "Frog Butler", &[]);
    settle(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p.other());
    let flyer = named(&mut g, p.other(), "Vampire Nighthawk", Zone::Battlefield);
    // Give the defending player priority before attackers, through a real pass.
    checked(&mut g, Action::Pass);
    g.state.players[p.index()].mana = [30; 6];
    activate_ability(&mut g, frog, 1, &[]);
    settle(&mut g);
    assert!(
        g.keywords(frog)
            .contains(Keywords::REACH.union(Keywords::DEATHTOUCH))
    );
    combat(&mut g, flyer);
    checked(
        &mut g,
        Action::Block {
            blocker: frog,
            attacker: flyer,
        },
    );
    checked(&mut g, Action::FinishBlockers);
    until(&mut g, |g| {
        g.state.step == Step::Main2 && g.state.pending == Pending::Priority
    });
    assert_eq!(g.state.objects[flyer as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[frog as usize].zone, Zone::Graveyard);
}
#[test]
fn tmt_inherited_two_genghis_self_and_other_mutants_but_not_enemy_or_nonmutants() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let frog = paid(&mut g, p, "Genghis Frog", &[]);
        settle(&mut g);
        assert!(g.keywords(frog).contains(Keywords::TRAMPLE));
        assert_eq!(count_subtype(&g, p, "Mutagen"), 1);
        paid(&mut g, p, "Crustacean Commando", &[]);
        settle(&mut g);
        assert_eq!(count_subtype(&g, p, "Mutagen"), 3);
        paid(&mut g, p, "Grizzly Bears", &[]);
        settle(&mut g);
        assert_eq!(count_subtype(&g, p, "Mutagen"), 3);
        next_turn(&mut g);
        own_main(&mut g, p.other());
        paid(&mut g, p.other(), "Crustacean Commando", &[]);
        settle(&mut g);
        assert_eq!(count_subtype(&g, p, "Mutagen"), 3);
    }
}
#[test]
fn tmt_inherited_two_grounded_tapped_discount_untapped_full_fee_and_target_fizzle() {
    for p in [Player::P0, Player::P1] {
        for tapped in [false, true] {
            let mut g = setup(p);
            let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            g.state.objects[victim as usize].tapped = tapped;
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            paid(&mut g, p, "Grounded for Life", &[Target::Card(victim)]);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                mana - if tapped { 2 } else { 5 }
            );
            // Target's tapped status is irrelevant at resolution and cannot refresh total fee.
            g.state.objects[victim as usize].tapped = !tapped;
            settle(&mut g);
            assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
        }
    }
    let p = Player::P0;
    let mut g = setup(p);
    let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    paid(&mut g, p, "Grounded for Life", &[Target::Card(victim)]);
    g.move_object(victim, Zone::Hand);
    g.move_object(victim, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.state.objects[victim as usize].zone, Zone::Battlefield);
}
#[test]
fn tmt_inherited_two_guac_flash_on_enemy_turn_untap_buff_food_and_expiration() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.objects[bear as usize].tapped = true;
        next_turn(&mut g);
        own_main(&mut g, p.other());
        checked(&mut g, Action::Pass);
        g.state.players[p.index()].mana = [30; 6];
        let pizza = paid(&mut g, p, "Guac & Marshmallow Pizza", &[]);
        settle_on(&mut g, Some(Target::Card(bear)));
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (4, 4));
        assert!(!g.state.objects[bear as usize].tapped);
        // Get the controller's priority for the instant-speed Food activation.
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        let life = g.state.players[p.index()].life;
        activate(&mut g, pizza, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        assert_eq!(g.state.objects[pizza as usize].zone, Zone::Graveyard);
        next_turn(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
    }
}
#[test]
fn tmt_inherited_two_hamato_private_scry_actual_stats_flying_eot_and_all_invalid_fizzle() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let top = named(&mut g, p, "Shock", Zone::Library);
        paid(&mut g, p, "Hamato Guardian Stance", &[Target::Card(bear)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrary { .. })
        });
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (3, 5));
        assert!(g.keywords(bear).contains(Keywords::FLYING));
        assert!(g.observe(p.other()).library_choice.is_none());
        checked(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Bottom,
            },
        );
        settle(&mut g);
        next_turn(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
        assert!(!g.keywords(bear).contains(Keywords::FLYING));
    }
    let p = Player::P0;
    let mut g = setup(p);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let top = *g.state.players[p.index()].library.last().unwrap();
    paid(&mut g, p, "Hamato Guardian Stance", &[Target::Card(bear)]);
    g.move_object(bear, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].library.last(), Some(&top));
}
#[test]
fn tmt_inherited_two_jitte_real_equip_and_two_damage_steps_with_departure_loses_keywords() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let jitte = paid(&mut g, p, "Hard-Won Jitte", &[]);
        settle(&mut g);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: jitte,
                ability: 0,
                counter_payment: Counters::default(),
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
                target: Target::Card(bear),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        settle(&mut g);
        let before = g.state.players[p.other().index()].life;
        combat(&mut g, bear);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: jitte,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.state.players[p.other().index()].life, before - 4);

        // A source move ends the continuous keyword grant immediately.
        g.move_object(jitte, Zone::Hand);
        assert!(!g.keywords(bear).contains(Keywords::DOUBLE_STRIKE));
    }
}
#[test]
fn tmt_inherited_two_henchbots_target_restrictions_and_return_without_stack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.objects[victim as usize].tapped = true;
        let untapped = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[own as usize].tapped = true;
        let bot = paid(&mut g, p, "Henchbots", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for card in [untapped, own] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Exile);
        paid(&mut g, p, "Murder", &[Target::Card(bot)]);
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Battlefield);
        assert!(!g.state.objects[victim as usize].tapped);
        assert!(g.state.objects[victim as usize].summoning_sick);
    }
}
#[test]
fn tmt_inherited_two_henchbots_departed_source_before_resolution_and_untapped_target() {
    for remove_source in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.objects[victim as usize].tapped = true;
        let bot = paid(&mut g, p, "Henchbots", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        if remove_source {
            g.move_object(bot, Zone::Hand);
        } else {
            g.state.objects[victim as usize].tapped = false;
        }
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn tmt_inherited_two_jennika_paid_etb_red_mutant_and_plainscycling_reveal_shuffle() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Jennika, Bad Apple Big Sister", &[]);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card.has_subtype("Mutant")
            })
            .unwrap()
            .0 as CardId;
        assert_eq!((g.stats(token).power, g.stats(token).toughness), (2, 2));
        assert_eq!(g.current_kinds(token), 1);
        assert_eq!(g.state.objects[token as usize].card.colors(), 8);
        let cycler = named(&mut g, p, "Jennika, Bad Apple Big Sister", Zone::Hand);
        let plains = named(&mut g, p, "Plains", Zone::Library);
        let forest = *g.state.players[p.index()].library.first().unwrap();
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        activate(&mut g, cycler, &[]);
        assert_eq!(g.state.objects[cycler as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            before - 2
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert!(g.observe(p.other()).actions.is_empty());
        reject(&mut g, Action::ChooseSearch { card: forest });
        checked(&mut g, Action::ChooseSearch { card: plains });
        settle(&mut g);
        assert_eq!(g.state.objects[plains as usize].zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(plains))
        );
    }
}
#[test]
fn tmt_inherited_two_krang_actual_haste_flying_trample_indestructible_and_filtered_grants() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let robot = paid(&mut g, p, "Mechanized Ninja Cavalry", &[]);
        settle(&mut g);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(
            &mut g,
            p.other(),
            "@token/667dad32c9f20b70d5e9bda3",
            Zone::Battlefield,
        );
        let krang = paid(&mut g, p, "Krang, Utrom Warlord", &[]);
        settle(&mut g);
        let mask = Keywords::FLYING
            .union(Keywords::TRAMPLE)
            .union(Keywords::HASTE)
            .union(Keywords::INDESTRUCTIBLE);
        assert!(g.keywords(krang).contains(mask));
        assert!(g.keywords(robot).contains(mask));
        assert!(!g.keywords(bear).intersects(mask));
        assert!(!g.keywords(enemy).intersects(mask));
        paid(&mut g, p, "Murder", &[Target::Card(robot)]);
        settle(&mut g);
        assert_eq!(g.state.objects[robot as usize].zone, Zone::Battlefield);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: krang });
        checked(&mut g, Action::Attack { card: robot });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: enemy,
                attacker: robot,
            },
        );
        g.move_object(krang, Zone::Exile);
        assert!(!g.keywords(robot).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(robot).contains(Keywords::FLYING));
    }
}
#[test]
fn tmt_inherited_two_lessons_draw_three_then_private_optional_tapped_land() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let land = named(&mut g, p, "Island", Zone::Library);
            let nonland = named(&mut g, p, "Shock", Zone::Hand);
            let foreign = named(&mut g, p.other(), "Forest", Zone::Hand);
            let before = g.state.players[p.index()].library.len();
            paid(&mut g, p, "Lessons from Life", &[]);
            until(&mut g, |g| {
                g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::DeclineHandEntry))
            });
            assert_eq!(g.state.players[p.index()].library.len(), before - 3);
            assert!(g.observe(p.other()).actions.is_empty());
            for card in [nonland, foreign] {
                let object = g.object_ref(card);
                reject(&mut g, Action::PutHandLand { object });
            }
            if accept {
                let object = g.object_ref(land);
                checked(&mut g, Action::PutHandLand { object });
            } else {
                checked(&mut g, Action::DeclineHandEntry);
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[land as usize].zone,
                if accept {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
            if accept {
                assert!(g.state.objects[land as usize].tapped);
            }
        }
    }
}
#[test]
fn tmt_inherited_two_make_your_move_artifact_enchantment_power_boundary_and_recheck() {
    for name in ["Buzz Bots", "Leader's Talent", "Grasping Longneck"] {
        let p = Player::P0;
        let mut g = setup(p);
        let target = named(&mut g, p.other(), name, Zone::Battlefield);
        paid(&mut g, p, "Make Your Move", &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
    }
    let p = Player::P0;
    let mut g = setup(p);
    let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
    let big = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    g.set_counters(big, Counters::of(CounterKind::PlusOne, 2));
    let spell = named(&mut g, p, "Make Your Move", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: spell });
    for card in [bear, land] {
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(big),
        },
    );
    until(&mut g, |g| g.state.announcing.is_none());
    // A real response shrinks the only target below power4.
    let response = named(&mut g, p, "Last Gasp", Zone::Hand);
    cast(&mut g, response, None, &[Target::Card(big)]);
    settle(&mut g);
    assert_eq!(g.state.objects[big as usize].zone, Zone::Battlefield);
}
#[test]
fn tmt_inherited_two_mechanized_both_hybrid_payments_robot_token_complete_recipe() {
    for mana in [[2, 0, 0, 0, 0, 0], [0, 0, 0, 2, 0, 0]] {
        let p = Player::P0;
        let mut g = setup(p);
        g.state.players[p.index()].mana = mana;
        let cavalry = paid(&mut g, p, "Mechanized Ninja Cavalry", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card.has_subtype("Robot")
            })
            .unwrap()
            .0 as CardId;
        assert_eq!((g.stats(cavalry).power, g.stats(cavalry).toughness), (1, 1));
        assert_eq!((g.stats(token).power, g.stats(token).toughness), (1, 1));
        assert_eq!(g.current_kinds(token), 17);
        assert_eq!(g.state.objects[token as usize].card.colors(), 0);
    }
}
#[test]
fn tmt_inherited_two_michelangelo_add_one_only_own_creature_plus_counters_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mikey = paid(&mut g, p, "Michelangelo, Weirdness to 11", &[]);
        settle(&mut g);
        assert_eq!(count_subtype(&g, p, "Mutagen"), 1);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype("Mutagen")
            })
            .unwrap()
            .0 as CardId;
        activate(&mut g, token, &[Target::Card(mikey)]);
        settle(&mut g);
        assert_eq!(g.counter_count(mikey, CounterKind::PlusOne), 2);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        paid(&mut g, p, "Crustacean Commando", &[]);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype("Mutagen")
            })
            .unwrap()
            .0 as CardId;
        activate(&mut g, token, &[Target::Card(enemy)]);
        settle(&mut g);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 1);
        // The replacement adds one per event, not one per existing counter.
        paid(&mut g, p, "Crustacean Commando", &[]);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype("Mutagen")
            })
            .unwrap()
            .0 as CardId;
        activate(&mut g, token, &[Target::Card(mikey)]);
        g.move_object(mikey, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.counter_count(mikey, CounterKind::PlusOne), 0);
    }
}

#[test]
fn tmt_inherited_two_michelangelo_multi_counter_event_noncreature_and_other_kind() {
    let p = Player::P0;
    let mut g = setup(p);
    let mikey = paid(&mut g, p, "Michelangelo, Weirdness to 11", &[]);
    settle(&mut g);
    let don = paid(&mut g, p, "Donatello, Mutant Mechanic", &[]);
    settle(&mut g);
    let robot = paid(&mut g, p, "Mechanized Ninja Cavalry", &[]);
    settle(&mut g);
    let artifact = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| {
            o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype("Mutagen")
        })
        .unwrap()
        .0 as CardId;
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    activate(&mut g, don, &[Target::Card(robot)]);
    settle(&mut g);
    assert_eq!(g.counter_count(robot, CounterKind::PlusOne), 4);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    activate(&mut g, don, &[Target::Card(artifact)]);
    settle(&mut g);
    assert_eq!(g.counter_count(artifact, CounterKind::PlusOne), 3);
    assert!(g.is_creature(artifact));
    next_turn(&mut g);
    own_main(&mut g, p.other());
    paid(&mut g, p.other(), "Freeze in Place", &[Target::Card(mikey)]);
    settle(&mut g);
    assert_eq!(g.counter_count(mikey, CounterKind::Stun), 3);
    assert_eq!(g.counter_count(mikey, CounterKind::PlusOne), 0);
}
