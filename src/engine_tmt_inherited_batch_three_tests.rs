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
fn tmt_inherited_three_mighty_mutanimals_own_token_triggers_alliance_self_or_other_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Mighty Mutanimals", &[]);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
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
                target: Target::Card(source),
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
            .unwrap()
            .0 as CardId;
        assert_eq!((g.stats(token).power, g.stats(token).toughness), (2, 2));
        assert!(g.has_subtype(token, "Mutant"));
        assert_eq!(g.state.objects[token as usize].card.colors(), 8);
        paid(&mut g, p, "Mechanized Ninja Cavalry", &[]);
        settle_on(&mut g, Some(Target::Card(token)));
        assert_eq!(g.counter_count(token, CounterKind::PlusOne), 2);
        next_turn(&mut g);
        own_main(&mut g, p.other());
        paid(&mut g, p.other(), "Grizzly Bears", &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
    }
}
#[test]
fn tmt_inherited_three_mouser_attack_modes_robot_recipe_and_first_strike_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spell = named(&mut g, p, "Mouser Attack!", Zone::Hand);
        cast(&mut g, spell, Some(0), &[]);
        settle(&mut g);
        let robot = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
            .unwrap()
            .0 as CardId;
        assert_eq!((g.stats(robot).power, g.stats(robot).toughness), (1, 1));
        assert_eq!(g.current_kinds(robot), 17);
        assert_eq!(g.state.objects[robot as usize].card.colors(), 0);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let blocker = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let spell = named(&mut g, p, "Mouser Attack!", Zone::Hand);
        cast(&mut g, spell, Some(1), &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (5, 2));
        assert!(g.keywords(bear).contains(Keywords::FIRST_STRIKE));
        combat(&mut g, bear);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: bear,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.state.objects[blocker as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
        next_turn(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
        assert!(!g.keywords(bear).contains(Keywords::FIRST_STRIKE));
    }
}
#[test]
fn tmt_inherited_three_mouser_mark_hybrid_other_artifact_attack_only_restriction() {
    for color in [1, 3] {
        let p = Player::P0;
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.index()].mana[color] = 2;
        let mouser = paid(&mut g, p, "Mouser Mark III", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        named(
            &mut g,
            p.other(),
            "@token/283fd83a9aec83a51b736926",
            Zone::Battlefield,
        );
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        reject(&mut g, Action::Attack { card: mouser });
        let artifact = named(
            &mut g,
            p,
            "@token/283fd83a9aec83a51b736926",
            Zone::Battlefield,
        );
        checked(&mut g, Action::Attack { card: mouser });
        checked(&mut g, Action::FinishAttackers);
        g.move_object(artifact, Zone::Graveyard);
        // Restriction is tested at declaration, removing the enabling artifact cannot undo an attack.
        assert!(g.state.attackers.iter().any(|a| a.card == mouser));
    }
}
#[test]
fn tmt_inherited_three_musicians_alliance_stacks_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Mutant Town Musicians", &[]);
        settle(&mut g);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (2, 4));
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
        paid(&mut g, p, "Mechanized Ninja Cavalry", &[]);
        settle(&mut g);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (4, 4));
        next_turn(&mut g);
        own_main(&mut g, p.other());
        paid(&mut g, p.other(), "Grizzly Bears", &[]);
        settle(&mut g);
        assert_eq!(g.stats(source).power, 2);
    }
}
#[test]
fn tmt_inherited_three_negate_rejects_creature_spells_and_counters_own_or_enemy_noncreatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = paid(&mut g, p, "Grizzly Bears", &[]);
        let creature_spell = g.state.stack.last().unwrap().card;
        let shock = paid(&mut g, p, "Shock", &[Target::Player(p.other())]);
        let shock_spell = g.state.stack.last().unwrap().card;
        checked(&mut g, Action::Pass);
        let negate = named(&mut g, p.other(), "Negate", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: negate });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(creature_spell),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(shock_spell),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.state.objects[shock as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn tmt_inherited_three_null_group_optional_private_discard_draw_and_turn_first_strike() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Null Group Biological Assets", &[]);
            settle(&mut g);
            assert!(g.keywords(source).contains(Keywords::FIRST_STRIKE));
            next_turn(&mut g);
            assert!(!g.keywords(source).contains(Keywords::FIRST_STRIKE));
            next_turn(&mut g);
            own_main(&mut g, p);
            let hand = named(&mut g, p, "Shock", Zone::Hand);
            let foreign = named(&mut g, p.other(), "Island", Zone::Hand);
            let before = g.state.players[p.index()].library.len();
            until(&mut g, |g| g.state.pending == Pending::Attackers);
            checked(&mut g, Action::Attack { card: source });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::DeclineEffectPayment)
            });
            assert!(g.observe(p.other()).actions.is_empty());
            reject(&mut g, Action::Discard { card: foreign });
            if discard {
                checked(&mut g, Action::Discard { card: hand });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                before - usize::from(discard)
            );
            assert_eq!(
                g.state.objects[hand as usize].zone,
                if discard { Zone::Graveyard } else { Zone::Hand }
            );
        }
    }
}
#[test]
fn tmt_inherited_three_omni_cheese_draw_paid_each_color_sacrifice_and_food_alternative() {
    for color in 0..5 {
        let p = Player::P0;
        let mut g = setup(p);
        let before = g.state.players[p.index()].library.len();
        let pizza = paid(&mut g, p, "Omni-Cheese Pizza", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        let mut production = [0; 6];
        production[color] = 1;
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::ActivateMana {
                card: pizza,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production,
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut g,
            Action::ActivateMana {
                card: pizza,
                ability: 0,
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
                production,
            },
        );
        assert_eq!(g.state.players[p.index()].mana, production);
        assert!(g.state.stack.is_empty());
        assert_eq!(g.state.objects[pizza as usize].zone, Zone::Graveyard);
    }
    let p = Player::P1;
    let mut g = setup(p);
    let pizza = paid(&mut g, p, "Omni-Cheese Pizza", &[]);
    settle(&mut g);
    let life = g.state.players[p.index()].life;
    activate_ability(&mut g, pizza, 1, &[]);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, life + 3);
    assert_eq!(g.state.objects[pizza as usize].zone, Zone::Graveyard);
}
#[test]
fn tmt_inherited_three_ooze_spill_counters_any_spell_then_mutagen_even_uncounterable() {
    for name in ["Grizzly Bears", "Sphinx of the Final Word"] {
        let p = Player::P0;
        let mut g = setup(p);
        let victim = paid(&mut g, p, name, &[]);
        let spell = g.state.stack.last().unwrap().card;
        checked(&mut g, Action::Pass);
        paid(&mut g, p.other(), "Ooze Spill", &[Target::Spell(spell)]);
        settle(&mut g);
        assert_eq!(
            g.state.objects[victim as usize].zone,
            if name == "Sphinx of the Final Word" {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(count_subtype(&g, p.other(), "Mutagen"), 1);
    }
}
#[test]
fn tmt_inherited_three_ooze_spill_all_illegal_spell_target_fizzles_token() {
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Grizzly Bears", &[]);
    let spell = g.state.stack.last().unwrap().card;
    checked(&mut g, Action::Pass);
    paid(&mut g, p.other(), "Ooze Spill", &[Target::Spell(spell)]);
    checked(&mut g, Action::Pass);
    paid(&mut g, p, "Ooze Spill", &[Target::Spell(spell)]);
    settle(&mut g);
    assert_eq!(count_subtype(&g, p.other(), "Mutagen"), 0);
}
#[test]
fn tmt_inherited_three_prehistoric_pet_turn_tap_cost_other_owned_only_and_bounce_while_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pet = paid(&mut g, p, "Prehistoric Pet", &[]);
        settle(&mut g);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: pet,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        next_turn(&mut g);
        own_main(&mut g, p.other());
        checked(&mut g, Action::Pass);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: pet,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        next_turn(&mut g);
        own_main(&mut g, p);
        until(&mut g, |g| {
            g.state.step == Step::BeginCombat && g.state.pending == Pending::Priority
        });
        g.state.players[p.index()].mana = [30; 6];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: pet,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for card in [pet, enemy] {
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
                target: Target::Card(own),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        assert!(g.state.objects[pet as usize].tapped);
        settle(&mut g);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Hand);
    }
}
#[test]
fn tmt_inherited_three_prehistoric_pet_actual_evasion_equal_power_can_block() {
    let p = Player::P0;
    let mut g = setup(p);
    let pet = paid(&mut g, p, "Prehistoric Pet", &[]);
    settle(&mut g);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    let big = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let equal = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
    combat(&mut g, pet);
    reject(
        &mut g,
        Action::Block {
            blocker: big,
            attacker: pet,
        },
    );
    checked(
        &mut g,
        Action::Block {
            blocker: equal,
            attacker: pet,
        },
    );
}
#[test]
fn tmt_inherited_three_primordial_life_reach_blocks_actual_flyer_and_trample() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let before = g.state.players[p.index()].life;
        let elephant = paid(&mut g, p, "Primordial Pachyderm", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, before + 2);
        assert!(
            g.keywords(elephant)
                .contains(Keywords::TRAMPLE.union(Keywords::REACH))
        );
        next_turn(&mut g);
        own_main(&mut g, p.other());
        let flyer = named(&mut g, p.other(), "Buzz Bots", Zone::Battlefield);
        combat(&mut g, flyer);
        checked(
            &mut g,
            Action::Block {
                blocker: elephant,
                attacker: flyer,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.state.objects[flyer as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[elephant as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn tmt_inherited_three_punk_frogs_hybrid_mana_ward_paid_declined_and_own_spell_no_ward() {
    for mana in [
        [0, 0, 0, 0, 5, 0],
        [0, 1, 0, 0, 4, 0],
        [0, 2, 0, 0, 3, 0],
        [0, 5, 0, 0, 0, 0],
    ] {
        for pay in [false, true] {
            let p = Player::P0;
            let q = p.other();
            let mut g = setup(p);
            g.state.players[p.index()].mana = mana;
            let frogs = paid(&mut g, p, "Punk Frogs", &[]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            checked(&mut g, Action::Pass);
            paid(&mut g, q, "Murder", &[Target::Card(frogs)]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::DeclineResolutionMana)
            });
            assert_eq!(g.actor(), Some(q));
            let before = g.state.players[q.index()].mana.iter().sum::<u32>();
            let action = if pay {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                    .unwrap()
            } else {
                Action::DeclineResolutionMana
            };
            checked(&mut g, action);
            settle(&mut g);
            assert_eq!(
                g.state.players[q.index()].mana.iter().sum::<u32>(),
                before - if pay { 3 } else { 0 }
            );
            assert_eq!(
                g.state.objects[frogs as usize].zone,
                if pay {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
    let p = Player::P1;
    let mut g = setup(p);
    let frogs = paid(&mut g, p, "Punk Frogs", &[]);
    settle(&mut g);
    paid(&mut g, p, "Murder", &[Target::Card(frogs)]);
    settle(&mut g);
    assert_eq!(g.state.objects[frogs as usize].zone, Zone::Graveyard);
}
#[test]
fn tmt_inherited_three_raphael_alliance_each_creature_and_opponent_only_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Raphael, Tough Turtle", &[]);
        settle(&mut g);
        let before = g.state.players[p.other().index()].life;
        paid(&mut g, p, "Mechanized Ninja Cavalry", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before - 2);
        next_turn(&mut g);
        own_main(&mut g, p.other());
        let before = g.state.players[p.other().index()].life;
        paid(&mut g, p.other(), "Grizzly Bears", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before);
    }
}
#[test]
fn tmt_inherited_three_ravenous_cast_artifact_trigger_and_haste_frozen_recipients() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Ravenous Robots", &[]);
        settle(&mut g);
        assert_eq!(count_subtype(&g, p, "Robot"), 1);
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        let spell = paid(&mut g, p, "Omni-Cheese Pizza", &[]);
        settle(&mut g);
        assert_eq!(count_subtype(&g, p, "Robot"), 2);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
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
        let foreign = named(
            &mut g,
            p.other(),
            "@token/667dad32c9f20b70d5e9bda3",
            Zone::Battlefield,
        );
        activate(&mut g, source, &[]);
        settle(&mut g);
        assert!(g.keywords(token).contains(Keywords::HASTE));
        assert!(!g.keywords(source).contains(Keywords::HASTE));
        assert!(!g.keywords(foreign).contains(Keywords::HASTE));
        let instant = named(&mut g, p, "Mouser Attack!", Zone::Hand);
        cast(&mut g, instant, Some(0), &[]);
        settle(&mut g);
        let late = g
            .state
            .objects
            .iter()
            .enumerate()
            .rfind(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card.has_subtype("Robot")
            })
            .unwrap()
            .0 as CardId;
        assert!(!g.keywords(late).contains(Keywords::HASTE));
        combat(&mut g, token);
        assert!(g.state.objects[token as usize].tapped);
        next_turn(&mut g);
        assert!(!g.keywords(token).contains(Keywords::HASTE));
    }
}
#[test]
fn tmt_inherited_three_sewers_owner_choice_top_bottom_and_mutagen_for_spell_controller() {
    for p in [Player::P0, Player::P1] {
        for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
            let q = p.other();
            let mut g = setup(p);
            let victim = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
            g.state.objects[victim as usize].controller = p;
            paid(&mut g, p, "Return to the Sewers", &[Target::Card(victim)]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::TargetLibrary { .. })
            });
            assert_eq!(g.actor(), Some(q));
            assert!(g.observe(p).actions.is_empty());
            let object = g.object_ref(victim);
            checked(
                &mut g,
                Action::ChooseLibraryDestination {
                    object,
                    destination,
                },
            );
            settle(&mut g);
            assert_eq!(g.state.objects[victim as usize].zone, Zone::Library);
            let library = &g.state.players[q.index()].library;
            assert_eq!(
                if destination == LibraryDestination::Top {
                    library.last()
                } else {
                    library.first()
                },
                Some(&victim)
            );
            assert_eq!(count_subtype(&g, p, "Mutagen"), 1);
            assert_eq!(count_subtype(&g, q, "Mutagen"), 0);
        }
    }
}
#[test]
fn tmt_inherited_three_sewers_all_invalid_target_no_token_and_token_target_ceases() {
    let p = Player::P0;
    let mut g = setup(p);
    let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    paid(&mut g, p, "Return to the Sewers", &[Target::Card(victim)]);
    g.move_object(victim, Zone::Hand);
    settle(&mut g);
    assert_eq!(count_subtype(&g, p, "Mutagen"), 0);
    let token = named(
        &mut g,
        p.other(),
        "@token/667dad32c9f20b70d5e9bda3",
        Zone::Battlefield,
    );
    paid(&mut g, p, "Return to the Sewers", &[Target::Card(token)]);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::TargetLibrary { .. })
    });
    let object = g.object_ref(token);
    checked(
        &mut g,
        Action::ChooseLibraryDestination {
            object,
            destination: LibraryDestination::Top,
        },
    );
    settle(&mut g);
    assert_eq!(count_subtype(&g, p, "Mutagen"), 1);
    assert!(!g.state.players[p.other().index()].library.contains(&token));
}
#[test]
fn tmt_inherited_three_rock_soldiers_optional_any_controller_noncreature_artifact_only() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = setup(p);
            let food = named(
                &mut g,
                p.other(),
                "@token/283fd83a9aec83a51b736926",
                Zone::Battlefield,
            );
            let creature = named(&mut g, p.other(), "Buzz Bots", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            let source = paid(&mut g, p, "Rock Soldiers", &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            for card in [source, creature, land] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(card),
                    },
                );
            }
            if choose {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(food),
                    },
                );
            } else {
                checked(&mut g, Action::FinishTargets);
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[food as usize].zone == Zone::Battlefield,
                !choose
            );
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (4, 3));
        }
    }
}

#[test]
fn tmt_inherited_three_null_group_empty_hand_attack_never_draws() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = paid(&mut g, p, "Null Group Biological Assets", &[]);
    settle(&mut g);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    let hand: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Hand && o.controller == p)
        .map(|(i, _)| i as CardId)
        .collect();
    for card in hand {
        g.move_object(card, Zone::Graveyard);
    }
    let before = g.state.players[p.index()].library.len();
    combat(&mut g, source);
    assert_eq!(g.state.players[p.index()].library.len(), before);
}
