//! Typed characteristic projections through paid actions and owned branches.
use super::ability_timeline_tests::{cast, checked, next_turn, settle, setup};
use super::tests::spawn;
use super::*;
use crate::ChoiceColor;
use crate::cards::RoleKind;

fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn aura(g: &mut Game, p: Player, name: &str, target: CardId, mana: Mana) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, &[Target::Card(target)]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    id
}
fn activate(g: &mut Game, p: Player, id: CardId) {
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
    checked(
        g,
        Action::Activate {
            card: id,
            ability: 1,
            target: None,
            payment: [0, 0, 0, 0, 0, 4],
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if stop(g) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing expected decision: {:?}", g.state.pending);
}
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
#[test]
fn layered_characteristics_paid_auras_names_colors_base_abilities_and_frozen_facts() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = fixture(&mut g, p, "Vampire Nighthawk", Zone::Battlefield);
        let original = g.object_view(target);
        assert!(original.keywords.contains(Keywords::FLYING));
        let piranhas = aura(&mut g, p, "Eaten by Piranhas", target, [0, 1, 0, 0, 0, 1]);
        let view = g.object_view(target);
        assert_eq!(view.name(), "Vampire Nighthawk");
        assert_eq!(view.colors, 4);
        assert!(view.has_subtype("Skeleton"));
        assert!(!view.has_subtype("Vampire"));
        assert_eq!(
            g.stats(target),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(g.keywords(target), Keywords::NONE);
        let protection = aura(&mut g, p, "Witness Protection", target, [0, 1, 0, 0, 0, 0]);
        let frozen = g.object_view(target);
        assert_eq!(frozen.name(), "Legitimate Businessperson");
        assert_eq!(frozen.colors, 17);
        assert!(frozen.has_subtype("Citizen"));
        assert!(!frozen.has_subtype("Skeleton"));
        let facts = DamageFacts::from_view(&frozen);
        assert_eq!(facts.colors(), 17);
        let snapshot = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        branch.sacrifice_batch(&[(p, protection)]);
        branch.state_based_actions();
        assert_eq!(branch.object_view(target).name(), "Vampire Nighthawk");
        assert!(branch.has_subtype(target, "Skeleton"));
        branch.sacrifice_batch(&[(p, piranhas)]);
        branch.state_based_actions();
        assert!(branch.keywords(target).contains(Keywords::FLYING));
        assert_eq!(
            branch.stats(target),
            Stats {
                power: 2,
                toughness: 3
            }
        );
        assert_eq!(frozen.name(), "Legitimate Businessperson");
        assert_eq!(frozen.colors, 17);
        assert_eq!(facts.colors(), 17);
        assert_eq!(g.snapshot(), snapshot);
        branch.restore(&snapshot).unwrap();
        assert_eq!(branch.object_view(target), frozen);
        checked(&mut branch, Action::Pass);
    }
}
#[test]
fn layered_characteristics_moon_replaces_intrinsic_mana_and_preserves_land_subtypes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let forest = fixture(&mut g, p, "Forest", Zone::Battlefield);
        let moon = aura(
            &mut g,
            p,
            "Imprisoned in the Moon",
            forest,
            [0, 1, 0, 0, 0, 2],
        );
        assert_eq!(g.current_kinds(forest), 2);
        assert!(g.has_subtype(forest, "Forest"));
        assert_eq!(g.object_view(forest).colors, 0);
        assert!(!g.printed_abilities_available(forest));
        let granted = Action::TapGrantedMana {
            card: forest,
            grantor: g.object_ref(moon),
            production: [0, 0, 0, 0, 0, 1],
        };
        checked(&mut g, granted.clone());
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        reject(&mut g, granted);
        let view = g.object_view(forest);
        assert!(view.tapped);
        g.sacrifice_batch(&[(p, moon)]);
        g.state_based_actions();
        assert!(g.printed_abilities_available(forest));
        assert!(g.object_view(forest).tapped);
        // A planeswalker is a land while enchanted, so zero loyalty cannot kill it.
        let walker = fixture(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 4;
        let moon = aura(
            &mut g,
            p,
            "Imprisoned in the Moon",
            walker,
            [0, 1, 0, 0, 0, 2],
        );
        g.state.objects[walker as usize].counters.loyalty = 0;
        g.state_based_actions();
        assert_eq!(g.object_view(walker).zone, Zone::Battlefield);
        assert_eq!(g.current_kinds(walker), 2);
        g.sacrifice_batch(&[(p, moon)]);
        g.state_based_actions();
        assert_eq!(g.object_view(walker).zone, Zone::Graveyard);
    }
}
#[test]
fn layered_characteristics_barrens_optional_counters_expiry_and_permanent_sanctuary() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let barrens = fixture(&mut g, p, "Crawling Barrens", Zone::Battlefield);
        activate(&mut g, p, barrens);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        assert_eq!(g.counter_count(barrens, CounterKind::PlusOne), 2);
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert_eq!(g.current_kinds(barrens), 2);
        assert!(
            g.state
                .roles
                .as_ref()
                .is_none_or(|r| r.animations.is_empty())
        );
        activate(&mut g, p, barrens);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.current_kinds(barrens), 3);
        assert!(g.has_subtype(barrens, "Elemental"));
        assert_eq!(
            g.stats(barrens),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        let sanctuary = fixture(&mut g, p, "Soulstone Sanctuary", Zone::Battlefield);
        activate(&mut g, p, sanctuary);
        settle(&mut g);
        assert_eq!(
            g.stats(sanctuary),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(g.has_subtype(sanctuary, "Citizen"));
        assert!(g.has_subtype(sanctuary, "Dragon"));
        assert!(g.keywords(sanctuary).contains(Keywords::VIGILANCE));
        next_turn(&mut g);
        assert_eq!(g.current_kinds(barrens), 2);
        assert!(!g.has_subtype(barrens, "Elemental"));
        assert_eq!(g.counter_count(barrens, CounterKind::PlusOne), 4);
        assert_eq!(g.current_kinds(sanctuary), 3);
        assert!(g.has_subtype(sanctuary, "Dragon"));
        assert!(g.keywords(sanctuary).contains(Keywords::VIGILANCE));
        let old = g.object_ref(sanctuary);
        g.move_object(sanctuary, Zone::Hand);
        g.move_object(sanctuary, Zone::Battlefield);
        assert_ne!(g.object_ref(sanctuary), old);
        assert_eq!(g.current_kinds(sanctuary), 2);
        assert!(!g.has_subtype(sanctuary, "Dragon"));
        assert_eq!(g.keywords(sanctuary), Keywords::NONE);
    }
}
#[test]
fn layered_characteristics_sanctuary_animation_timestamps_against_removal_and_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sanctuary = fixture(&mut g, p, "Soulstone Sanctuary", Zone::Battlefield);
        activate(&mut g, p, sanctuary);
        settle(&mut g);
        let witness = aura(
            &mut g,
            p,
            "Witness Protection",
            sanctuary,
            [0, 1, 0, 0, 0, 0],
        );
        assert_eq!(g.current_kinds(sanctuary), 1);
        assert_eq!(
            g.stats(sanctuary),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(g.has_subtype(sanctuary, "Citizen"));
        assert!(!g.has_subtype(sanctuary, "Dragon"));
        assert_eq!(g.keywords(sanctuary), Keywords::NONE);
        let copy = fixture(&mut g, p, "Self-Reflection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        cast(&mut g, copy, None, &[Target::Card(sanctuary)]);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.is_copy_token() && o.zone == Zone::Battlefield)
            .unwrap()
            .0 as CardId;
        assert_eq!(g.object_view(token).name(), "Soulstone Sanctuary");
        assert_eq!(g.current_kinds(token), 2);
        assert!(!g.has_subtype(token, "Citizen"));
        g.sacrifice_batch(&[(p, witness)]);
        g.state_based_actions();
        assert!(g.has_subtype(sanctuary, "Dragon"));
        assert!(g.keywords(sanctuary).contains(Keywords::VIGILANCE));
        // A newer resolved animation restores layer-4 all types but not removed printed abilities.
        activate(&mut g, p, sanctuary);
        let witness = aura(
            &mut g,
            p,
            "Eaten by Piranhas",
            sanctuary,
            [0, 1, 0, 0, 0, 1],
        );
        settle(&mut g);
        assert!(g.has_subtype(sanctuary, "Dragon"));
        assert_eq!(
            g.stats(sanctuary),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(g.keywords(sanctuary).contains(Keywords::VIGILANCE));
        assert!(!g.printed_abilities_available(sanctuary));
        g.sacrifice_batch(&[(p, witness)]);
        g.state_based_actions();
    }
}

#[test]
fn layered_characteristics_moon_funding_and_competing_auras_preserve_typed_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let forest = fixture(&mut g, p, "Forest", Zone::Battlefield);
        let moon = aura(
            &mut g,
            p,
            "Imprisoned in the Moon",
            forest,
            [0, 1, 0, 0, 0, 2],
        );
        let before = g.snapshot();
        let goal = |required| FundingGoal {
            actor: p,
            required,
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(g.can_fund(goal([0, 0, 0, 0, 0, 1])));
        assert!(!g.can_fund(goal([0, 0, 0, 0, 1, 0])));
        assert_eq!(
            g.snapshot(),
            before,
            "speculative mana queries are read-only"
        );
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.sacrifice_batch(&[(p, moon)]);
        child.state_based_actions();
        assert!(child.can_fund(goal([0, 0, 0, 0, 1, 0])));
        assert!(!child.can_fund(goal([0, 0, 0, 0, 0, 1])));
        assert_eq!(g.snapshot(), before);
        child.restore(&before).unwrap();
        assert!(child.can_fund(goal([0, 0, 0, 0, 0, 1])));
        let mascot = fixture(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        assert_eq!(g.current_kinds(mascot), 17);
        assert!(g.has_subtype(mascot, "Dragon"));
        let witness = aura(&mut g, p, "Witness Protection", mascot, [0, 1, 0, 0, 0, 0]);
        assert_eq!(g.current_kinds(mascot), 1);
        assert!(!g.has_subtype(mascot, "Dragon"));
        let moon = aura(
            &mut g,
            p,
            "Imprisoned in the Moon",
            mascot,
            [0, 1, 0, 0, 0, 2],
        );
        assert_eq!(g.current_kinds(mascot), 2);
        assert_eq!(g.object_view(mascot).colors, 0);
        assert!(!g.has_subtype(mascot, "Citizen"));
        assert!(!g.has_subtype(mascot, "Dragon"));
        assert_eq!(
            g.object_view(witness).zone,
            Zone::Graveyard,
            "a creature Aura falls off its now-land recipient"
        );
        assert_eq!(g.object_view(mascot).name(), "Three Tree Mascot");
        g.sacrifice_batch(&[(p, moon)]);
        g.state_based_actions();
        assert_eq!(g.current_kinds(mascot), 17);
        assert!(g.has_subtype(mascot, "Dragon"));
    }
}
#[test]
fn layered_characteristics_source_departure_blocks_animation_and_counter_carryover() {
    for p in [Player::P0, Player::P1] {
        for name in ["Crawling Barrens", "Soulstone Sanctuary"] {
            let mut g = setup(p);
            let land = fixture(&mut g, p, name, Zone::Battlefield);
            activate(&mut g, p, land);
            let old = g.object_ref(land);
            let removal = fixture(&mut g, p.other(), "Demolition Field", Zone::Battlefield);
            checked(&mut g, Action::Pass);
            let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==removal&&*t==land)).unwrap();
            checked(&mut g, action);
            // Resolve the actual destruction, then complete its ordinary search choices.
            for _ in 0..80 {
                if g.object_view(land).zone == Zone::Graveyard {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            assert_eq!(g.object_view(land).zone, Zone::Graveyard);
            settle(&mut g);
            g.move_object(land, Zone::Hand);
            g.move_object(land, Zone::Battlefield);
            assert_ne!(g.object_ref(land), old);
            assert_eq!(g.current_kinds(land), 2);
            assert_eq!(g.counter_count(land, CounterKind::PlusOne), 0);
            assert!(!g.has_subtype(land, "Elemental"));
            assert!(!g.has_subtype(land, "Dragon"));
        }
    }
}

#[test]
fn layered_characteristics_banner_and_convoke_use_current_colors_in_actual_and_funding_worlds() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let banner = fixture(&mut g, p, "Heraldic Banner", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        cast(&mut g, banner, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(ChoiceColor::Green),
            },
        );
        settle(&mut g);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        aura(&mut g, p, "Eaten by Piranhas", bear, [0, 1, 0, 0, 0, 1]);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        aura(&mut g, p, "Witness Protection", bear, [0, 1, 0, 0, 0, 0]);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 2,
                toughness: 1
            }
        );
        assert_eq!(g.convoke_masks(p, &[])[17], 1);
        assert_eq!(g.convoke_masks(p, &[])[16], 0);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 4];
        let spell = fixture(&mut g, p, "Sun-Dappled Celebrant", Zone::Hand);
        let before = g.snapshot();
        assert!(g.can_fund_convoke(FundingGoal {
            actor: p,
            required: [2, 0, 0, 0, 0, 0],
            generic: 4,
            reserve: None,
            source: None
        }));
        assert_eq!(g.snapshot(), before);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::BeginConvoke);
        let object = g.object_ref(bear);
        reject(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::Black,
            },
        );
        checked(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::White,
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 0, 4],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(bear).tapped);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}
#[test]
fn layered_characteristics_nonland_targeting_and_planeswalker_combat_use_current_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let walker = fixture(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 4;
        aura(
            &mut g,
            p,
            "Imprisoned in the Moon",
            walker,
            [0, 1, 0, 0, 0, 2],
        );
        let shock = fixture(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: shock });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(walker),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert_eq!(g.state.objects[walker as usize].counters.loyalty, 4);
        let land = fixture(&mut g, p, "Soulstone Sanctuary", Zone::Battlefield);
        activate(&mut g, p, land);
        settle(&mut g);
        aura(&mut g, p, "Witness Protection", land, [0, 1, 0, 0, 0, 0]);
        let bounce = fixture(&mut g, p, "Into the Roil", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        checked(&mut g, Action::AnnounceCast { card: bounce });
        checked(&mut g, Action::ChooseKicker { pay: false });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(walker),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Hand);
        let attacker = fixture(&mut g, p, "Goblin Smuggler", Zone::Battlefield);
        until(&mut g, |g| matches!(g.state.pending, Pending::Attackers));
        let defender = g.object_ref(walker);
        reject(
            &mut g,
            Action::AttackPlaneswalker {
                card: attacker,
                defender,
            },
        );
        checked(&mut g, Action::Attack { card: attacker });
        checked(&mut g, Action::FinishAttackers);
        assert!(g.state.attackers.iter().any(|a| a.card == attacker));
        // A noncast Aura entry can happen during combat. Type loss removes the
        // recipient from combat without moving it off the battlefield.
        let moon = fixture(&mut g, p.other(), "Imprisoned in the Moon", Zone::Hand);
        let mut movement = g.entry_move(moon, Zone::Battlefield, false);
        movement.attachment = Some(g.object_ref(attacker));
        let before = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.stage_entries(vec![movement], None);
        child.state_based_actions();
        assert_eq!(child.object_view(attacker).zone, Zone::Battlefield);
        assert_eq!(child.current_kinds(attacker), 2);
        assert!(!child.state.attackers.iter().any(|a| a.card == attacker));
        assert_eq!(g.snapshot(), before);
        child.restore(&before).unwrap();
        assert!(child.state.attackers.iter().any(|a| a.card == attacker));
    }
}

