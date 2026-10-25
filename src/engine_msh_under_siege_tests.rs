//! One complete Saga and its effective-subtype simultaneous damage predicate.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
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
fn paid(g: &mut Game, p: Player) -> CardId {
    let source = named(g, p, "Avengers: Under Siege", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 2];
    cast(g, source, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    source
}
fn next_chapter(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    g.begin_step(Step::Main1);
    g.prepare_triggers();
}
fn stack_chapter(g: &mut Game, source: CardId, ability: u16) {
    for _ in 0..30 {
        if g.state
            .stack
            .iter()
            .any(|s| s.card == source && s.kind == StackKind::TriggeredAbility { ability })
        {
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
            .unwrap();
        checked(g, a);
    }
    panic!("no chapter on stack");
}
fn set_types(g: &mut Game, id: CardId, types: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    assert!(timestamp > 0);
    g.state
        .roles
        .get_or_insert_default()
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types,
            additional: false,
            timestamp,
        });
}
fn animate(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_default()
        .animations
        .push(characteristic_layers::AnimationView {
            object,
            timestamp,
            artifact: false,
            preserve_card_types: false,
            colors: None,
            creature_types: None,
            all_creature_types: false,
            base: Some((4, 4)),
            keywords: Keywords::NONE,
            expires_turn: None,
        });
}
fn visible(g: &Game, p: Player, id: CardId) -> bool {
    g.observe(p).objects.iter().any(|o| o.reference.card == id)
}

#[test]
fn msh_under_siege_paid_all_chapters_natural_turns_exact_tokens_and_treasure_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = paid(&mut g, p);
        assert_eq!(g.counter_count(source, CounterKind::Lore), 1);
        let villains = tokens(&g, p, "Villain");
        assert_eq!(villains.len(), 2);
        for &id in &villains {
            let v = g.object_view(id);
            assert_eq!((v.power, v.toughness, v.kinds, v.colors), (2, 1, 1, 4));
            assert!(g.keywords(id).contains(Keywords::MENACE));
            assert!(!v.tapped);
            assert!(v.summoning_sick);
        }
        // The first main phase after a real intervening opponent turn advances II.
        next_turn(&mut g);
        next_turn(&mut g);
        for _ in 0..100 {
            if g.counter_count(source, CounterKind::Lore) >= 2 {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Lore), 2);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(tokens(&g, p, "Villain").len(), 2);
        next_turn(&mut g);
        next_turn(&mut g);
        for _ in 0..100 {
            if g.state.objects[source as usize].zone != Zone::Battlefield {
                break;
            }
            checked(&mut g, Action::Pass);
            settle(&mut g);
        }
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        let treasure = tokens(&g, p, "Treasure");
        assert_eq!(treasure.len(), 2);
        let token = treasure[0];
        let v = g.object_view(token);
        assert_eq!((v.kinds, v.colors), (16, 0));
        for color in 0..5 {
            let mut branch = g.clone();
            branch.state.priority = p;
            branch.state.pending = Pending::Priority;
            let mut production = [0; 6];
            production[color] = 1;
            checked(
                &mut branch,
                Action::ActivateMana {
                    card: token,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production,
                },
            );
            assert_eq!(branch.state.players[p.index()].mana[color], 1);
            assert!(!branch.on_battlefield(token));
        }
    }
}

#[test]
fn msh_under_siege_live_types_include_tokens_artifacts_animated_lands_and_changeling() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p);
        let plain = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let changed = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let artifact = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        animate(&mut g, artifact);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        animate(&mut g, land);
        let land_villain = named(&mut g, p, "Forest", Zone::Battlefield);
        animate(&mut g, land_villain);
        set_types(&mut g, land_villain, &["Villain"]);
        let skrull = named(&mut g, p.other(), "Undercover Skrull", Zone::Battlefield);
        for _ in 0..2 {
            named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        }
        assert!(g.has_subtype(skrull, "Villain"));
        let all_types = named(&mut g, p.other(), "Barkform Harvester", Zone::Battlefield);
        assert!(
            !g.state.objects[all_types as usize]
                .card
                .info()
                .type_line
                .contains("Villain")
        );
        assert!(g.has_subtype(all_types, "Villain"));
        next_chapter(&mut g, p);
        stack_chapter(&mut g, source, 1);
        // These effective types change after the damage packet was queued.
        set_types(&mut g, changed, &["Villain"]);
        let lost = tokens(&g, p, "Villain")[0];
        set_types(&mut g, lost, &["Hero"]);
        settle(&mut g);
        assert_eq!(g.state.objects[plain as usize].damage, 2);
        assert_eq!(g.state.objects[artifact as usize].damage, 2);
        assert_eq!(g.state.objects[land as usize].damage, 2);
        assert_eq!(g.state.objects[land_villain as usize].damage, 0);
        assert_eq!(g.state.objects[changed as usize].damage, 0);
        assert_eq!(g.state.objects[skrull as usize].damage, 0);
        assert_eq!(g.state.objects[all_types as usize].damage, 0);
        assert!(!g.on_battlefield(lost));
    }
}

