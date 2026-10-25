//! Paid printed-card scenarios using ordinary supported card identities.
use super::assurance::big_helpers::{checked, named, setup};
use super::msh_galactus_tests::{finish, reject, until};
use super::*;
const WASP: &str = "The Wondrous Wasp";
const SPIDER: &str = "Spider-Woman, Secret Agent";
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn body(g: &mut Game, p: Player, name: &str) -> CardId {
    named(g, p, name, Zone::Hand)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
}
fn settle(g: &mut Game) {
    until(g, quiet);
}
fn spell(g: &mut Game, p: Player, name: &str, mana: Mana, targets: &[CardId]) -> CardId {
    priority(g, p);
    let id = body(g, p, name);
    g.state.players[p.index()].mana = mana;
    checked(g, Action::AnnounceCast { card: id });
    if let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
    {
        checked(g, a);
    }
    for &target in targets {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn trigger(g: &mut Game, id: CardId) {
    until(g, |g| {
        g.state.announcing.as_ref().is_some_and(|a| {
            a.item.card == id && matches!(a.item.kind, StackKind::TriggeredAbility { .. })
        })
    });
}
fn sting(g: &mut Game, id: CardId, target: Option<CardId>) {
    trigger(g, id);
    if let Some(card) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    if g.state.announcing.is_some() {
        finish(g);
    }
    settle(g);
}
fn cast_sting(g: &mut Game, p: Player, name: &str, t: CardId) -> CardId {
    let mana = if name == WASP {
        [0, 1, 0, 0, 0, 1]
    } else {
        [0, 1, 0, 0, 0, 3]
    };
    let id = spell(g, p, name, mana, &[]);
    sting(g, id, Some(t));
    id
}
fn other_spell(g: &mut Game, p: Player, name: &str, mana: Mana, targets: &[CardId]) -> CardId {
    let id = spell(g, p, name, mana, targets);
    settle(g);
    id
}
fn rows(g: &Game) -> usize {
    g.source_linked_effect_views().map_or(0, |r| r.len())
}
#[test]
fn msh_sting_wasp_paid_flash_optional_body_and_flying() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.priority = p;
        let id = spell(&mut g, p, WASP, [0, 1, 0, 0, 0, 1], &[]);
        sting(&mut g, id, None);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 2,
                toughness: 1
            }
        );
        assert!(
            g.keywords(id)
                .contains(Keywords::FLASH.union(Keywords::FLYING))
        );
        assert_eq!(rows(&g), 0);
        // Bounded combat boundary: an actual legal attack/block checks flying.
        let ground = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.active = p;
        g.state.priority = p;
        g.state.objects[id as usize].summoning_sick = false;
        g.begin_step(Step::BeginCombat);
        settle(&mut g);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: id,
            },
        );
    }
}
#[test]
fn msh_sting_wasp_paid_both_relations_remove_then_later_grant() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            let mut g = setup(p);
            let t = named(&mut g, owner, "Serra Angel", Zone::Battlefield);
            let s = cast_sting(&mut g, p, WASP, t);
            assert!(g.object_view(t).tapped);
            assert_eq!(g.keywords(t), Keywords::NONE);
            assert!(!g.printed_abilities_available(t));
            assert_eq!(rows(&g), 1);
            // The paid spell creates a later layer-six grant that survives older removal.
            other_spell(&mut g, p, "Take Up the Shield", [1, 0, 0, 0, 0, 1], &[t]);
            assert!(g.keywords(t).contains(Keywords::INDESTRUCTIBLE));
            assert!(!g.keywords(t).contains(Keywords::VIGILANCE));
            other_spell(&mut g, p, "Witness Protection", [0, 1, 0, 0, 0, 0], &[s]);
            assert!(!g.printed_abilities_available(s));
            assert!(!g.printed_abilities_available(t));
            other_spell(&mut g, p, "Unsummon", [0, 1, 0, 0, 0, 0], &[s]);
            assert!(g.printed_abilities_available(t));
            assert!(g.keywords(t).contains(Keywords::VIGILANCE));
            assert_eq!(rows(&g), 0);
        }
    }
}
#[test]
fn msh_sting_wasp_source_absent_or_blinked_before_resolution_taps_only() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let s = spell(&mut g, p, WASP, [0, 1, 0, 0, 0, 1], &[]);
            trigger(&mut g, s);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(t),
                },
            );
            finish(&mut g);
            // Paid response, leaving the original entry trigger on the stack.
            priority(&mut g, p);
            let u = named(&mut g, p, "Unsummon", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            super::assurance::big_helpers::cast(&mut g, u, None, &[Target::Card(s)]);
            until(&mut g, |g| g.object_view(s).zone == Zone::Hand);
            if blink {
                g.move_object(s, Zone::Battlefield);
            } // Explicit new-incarnation fixture.
            settle(&mut g);
            assert!(g.object_view(t).tapped);
            assert!(g.printed_abilities_available(t));
            assert_eq!(rows(&g), 0);
        }
    }
}
#[test]
fn msh_sting_wasp_target_incarnation_and_multiple_effect_timestamps() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let s = cast_sting(&mut g, p, WASP, t);
        g.grant_keyword_effects(&[t], g.object_ref(t), Keywords::HASTE); // Bounded later resolution grant.
        assert!(g.keywords(t).contains(Keywords::HASTE));
        let s2 = cast_sting(&mut g, p.other(), WASP, t);
        assert_eq!(rows(&g), 2);
        assert!(!g.keywords(t).contains(Keywords::HASTE));
        other_spell(&mut g, p, "Unsummon", [0, 1, 0, 0, 0, 0], &[s2]);
        assert!(g.keywords(t).contains(Keywords::HASTE));
        assert!(!g.printed_abilities_available(t));
        other_spell(&mut g, p, "Unsummon", [0, 1, 0, 0, 0, 0], &[t]);
        assert_eq!(rows(&g), 0);
        g.move_object(t, Zone::Battlefield);
        assert!(g.printed_abilities_available(t));
        assert_eq!(g.object_view(s).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_sting_spider_paid_hybrid_flash_target_legality() {
    for p in [Player::P0, Player::P1] {
        for color in [0, 1] {
            let mut g = setup(p);
            g.state.active = p.other();
            g.state.priority = p;
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let land = named(&mut g, p.other(), "Island", Zone::Battlefield);
            let mut mana = [0, 0, 0, 0, 0, 3];
            mana[color] = 1;
            let s = spell(&mut g, p, SPIDER, mana, &[]);
            trigger(&mut g, s);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            reject(&mut g, Action::FinishTargets);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert_eq!(
                g.stats(s),
                Stats {
                    power: 1,
                    toughness: 4
                }
            );
            assert!(g.keywords(s).contains(Keywords::FLASH));
            assert!(g.object_view(enemy).tapped);
            assert_eq!(rows(&g), 1);
            assert!(g.printed_abilities_available(enemy));
        }
    }
}
#[test]
fn msh_sting_spider_every_untap_cause_and_stun_remains() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        cast_sting(&mut g, p, SPIDER, t);
        g.set_counters(t, Counters::of(CounterKind::Stun, 1)); // Bounded preexisting counter.
        other_spell(&mut g, p.other(), "Shore Up", [0, 1, 0, 0, 0, 0], &[t]);
        assert!(g.object_view(t).tapped);
        assert_eq!(g.counter_count(t, CounterKind::Stun), 1);
        g.state.active = p.other();
        g.untap_active_player(); // Exact untap-step primitive.
        assert!(g.object_view(t).tapped);
        assert_eq!(g.counter_count(t, CounterKind::Stun), 1);
        assert!(!g.untap_allowed(t, untap_rules::UntapCause::Effect));
    }
}
#[test]
fn msh_sting_spider_control_duration_irreversible_suppression_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let s = cast_sting(&mut g, p, SPIDER, t);
        other_spell(&mut g, p, "Witness Protection", [0, 1, 0, 0, 0, 0], &[s]);
        assert!(!g.printed_abilities_available(s));
        assert!(!g.untap_allowed(t, untap_rules::UntapCause::Effect));
        // Paid theft of recipient does not end an external untap prohibition.
        other_spell(
            &mut g,
            p,
            "Involuntary Employment",
            [0, 0, 0, 1, 0, 3],
            &[t],
        );
        assert!(g.object_view(t).tapped);
        assert_eq!(g.object_view(t).controller, p);
        // Bounded next-main timing for the opponent's paid sorcery.
        g.state.active = p.other();
        g.state.step = Step::Main1;
        other_spell(
            &mut g,
            p.other(),
            "Involuntary Employment",
            [0, 0, 0, 1, 0, 3],
            &[s],
        );
        assert_eq!(rows(&g), 0);
        g.record_control(g.object_ref(s), p, None); // Bounded return of exact original source control.
        g.untap_objects(&[t], untap_rules::UntapCause::Effect);
        assert!(!g.object_view(t).tapped);
        assert_eq!(rows(&g), 0);
    }
}
#[test]
fn msh_sting_wasp_actual_and_funding_mana_restore_after_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let s = cast_sting(&mut g, p, WASP, elf);
        // Tapping and ability removal independently block production; untap only
        // changes the first obstacle, so both actual and overlay must still fail.
        g.untap_objects(&[elf], untap_rules::UntapCause::Effect);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let snap = g.snapshot();
        assert!(!g.can_fund(goal.clone()));
        assert_eq!(g.snapshot(), snap);
        let a = Action::TapMana { card: elf };
        reject(&mut g, a.clone());
        other_spell(&mut g, p, "Unsummon", [0, 1, 0, 0, 0, 0], &[s]);
        priority(&mut g, p);
        assert!(g.can_fund(goal));
        checked(&mut g, a);
        assert_eq!(g.state.players[p.index()].mana[4], 1);
        let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
        super::assurance::big_helpers::cast(&mut g, growth, None, &[Target::Card(elf)]);
        settle(&mut g);
        assert_eq!(g.stats(elf).power, 4);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_sting_public_duration_privacy_orphan_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let s = cast_sting(&mut g, p, WASP, t);
        for viewer in [p, p.other()] {
            let view = serde_json::to_value(g.observe(viewer)).unwrap();
            assert_eq!(view["source_linked_effects"].as_array().unwrap().len(), 1);
            let mut other = g.clone();
            named(&mut other, viewer.other(), "Mountain", Zone::Hand);
            let mut baseline = g.clone();
            named(&mut baseline, viewer.other(), "Forest", Zone::Hand);
            assert_eq!(
                serde_json::to_value(other.observe(viewer)).unwrap(),
                serde_json::to_value(baseline.observe(viewer)).unwrap()
            );
        }
        priority(&mut g, p);
        let u = named(&mut g, p, "Unsummon", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        super::assurance::big_helpers::cast(&mut g, u, None, &[Target::Card(s)]);
        let a = Action::Pass;
        let actor = g.actor().unwrap();
        let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
        g = super::msh_thirteenth_combined_tests::owned_checked_step(g, actor, index, a);
        settle(&mut g);
        assert_eq!(rows(&g), 0);
        assert!(g.printed_abilities_available(t));
        assert!(
            serde_json::to_value(g.observe(p))
                .unwrap()
                .get("source_linked_effects")
                .is_none()
        );
    }
}