#[test]
fn layered_characteristics_cleanup_expiry_detaches_equipment_and_resolves_role_death_this_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = fixture(&mut g, p, "Crawling Barrens", Zone::Battlefield);
        activate(&mut g, p, land);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        let role = Card::catalogue()
            .find(|c| c.role() == Some(RoleKind::Wicked))
            .unwrap();
        let recipient = g.object_ref(land);
        assert!(g.is_creature(land));
        g.create_attached_role_groups(
            p,
            &[TokenGroupView {
                token: role,
                amount: 1,
                tapped: false,
            }],
            recipient,
        );
        let role = g
            .state
            .roles
            .as_ref()
            .unwrap()
            .attached
            .last()
            .unwrap()
            .aura;
        let equipment = fixture(&mut g, p, "Short Bow", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let equip=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==equipment&&*t==land)).unwrap();
        checked(&mut g, equip);
        settle(&mut g);
        let turn = g.state.turn;
        let life = g.state.players[p.other().index()].life;
        for _ in 0..80 {
            if g.state.step == Step::Cleanup {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                    )
                })
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.state.step, Step::Cleanup);
        assert_eq!(g.state.turn, turn);
        assert_eq!(g.current_kinds(land), 2);
        assert!(!g.on_battlefield(role.card));
        assert!(g.object_view(equipment).attached_to.is_none());
        assert!(!g.state.stack.is_empty() || !g.state.pending_triggers.is_empty());
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 1);
        assert_eq!(g.state.turn, turn, "cleanup triggers precede the next turn");
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.turn, turn + 1);
    }
}
#[test]
fn layered_characteristics_restored_planeswalker_does_not_reconnect_old_attack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let walker = fixture(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 4;
        let attacker = fixture(&mut g, p, "Goblin Smuggler", Zone::Battlefield);
        let defender = g.object_ref(walker);
        until(&mut g, |g| matches!(g.state.pending, Pending::Attackers));
        checked(
            &mut g,
            Action::AttackPlaneswalker {
                card: attacker,
                defender,
            },
        );
        checked(&mut g, Action::FinishAttackers);
        let moon = fixture(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        let mut movement = g.entry_move(moon, Zone::Battlefield, false);
        movement.attachment = Some(defender);
        let before = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.stage_entries(vec![movement], None);
        child.state_based_actions();
        assert_eq!(child.state.attackers[0].defender, None);
        assert_eq!(child.state.attackers[0].card, attacker);
        let disenchant = fixture(&mut child, p, "Disenchant", Zone::Hand);
        child.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        cast(&mut child, disenchant, None, &[Target::Card(moon)]);
        settle(&mut child);
        assert_eq!(child.current_kinds(walker) & 64, 64);
        assert_eq!(child.object_ref(walker), defender);
        assert_eq!(child.state.attackers[0].defender, None);
        for _ in 0..60 {
            if child.state.step == Step::Main2 {
                break;
            }
            let action = child
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::FinishBlockers))
                .unwrap();
            checked(&mut child, action);
        }
        assert_eq!(child.state.objects[walker as usize].counters.loyalty, 4);
        assert_eq!(g.snapshot(), before);
        child.restore(&before).unwrap();
        assert_eq!(
            child.state.attackers[0].defender,
            Some(CombatDefender::Planeswalker(defender))
        );
    }
}

