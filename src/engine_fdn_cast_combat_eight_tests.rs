use super::tests::{act, scenario, spawn};
use super::*;
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn replay(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|b| *b == a)
        .expect("legal");
    let before = g.snapshot();
    assert_eq!(g.apply(p.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), before);
    let mut raw = {
        let parent = g.clone();
        parent.clone()
    };
    let mut indexed = {
        let parent = g.clone();
        parent.clone()
    };
    raw.apply(p, a.clone()).unwrap();
    indexed.apply_index(p, revision, index).unwrap();
    act(g, a.clone());
    assert_eq!(g.snapshot(), raw.snapshot());
    assert_eq!(g.snapshot(), indexed.snapshot());
    for viewer in [p, p.other()] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(indexed.observe(viewer)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, revision, index), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn pay(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .expect("payment");
        replay(g, a);
    }
}
fn settle_choice(g: &mut Game, accept: bool) {
    g.prepare_triggers();
    for _ in 0..200 {
        if g.outcome().is_some()
            || g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.announcing.is_none()
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| match a {
                Action::ChooseOptional { accept: a } => *a == accept,
                Action::PutTrigger { .. }
                | Action::Pass
                | Action::FinishTargets
                | Action::PayCosts { .. } => true,
                _ => false,
            })
            .expect("settle action");
        replay(g, a);
    }
    panic!("unsettled");
}
fn settle(g: &mut Game) {
    settle_choice(g, true);
}
fn cast(g: &mut Game, p: Player, n: &str) -> CardId {
    let id = named(g, p, n, Zone::Hand);
    replay(g, Action::AnnounceCast { card: id });
    pay(g);
    settle(g);
    id
}
fn announce(g: &mut Game, p: Player, n: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, n, Zone::Hand);
    replay(g, Action::AnnounceCast { card: id });
    for &target in targets {
        replay(g, Action::ChooseTarget { target });
    }
    pay(g);
    id
}
fn queued(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|item| item.card == id)
        .count()
}
fn blank(g: &mut Game, p: Player, id: CardId) -> CardId {
    let aura = named(g, p, "Stop Cold", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(id));
    aura
}
#[test]
fn fdn_cast_combat_eight_paid_printed_costs_and_intrinsics() {
    for p in [Player::P0, Player::P1] {
        for (name, total) in [
            ("Mindsparker", 3),
            ("Mold Adder", 1),
            ("Taurean Mauler", 3),
            ("Ramos, Dragon Engine", 6),
            ("Stromkirk Noble", 1),
            ("Slumbering Cerberus", 2),
            ("Quick-Draw Katana", 2),
            ("Linden, the Steadfast Queen", 3),
        ] {
            let mut g = setup(p);
            let before = g.state.players[p.index()].mana.iter().sum::<u32>();
            let id = cast(&mut g, p, name);
            assert_eq!(
                before - g.state.players[p.index()].mana.iter().sum::<u32>(),
                total,
                "{name}"
            );
            assert!(g.on_battlefield(id));
            if name == "Taurean Mauler" {
                assert!(g.has_subtype(id, "Human"));
                assert!(g.has_subtype(id, "Dragon"));
            }
        }
    }
}
#[test]
fn fdn_cast_combat_eight_mindsparker_color_kind_controller_saved_caster_and_removal() {
    for p in [Player::P0, Player::P1] {
        for (name, hits) in [
            ("Think Twice", true),
            ("Raise the Past", true),
            ("Shock", false),
            ("Mold Adder", false),
            ("Linden, the Steadfast Queen", false),
            ("Mindsparker", false),
        ] {
            let mut g = setup(p.other());
            let source = named(&mut g, p, "Mindsparker", Zone::Battlefield);
            let targets = if name == "Shock" {
                vec![Target::Player(p)]
            } else {
                vec![]
            };
            announce(&mut g, p.other(), name, &targets);
            assert_eq!(queued(&g, source), usize::from(hits), "{name}");
            if hits {
                g.move_object(source, Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if hits { 18 } else { 20 }
            );
        }
        let mut g = setup(p);
        let source = named(&mut g, p, "Mindsparker", Zone::Battlefield);
        announce(&mut g, p, "Think Twice", &[]);
        assert_eq!(queued(&g, source), 0);
        settle(&mut g);
        g.state.active = p.other();
        g.state.priority = p.other();
        let aura = blank(&mut g, p.other(), source);
        announce(&mut g, p.other(), "Think Twice", &[]);
        assert_eq!(queued(&g, source), 0);
        settle(&mut g);
        g.move_object(aura, Zone::Graveyard);
        announce(&mut g, p.other(), "Think Twice", &[]);
        assert_eq!(queued(&g, source), 1);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn fdn_cast_combat_eight_adder_mauler_optional_single_multicolor_and_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            for (name, adder) in [("Mold Adder", true), ("Taurean Mauler", false)] {
                let mut g = setup(p.other());
                let source = named(&mut g, p, name, Zone::Battlefield);
                announce(&mut g, p.other(), "Trygon Predator", &[]);
                assert_eq!(queued(&g, source), 1);
                settle_choice(&mut g, accept);
                assert_eq!(
                    g.state.objects[source as usize].counters.plus_one,
                    u64::from(accept)
                );
                announce(&mut g, p.other(), "Quick-Draw Katana", &[]);
                assert_eq!(queued(&g, source), usize::from(!adder));
                settle_choice(&mut g, accept);
                let before = g.state.objects[source as usize].counters.plus_one;
                announce(&mut g, p.other(), "Think Twice", &[]);
                assert_eq!(queued(&g, source), 1);
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Battlefield);
                settle_choice(&mut g, accept);
                assert_eq!(g.state.objects[source as usize].counters.plus_one, 0);
                assert!(before <= 2);
            }
        }
    }
}
#[test]
fn fdn_cast_combat_eight_ramos_spell_colors_not_payment_and_mana_counter_cost_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Ramos, Dragon Engine", Zone::Battlefield);
        announce(&mut g, p, "Trygon Predator", &[]);
        assert_eq!(queued(&g, source), 1);
        let saved = g
            .state
            .pending_triggers
            .iter()
            .chain(&g.state.ordering_triggers)
            .chain(&g.state.stack)
            .find(|item| item.card == source)
            .unwrap();
        assert_eq!(g.item_amount(AmountValue::SpellColors, saved), 2);
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 2);
        announce(&mut g, p, "Quick-Draw Katana", &[]);
        assert_eq!(queued(&g, source), 1);
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 2);
        announce(&mut g, p, "Mindsparker", &[]);
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 3);
        let invalid = Action::ActivateMana {
            card: source,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters {
                plus_one: 5,
                ..Counters::default()
            },
            production: [2, 2, 2, 2, 2, 0],
        };
        reject(&mut g, invalid.clone());
        g.state.objects[source as usize].counters.plus_one = 10;
        g.state.players[p.index()].mana = [0; 6];
        replay(&mut g, invalid.clone());
        assert_eq!(g.state.players[p.index()].mana, [2, 2, 2, 2, 2, 0]);
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 5);
        assert!(g.state.stack.is_empty());
        reject(&mut g, invalid);
        g.finish_cleanup();
        g.state.active = p.other();
        g.state.priority = p.other();
        replay(&mut g, Action::Pass); // Ramos may activate on the opponent's turn, without tapping.
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==source))
            .unwrap();
        replay(&mut g, action);
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 0);
    }
}
#[test]
fn fdn_cast_combat_eight_ramos_funds_announcement_and_rejects_wrong_counter_kind() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = named(&mut g, p, "Ramos, Dragon Engine", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        g.state.objects[r as usize].counters.stun = 5;
        let card = named(&mut g, p, "Linden, the Steadfast Queen", Zone::Hand);
        assert!(!g.legal_actions().contains(&Action::AnnounceCast { card }));
        g.state.objects[r as usize].counters.plus_one = 5;
        // Linden requires three white, and Ramos produces only two; do not invent funding.
        assert!(!g.legal_actions().contains(&Action::AnnounceCast { card }));
        let cast = named(&mut g, p, "Trygon Predator", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: cast });
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==r))
            .unwrap();
        replay(&mut g, action);
        pay(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(cast));
        assert_eq!(g.state.objects[r as usize].counters.plus_one, 2);
        assert_eq!(g.state.objects[r as usize].counters.stun, 5);
    }
}
#[test]
fn fdn_cast_combat_eight_stromkirk_human_changeling_blockers_and_combat_only_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Stromkirk Noble", Zone::Battlefield);
        let human = named(
            &mut g,
            p.other(),
            "Linden, the Steadfast Queen",
            Zone::Battlefield,
        );
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let changeling = named(&mut g, p.other(), "Taurean Mauler", Zone::Battlefield);
        assert!(!g.can_block_pair(human, source));
        assert!(!g.can_block_pair(changeling, source));
        assert!(g.can_block_pair(bear, source));
        let aura = blank(&mut g, p.other(), source);
        assert!(g.can_block_pair(human, source));
        g.move_object(aura, Zone::Graveyard);
        assert!(!g.can_block_pair(human, source));
        for combat in [false, true] {
            let damage = DamageEvent {
                preventable: true,
                source: g.damage_source(source),
                target: Target::Player(p.other()),
                amount: 1,
                combat,
            };
            g.apply_damage_batch(&[damage]);
            assert_eq!(queued(&g, source), usize::from(combat));
            settle(&mut g);
        }
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 1);
        let damage = DamageEvent {
            preventable: true,
            source: g.damage_source(source),
            target: Target::Player(p.other()),
            amount: 1,
            combat: true,
        };
        g.apply_damage_batch(&[damage]);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 0);
    }
}
#[test]
fn fdn_cast_combat_eight_cerberus_untap_prohibition_removable_and_morbid_each_end() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Slumbering Cerberus", Zone::Battlefield);
        g.state.objects[c as usize].tapped = true;
        g.state.objects[c as usize].counters.stun = 1;
        g.untap_active_player();
        assert!(g.state.objects[c as usize].tapped);
        assert_eq!(g.state.objects[c as usize].counters.stun, 1);
        g.untap_objects(&[c], untap_rules::UntapCause::Effect);
        assert!(g.state.objects[c as usize].tapped);
        assert_eq!(g.state.objects[c as usize].counters.stun, 0);
        g.untap_objects(&[c], untap_rules::UntapCause::Effect);
        assert!(!g.state.objects[c as usize].tapped);
        g.state.objects[c as usize].tapped = true;
        g.begin_step(Step::End);
        assert_eq!(queued(&g, c), 0);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.move_object(bear, Zone::Graveyard);
        g.begin_step(Step::End);
        assert_eq!(queued(&g, c), 1);
        settle(&mut g);
        assert!(!g.state.objects[c as usize].tapped);
        g.state.active = p.other();
        g.state.objects[c as usize].tapped = true;
        g.begin_step(Step::End);
        assert_eq!(queued(&g, c), 1);
        settle(&mut g);
        assert!(!g.state.objects[c as usize].tapped);
        let aura = named(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(c));
        g.state.active = p;
        g.state.objects[c as usize].tapped = true;
        g.untap_active_player();
        assert!(!g.state.objects[c as usize].tapped);
        g.move_object(aura, Zone::Graveyard);
    }
}
#[test]
fn fdn_cast_combat_eight_katana_paid_equip_own_source_turn_and_exact_link() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let katana = cast(&mut g, p, "Quick-Draw Katana");
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: katana,
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
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        pay(&mut g);
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            2
        );
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 4);
        assert!(g.keywords(bear).contains(Keywords::FIRST_STRIKE));
        g.state.active = p.other();
        assert_eq!(g.stats(bear).power, 2);
        assert!(!g.keywords(bear).contains(Keywords::FIRST_STRIKE));
        g.state.objects[bear as usize].controller = p.other();
        g.state.active = p;
        assert_eq!(g.stats(bear).power, 4); // follows Equipment controller, not recipient.
        let aura = blank(&mut g, p.other(), katana);
        assert_eq!(g.stats(bear).power, 2);
        assert!(!g.keywords(bear).contains(Keywords::FIRST_STRIKE));
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(g.stats(bear).power, 4);
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        assert_eq!(g.stats(bear).power, 2);
        assert!(!g.keywords(bear).contains(Keywords::FIRST_STRIKE));
    }
}
#[test]
fn fdn_cast_combat_eight_linden_one_trigger_each_white_attacker_including_self() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let l = named(&mut g, p, "Linden, the Steadfast Queen", Zone::Battlefield);
        let w = named(&mut g, p, "Inspiring Paladin", Zone::Battlefield);
        let green = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.attackers = vec![
            Attacker {
                card: l,
                defender: None,
                blocked: false,
            },
            Attacker {
                card: w,
                defender: None,
                blocked: false,
            },
            Attacker {
                card: green,
                defender: None,
                blocked: false,
            },
        ];
        g.declaration_triggers(p);
        assert_eq!(queued(&g, l), 2);
        g.move_object(l, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}
#[test]
fn fdn_cast_combat_eight_paid_attack_declaration_linden_vigilance_and_first_strike() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let l = named(&mut g, p, "Linden, the Steadfast Queen", Zone::Battlefield);
        let sparker = named(&mut g, p, "Mindsparker", Zone::Battlefield);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        for id in [l, sparker, bear] {
            g.state.objects[id as usize].summoning_sick = false;
        }
        replay(&mut g, Action::Attack { card: l });
        replay(&mut g, Action::Attack { card: sparker });
        replay(&mut g, Action::FinishAttackers);
        assert!(!g.state.objects[l as usize].tapped);
        assert!(g.state.objects[sparker as usize].tapped);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        for _ in 0..8 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            replay(&mut g, Action::Pass);
        }
        replay(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: sparker,
            },
        );
        replay(&mut g, Action::FinishBlockers);
        // Advance actual combat decisions; Mindsparker strikes before the Bear.
        for _ in 0..50 {
            if g.state.step == Step::EndCombat || g.state.step == Step::Main2 {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::AssignDamage { .. }))
                .expect("combat action");
            replay(&mut g, action);
        }
        assert!(g.on_battlefield(sparker));
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[sparker as usize].damage, 0);
    }
}
