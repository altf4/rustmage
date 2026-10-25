//! Individual paid Super-Adaptoid interactions; typed gated staging is explicit until admission.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_thirteenth_combined_tests::owned_checked_step;
use super::*;
const ADAPT: &str = "Super-Adaptoid";
fn seed(g: &mut Game, p: Player, zone: Zone) -> CardId {
    let card = Card::from_name(ADAPT).unwrap();
    if card.is_supported() {
        return named(g, p, ADAPT, zone);
    }
    let id = named(g, p, "Hill Giant", zone);
    g.state.objects[id as usize].card = card;
    id
}
fn packet(g: &mut Game, p: Player) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = seed(g, p, Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..160 {
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
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled");
}
fn targets(g: &mut Game, source: CardId) {
    until(g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
            && g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == source)
    });
}
fn finish(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
}
fn choose(g: &mut Game, source: CardId, target: CardId) {
    targets(g, source);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g);
}
pub(super) fn paid(g: &mut Game, p: Player, donor: Option<CardId>) -> CardId {
    let id = packet(g, p);
    if let Some(d) = donor {
        choose(g, id, d);
    }
    settle(g);
    id
}
fn other_paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    super::msh_broker_thrall_tests::paid(g, p, name, targets)
}
fn other_packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    super::msh_broker_thrall_tests::packet(g, p, name, targets)
}
fn reject(g: &mut Game, a: Action) {
    let snap = g.snapshot();
    let bytes = g.state_bytes();
    let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), snap);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        views
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let p = g.actor().unwrap();
    let i = g.legal_actions().iter().position(|v| *v == a).unwrap();
    owned_checked_step(g, p, i, a)
}
#[test]
fn msh_adaptoid_paid_real_lands_mandatory_other_creature_and_no_target_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = seed(&mut g, p, Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: id });
        let a = named(&mut g, p, "Forest", Zone::Battlefield);
        let b = named(&mut g, p, "Forest", Zone::Battlefield);
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(&mut g, Action::TapMana { card: a });
        checked(&mut g, Action::TapMana { card: b });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 1,
                toughness: 2
            }
        );
        assert!(g.object_view(a).tapped && g.object_view(b).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(id).counters, Counters::default());
    }
}
#[test]
fn msh_adaptoid_cda_counts_current_controlled_legendary_creatures_in_every_zone() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
        named(
            &mut g,
            p.other(),
            "Winter Soldier, Icy Assassin",
            Zone::Battlefield,
        );
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p, "The Ten Rings", Zone::Battlefield);
        let hand = seed(&mut g, p, Zone::Hand);
        let grave = seed(&mut g, p, Zone::Graveyard);
        let exile = seed(&mut g, p, Zone::Exile);
        let library = seed(&mut g, p, Zone::Library);
        for id in [hand, grave, exile, library] {
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 1,
                    toughness: 2
                }
            );
        }
        let id = paid(&mut g, p, Some(own));
        assert_eq!(
            g.stats(id),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        g.set_counters(id, Counters::of(CounterKind::PlusOne, 3));
        assert_eq!(
            g.stats(id),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        other_paid(
            &mut g,
            p,
            "Harmless Offering",
            &[Target::Player(p.other()), Target::Card(own)],
        );
        assert_eq!(g.stats(id).power, 4);
        for other in [hand, grave, exile, library] {
            assert_eq!(g.stats(other).power, 1);
        }
    }
}
#[test]
fn msh_adaptoid_all_eleven_current_target_keywords_get_one_counter_each() {
    for p in [Player::P0, Player::P1] {
        for kind in CounterKind::KEYWORDS {
            let mut g = setup(p);
            let donor = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.set_counters(donor, Counters::of(kind, 3));
            let id = paid(&mut g, p, Some(donor));
            for other in CounterKind::KEYWORDS {
                assert_eq!(g.counter_count(id, other), u64::from(kind == other));
            }
            assert!(g.keywords(id).contains(kind.keyword().unwrap()));
            assert_eq!(g.counter_count(donor, kind), 3);
        }
    }
}
#[test]
fn msh_adaptoid_current_source_keywords_prevent_duplicate_and_batch_uses_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        other_paid(&mut g, p, "Doubling Season", &[]);
        other_paid(&mut g, p, "Doubling Season", &[]);
        let donor = other_paid(&mut g, p, "Serra Angel", &[]);
        let id = packet(&mut g, p);
        targets(&mut g, id);
        g.set_counters(id, Counters::of(CounterKind::Flying, 2));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        reject(&mut g, Action::FinishTargets);
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(donor),
            },
        );
        finish(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        g = orphan(g, action);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Flying), 2);
        assert_eq!(g.counter_count(id, CounterKind::Vigilance), 4);
    }
}
#[test]
fn msh_adaptoid_real_entry_haste_then_declared_attack_gains_real_donor_keywords() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let donor = other_paid(&mut g, p, "Volcanic Villain", &[]);
        let angel = other_paid(&mut g, p, "Serra Angel", &[]);
        let id = paid(&mut g, p, Some(donor));
        assert_eq!(g.counter_count(id, CounterKind::Haste), 1);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        g = orphan(g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        choose(&mut g, id, angel);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Flying), 1);
        assert_eq!(g.counter_count(id, CounterKind::Vigilance), 1);
        next_turn(&mut g);
        assert!(g.keywords(id).contains(Keywords::HASTE));
        assert_eq!(g.counter_count(id, CounterKind::Vigilance), 1);
    }
}
#[test]
fn msh_adaptoid_target_must_still_be_current_creature_and_source_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = setup(p);
            let angel = other_paid(&mut g, p, "Serra Angel", &[]);
            let id = packet(&mut g, p);
            choose(&mut g, id, angel);
            other_packet(
                &mut g,
                p.other(),
                if source_leaves { "Murder" } else { "Unsummon" },
                &[Target::Card(if source_leaves { id } else { angel })],
            );
            settle(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::Flying), 0);
            assert_eq!(g.counter_count(id, CounterKind::Vigilance), 0);
            assert_eq!(
                g.object_view(if source_leaves { id } else { angel }).zone,
                if source_leaves {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                }
            );
        }
    }
}
#[test]
fn msh_adaptoid_live_donor_keywords_rechecked_after_queued_ability_and_hexproof_fizzles() {
    for p in [Player::P0, Player::P1] {
        for variant in 0..3 {
            let mut g = setup(p);
            other_paid(&mut g, p, "High Fae Trickster", &[]);
            let angel = if variant == 1 {
                named(&mut g, p.other(), "Serra Angel", Zone::Battlefield)
            } else {
                other_paid(&mut g, p, "Serra Angel", &[])
            };
            let id = packet(&mut g, p);
            choose(&mut g, id, angel);
            other_packet(
                &mut g,
                if variant == 1 { p.other() } else { p },
                if variant == 0 {
                    "Witness Protection"
                } else {
                    "Shore Up"
                },
                &[Target::Card(angel)],
            );
            settle(&mut g);
            // Your target's own hexproof does not prohibit your ability.
            assert_eq!(
                g.counter_count(id, CounterKind::Flying),
                u64::from(variant == 2)
            );
        }
    }
}