#[test]
fn layered_characteristics_cast_keyword_effect_continues_exact_incarnation_not_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None, &[]);
        let spell = g.object_ref(bear);
        g.grant_stack_keywords(spell, Keywords::HASTE);
        let before = g.object_view(bear);
        assert!(before.keywords.contains(Keywords::HASTE));
        let timestamp = g
            .state
            .roles
            .as_ref()
            .unwrap()
            .keyword_effects
            .last()
            .unwrap()
            .timestamp;
        settle(&mut g);
        assert_ne!(g.object_ref(bear), spell);
        assert!(g.keywords(bear).contains(Keywords::HASTE));
        assert_eq!(
            g.state
                .roles
                .as_ref()
                .unwrap()
                .keyword_effects
                .last()
                .unwrap()
                .timestamp,
            timestamp
        );
        let protection = aura(&mut g, p, "Witness Protection", bear, [0, 1, 0, 0, 0, 0]);
        assert!(!g.keywords(bear).contains(Keywords::HASTE));
        g.sacrifice_batch(&[(p, protection)]);
        g.state_based_actions();
        assert!(g.keywords(bear).contains(Keywords::HASTE));
        g.move_object(bear, Zone::Hand);
        assert!(!g.keywords(bear).contains(Keywords::HASTE));
        assert!(before.keywords.contains(Keywords::HASTE));
    }
}