#[test]
fn msh_under_siege_chapter_three_counts_current_kindred_villains_once_and_captured_controller() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p);
            next_chapter(&mut g, p);
            settle(&mut g);
            next_chapter(&mut g, p);
            stack_chapter(&mut g, source, 2);
            let kindred = named(&mut g, p, "Clachan Festival", Zone::Battlefield);
            set_types(&mut g, kindred, &["Villain"]);
            assert_eq!(g.current_kinds(kindred), 32 | 128);
            assert!(g.has_subtype(kindred, "Villain"));
            let removed = tokens(&g, p, "Villain")[0];
            g.move_object(removed, Zone::Exile);
            let foreign = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            set_types(&mut g, foreign, &["Villain"]);
            if depart {
                g.move_object(source, Zone::Exile);
            } else {
                let r = g.object_ref(source);
                g.change_controller(r, p.other());
            }
            settle(&mut g);
            assert_eq!(tokens(&g, p, "Treasure").len(), 2); // one token + one Kindred
            assert!(tokens(&g, p.other(), "Treasure").is_empty());
        }
    }
}

#[test]
fn msh_under_siege_simultaneous_lifelink_damage_is_one_life_event_with_opponent_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p);
        let a = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let pride = named(&mut g, p, "Ajani's Pridemate", Zone::Battlefield);
        g.state.objects[pride as usize].growth.toughness = 2;
        let tyrant = named(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        g.grant_keyword_effects(&[source], g.object_ref(source), Keywords::LIFELINK);
        next_chapter(&mut g, p);
        settle(&mut g);
        // Turtle, Pridemate and Tyrant take2, opposing Turtle and opponent take4.
        assert_eq!(g.state.objects[a as usize].damage, 2);
        assert_eq!(g.state.objects[b as usize].damage, 4);
        assert_eq!(g.state.objects[tyrant as usize].damage, 2);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.players[p.index()].life, 34);
        // One source deals all14 simultaneously: one life-gain event, one trigger.
        assert_eq!(g.counter_count(pride, CounterKind::PlusOne), 1);
    }
}

#[test]
fn msh_under_siege_damage_source_current_or_departure_controller_differs_from_ability_controller() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p);
            let a = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
            g.grant_keyword_effects(&[source], g.object_ref(source), Keywords::LIFELINK);
            next_chapter(&mut g, p);
            stack_chapter(&mut g, source, 1);
            let r = g.object_ref(source);
            g.change_controller(r, p.other());
            if depart {
                g.move_object(source, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(g.state.objects[a as usize].damage, 2);
            // Opponent of old ability controller takes2, but the new source controller gains4.
            assert_eq!(g.state.players[p.other().index()].life, 22);
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}

#[test]
fn msh_under_siege_damage_is_untargeted_private_and_actions_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let secret = named(&mut g, p.other(), "Shock", Zone::Hand);
        let hidden = named(&mut g, p.other(), "Hill Giant", Zone::Library);
        let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.grant_keyword_effects(
            &[victim],
            g.object_ref(victim),
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
        );
        let source = named(&mut g, p, "Avengers: Under Siege", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: source });
        g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: source });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        next_chapter(&mut g, p);
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].damage, 2);
        assert!(g.on_battlefield(victim));
        assert!(!visible(&g, p, secret) && !visible(&g, p, hidden));
        assert!(visible(&g, p.other(), secret));
        assert!(!visible(&g, p.other(), hidden));
    }
}

#[test]
fn msh_under_siege_multiple_lore_crossings_replacement_and_final_packet_lifetime() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let source = paid(&mut g, p);
        // One entry lore becomes two and crosses I+II; token replacement doubles I.
        assert_eq!(g.counter_count(source, CounterKind::Lore), 2);
        assert_eq!(tokens(&g, p, "Villain").len(), 4);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        next_chapter(&mut g, p);
        stack_chapter(&mut g, source, 2);
        assert!(g.on_battlefield(source));
        // Final chapter survives departure and creates four doubled Treasures.
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Treasure").len(), 8);
    }
}

#[test]
fn msh_under_siege_suppressed_before_counter_event_is_quiet_queued_packet_survives() {
    for p in [Player::P0, Player::P1] {
        for before in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p);
            animate(&mut g, source);
            if !before {
                next_chapter(&mut g, p);
                stack_chapter(&mut g, source, 1);
            }
            // Ordinary production Aura state; no testing-only characteristic hook.
            let aura = named(&mut g, p, "Witness Protection", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
            assert!(!g.printed_abilities_available(source));
            if before {
                next_chapter(&mut g, p);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if before { 20 } else { 18 }
            );
        }
    }
}

#[test]
fn msh_under_siege_villain_tokens_menace_is_actual_block_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p);
        let token = tokens(&g, p, "Villain")[0];
        let a = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: token });
        g.state.objects[token as usize].summoning_sick = false;
        checked(&mut g, Action::Attack { card: token });
        checked(&mut g, Action::FinishAttackers);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::Blockers) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: token,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: token,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}

#[test]
fn msh_under_siege_created_treasure_funds_real_spell_and_cost_departure_is_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p);
        next_chapter(&mut g, p);
        settle(&mut g);
        next_chapter(&mut g, p);
        settle(&mut g);
        let treasure = tokens(&g, p, "Treasure")[0];
        g.state.players[p.index()].mana = [0; 6];
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: shock })
        );
        checked(&mut g, Action::AnnounceCast { card: shock });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::ActivateMana {
                card: treasure,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert!(!g.on_battlefield(treasure));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