#[test]
fn msh_sting_wasp_self_target_and_control_change_preserve_resolved_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = spell(&mut g, p, WASP, [0, 1, 0, 0, 0, 1], &[]);
        sting(&mut g, s, Some(s));
        assert!(g.object_view(s).tapped);
        assert_eq!(g.keywords(s), Keywords::NONE);
        assert_eq!(rows(&g), 1);
        g.state.active = p.other();
        g.state.step = Step::Main1;
        other_spell(
            &mut g,
            p.other(),
            "Involuntary Employment",
            [0, 0, 0, 1, 0, 3],
            &[s],
        );
        assert_eq!(g.object_view(s).controller, p.other());
        assert_eq!(rows(&g), 1);
        assert!(!g.printed_abilities_available(s));
        assert!(g.keywords(s).contains(Keywords::HASTE)); // Later resolved grant survives.
    }
}
#[test]
fn msh_sting_both_invalid_target_and_spider_source_departure_before_resolution() {
    for p in [Player::P0, Player::P1] {
        for name in [WASP, SPIDER] {
            let mut g = setup(p);
            let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let mana = if name == WASP {
                [0, 1, 0, 0, 0, 1]
            } else {
                [0, 1, 0, 0, 0, 3]
            };
            let s = spell(&mut g, p, name, mana, &[]);
            trigger(&mut g, s);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(t),
                },
            );
            finish(&mut g);
            other_spell(
                &mut g,
                p.other(),
                "Snakeskin Veil",
                [0, 0, 0, 0, 1, 0],
                &[t],
            );
            assert!(!g.object_view(t).tapped);
            assert_eq!(rows(&g), 0);
            assert!(g.printed_abilities_available(t));
            if name == SPIDER {
                let mut g = setup(p);
                let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
                let s = spell(&mut g, p, SPIDER, [0, 1, 0, 0, 0, 3], &[]);
                trigger(&mut g, s);
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(t),
                    },
                );
                finish(&mut g);
                other_spell(&mut g, p, "Unsummon", [0, 1, 0, 0, 0, 0], &[s]);
                assert!(g.object_view(t).tapped);
                assert_eq!(rows(&g), 0);
                assert!(g.untap_allowed(t, untap_rules::UntapCause::Effect));
            }
        }
    }
}
#[test]
fn msh_sting_wasp_prospective_paid_sacrifice_restores_funding_and_actual_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let s = cast_sting(&mut g, p, WASP, elf);
        g.untap_objects(&[elf], untap_rules::UntapCause::Effect);
        let ring = other_spell(&mut g, p, "Lotus Ring", [0, 0, 0, 0, 0, 3], &[]);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ring,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(s),
            },
        );
        finish(&mut g);
        settle(&mut g);
        g.state.objects[s as usize].summoning_sick = false; // Bounded age boundary for granted tap cost.
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        let goal = |n| FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, n, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let snap = g.snapshot();
        assert!(g.can_fund(goal(4)));
        assert!(!g.can_fund(goal(5)));
        assert_eq!(g.snapshot(), snap);
        let grantor = g.object_ref(ring);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: s,
                grantor,
                production: [0, 0, 0, 0, 3, 0],
            },
        );
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        assert_eq!(rows(&g), 0);
        assert!(g.printed_abilities_available(elf));
        checked(&mut g, Action::TapMana { card: elf });
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 4, 0]);
        let guard = named(&mut g, p, "Wildwood Scourge", Zone::Hand);
        // Spend the actual green production on a real X creature, not a fabricated goal.
        checked(&mut g, Action::AnnounceCast { card: guard });
        checked(&mut g, Action::ChooseX { value: 3 });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(guard, CounterKind::PlusOne), 3);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn msh_sting_spider_paid_finale_cannot_choose_prohibited_animated_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p.other(), "Island", Zone::Battlefield);
        // Bounded earlier-layer fixture makes an ordinary land a legal creature
        // target. The actual Spider and Finale spells below are paid normally.
        let reference = g.object_ref(t);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .as_mut()
            .unwrap()
            .animations
            .push(AnimationView {
                object: reference,
                timestamp,
                artifact: false,
                preserve_card_types: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                expires_turn: None,
            });
        cast_sting(&mut g, p, SPIDER, t);
        g.set_counters(t, Counters::of(CounterKind::Stun, 1));
        let healthy = named(&mut g, p.other(), "Island", Zone::Battlefield);
        g.state.objects[healthy as usize].tapped = true;
        g.state.active = p.other();
        g.state.step = Step::Main1;
        priority(&mut g, p.other());
        let f = named(&mut g, p.other(), "Finale of Revelation", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 2, 0, 0, 0, 10];
        checked(&mut g, Action::AnnounceCast { card: f });
        checked(&mut g, Action::ChooseX { value: 10 });
        finish(&mut g);
        until(&mut g, |g| {
            g.state.resolving.as_ref().is_some_and(|r| {
                r.choice
                    .as_ref()
                    .is_some_and(|c| matches!(c.kind, EffectChoiceKind::UntapLands { .. }))
            })
        });
        reject(&mut g, Action::SelectLand { object: reference });
        let object = g.object_ref(healthy);
        checked(&mut g, Action::SelectLand { object });
        checked(&mut g, Action::FinishLandSelection);
        settle(&mut g);
        assert!(g.object_view(t).tapped);
        assert_eq!(g.counter_count(t, CounterKind::Stun), 1);
        assert!(!g.object_view(healthy).tapped);
    }
}