#[test]
fn msh_adaptoid_ability_removal_after_trigger_preserves_packet_and_new_counter_timestamp() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        other_paid(&mut g, p, "High Fae Trickster", &[]);
        let angel = other_paid(&mut g, p, "Serra Angel", &[]);
        let id = packet(&mut g, p);
        choose(&mut g, id, angel);
        other_packet(&mut g, p, "Witness Protection", &[Target::Card(id)]);
        settle(&mut g);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(g.counter_count(id, CounterKind::Flying), 1);
        assert_eq!(g.counter_count(id, CounterKind::Vigilance), 1);
        assert!(g.keywords(id).contains(Keywords::FLYING));
        other_paid(&mut g, p, "Murder", &[Target::Card(angel)]);
        assert_eq!(g.stats(id).power, 1);
    }
}
#[test]
fn msh_adaptoid_real_taskmaster_copy_keeps_live_cda_and_intrinsic_legendary_override() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let donor = other_paid(&mut g, p, "Thanos, the Mad Titan", &[]);
        let id = paid(&mut g, p, Some(donor));
        let copy = other_paid(&mut g, p, "Taskmaster, Mercenary Mimic", &[]);
        next_turn(&mut g);
        next_turn(&mut g);
        targets(&mut g, copy);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.stats(id).power, 3);
        assert_eq!(
            g.stats(copy),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert_eq!(g.counter_count(copy, CounterKind::Lifelink), 0);
        assert!(!g.keywords(copy).contains(Keywords::LIFELINK));
    }
}
