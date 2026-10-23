// Complete paid resolved-stat bodies; helpers above check every owned action.
fn resolved_foreign_source(g: &mut Game, p: Player, name: &str) -> CardId {
    let source = named(g, p.other(), name, Zone::Exile);
    g.state.exile_permissions.push(ExilePlayPermission {
        cast_only: false,
        without_mana: false,
        source: None,
        any_mana: false,
        object: g.object_ref(source),
        player: p,
        expires_turn: g.state.turn,
        next_end_step: None,
    });
    checked(g, Action::AnnounceExile { card: source });
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    settle(g);
    assert_eq!(g.object_view(source).owner, p.other());
    assert_eq!(g.object_view(source).controller, p);
    source
}
fn resolved_stack_pending(g: &mut Game, target: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..30 {
        if g.state.pending == Pending::Priority && g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let a = target
            .and_then(|t| {
                actions
                    .iter()
                    .find(|a| **a == Action::ChooseTarget { target: t })
            })
            .cloned()
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .cloned()
            })
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .cloned()
            })
            .unwrap();
        checked(g, a);
    }
    panic!("trigger announcement did not finish");
}
fn resolved_response(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    for _ in 0..100 {
        if g.object_view(id).zone != Zone::Stack
            && g.state.resolving.as_ref().is_none_or(|r| r.item.card != id)
        {
            return id;
        }
        let actions = g.legal_actions();
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap()
        };
        checked(g, a);
    }
    panic!("response did not resolve");
}
#[test]
fn resolved_genemorph_captures_resolution_land_count_and_keeps_fixed_values_after_source_departure()
{
    for p in [Player::P0, Player::P1] {
        for sacrifice in [false, true] {
            let mut g = setup(p);
            for _ in 0..8 {
                named(&mut g, p, "Shock", Zone::Library);
            }
            for _ in 0..5 {
                named(&mut g, p, "Forest", Zone::Battlefield);
            }
            // Opposing-owner lands must never supply the caster's threshold.
            for _ in 0..7 {
                named(&mut g, p.other(), "Forest", Zone::Battlefield);
            }
            let source = resolved_foreign_source(&mut g, p, "Genemorph Imago");
            assert!(g.keywords(source).contains(Keywords::FLYING));
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = named(&mut g, p, "Evolving Wilds", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: land });
            resolved_stack_pending(&mut g, Some(Target::Card(bear)));
            if sacrifice {
                if g.actor() != Some(p) {
                    checked(&mut g, Action::Pass);
                }
                activate(&mut g, land);
            }
            settle_on(&mut g, Some(Target::Card(bear)));
            let value = if sacrifice { 3 } else { 6 };
            assert_eq!(
                (g.stats(bear).power, g.stats(bear).toughness),
                (value, value)
            );
            resolved_response(&mut g, p, "Unsummon", &[Target::Card(source)]);
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Hand);
            assert_eq!(g.object_view(source).owner, p.other());
            assert_eq!(
                (g.stats(bear).power, g.stats(bear).toughness),
                (value, value)
            );
            resolved_response(&mut g, p, "Giant Growth", &[Target::Card(bear)]);
            settle(&mut g);
            assert_eq!(
                (g.stats(bear).power, g.stats(bear).toughness),
                (value + 3, value + 3)
            );
            cleanup_checked(&mut g);
            assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
        }
    }
}
#[test]
fn resolved_yearling_reads_current_signed_power_and_departure_lki_without_clamping() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            for accept in [false, true] {
                let mut g = setup(p);
                for _ in 0..6 {
                    named(&mut g, p, "Forest", Zone::Library);
                }
                let source = resolved_foreign_source(&mut g, p, "Belligerent Yearling");
                assert!(g.keywords(source).contains(Keywords::TRAMPLE));
                let dino = named(&mut g, p, "Barkform Harvester", Zone::Hand);
                // Stop after the paid permanent resolves, before its event ability resolves.
                cast(&mut g, dino, None, &[]);
                while g.object_view(dino).zone == Zone::Stack {
                    checked(&mut g, Action::Pass);
                }
                resolved_stack_pending(&mut g, None);
                let entered = g.object_ref(dino);
                assert!(
                    g.observe(p)
                        .resolved_stat_subjects
                        .iter()
                        .any(|x| x.object == entered)
                );
                resolved_response(&mut g, p, "Depower", &[Target::Card(dino)]);
                assert_eq!(g.stats(dino).power, -2);
                if leave {
                    resolved_response(&mut g, p, "Murder", &[Target::Card(dino)]);
                    assert_eq!(g.object_view(dino).zone, Zone::Graveyard);
                    assert_eq!(
                        g.observe(p)
                            .resolved_stat_subjects
                            .iter()
                            .find(|x| x.object == entered)
                            .unwrap()
                            .departure_power,
                        Some(-2)
                    );
                }
                settle_choices(&mut g, None, accept, false);
                assert_eq!(
                    (g.stats(source).power, g.stats(source).toughness),
                    (if accept { -2 } else { 3 }, 2)
                );
                cleanup_checked(&mut g);
                assert_eq!(g.stats(source).power, 3);
                assert!(g.observe(p).resolved_stat_subjects.is_empty());
            }
        }
    }
}
#[test]
fn resolved_moon_second_draw_and_once_per_turn_artifact_entry_are_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for q in [p, p.other()] {
            for _ in 0..16 {
                named(&mut g, q, "Forest", Zone::Library);
            }
        }
        let source = resolved_foreign_source(&mut g, p, "Moon Girl and Devil Dinosaur");
        for ordinal in 0..2 {
            let artifact = named(&mut g, p, "Three Tree Mascot", Zone::Hand);
            cast(&mut g, artifact, None, &[]);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                1,
                "artifact {ordinal}"
            );
            assert_eq!(g.stats(source).power, 2);
        }
        let draw = named(&mut g, p, "Quick Study", Zone::Hand);
        cast(&mut g, draw, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (6, 6));
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
        // Further draws must not refresh the timestamp or add duplicate setters.
        let effects = g.observe(p).base_stat_effects.clone();
        let draw = named(&mut g, p, "Quick Study", Zone::Hand);
        cast(&mut g, draw, None, &[]);
        settle(&mut g);
        assert_eq!(g.observe(p).base_stat_effects, effects);
        cleanup_checked(&mut g);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (2, 2));
        assert!(!g.keywords(source).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn resolved_yearling_subject_reentry_and_source_return_never_rebind_old_incarnations() {
    for p in [Player::P0, Player::P1] {
        for return_source in [false, true] {
            let mut g = setup(p);
            for _ in 0..8 {
                named(&mut g, p, "Forest", Zone::Library);
            }
            let source = resolved_foreign_source(&mut g, p, "Belligerent Yearling");
            let dino = named(&mut g, p, "Barkform Harvester", Zone::Hand);
            cast(&mut g, dino, None, &[]);
            while g.object_view(dino).zone == Zone::Stack {
                checked(&mut g, Action::Pass);
            }
            resolved_stack_pending(&mut g, None);
            let original_source = g.object_ref(source);
            let original_subject = g.object_ref(dino);
            resolved_response(&mut g, p, "Depower", &[Target::Card(dino)]);
            let returning = if return_source { source } else { dino };
            resolved_response(&mut g, p, "Fake Your Own Death", &[Target::Card(returning)]);
            resolved_response(&mut g, p, "Murder", &[Target::Card(returning)]);
            settle_choices(&mut g, None, true, false);
            assert_eq!(g.object_view(returning).zone, Zone::Battlefield);
            if return_source {
                assert_ne!(g.object_ref(source), original_source);
                assert_eq!(g.object_view(source).owner, p.other());
                assert_eq!(g.object_view(source).controller, p.other());
                assert_eq!(g.stats(source).power, 3);
                assert!(
                    !g.observe(p)
                        .base_stat_effects
                        .iter()
                        .any(|e| e.recipient == g.object_ref(source))
                );
            } else {
                assert_ne!(g.object_ref(dino), original_subject);
                // The new entry's trigger resolves first with current 2, followed by
                // the original packet's exact old departure power (-2 + Fake's 2).
                assert_eq!(g.stats(dino).power, 2);
                assert_eq!(g.stats(source).power, 0);
                assert_eq!(
                    g.observe(p)
                        .resolved_stat_subjects
                        .iter()
                        .find(|x| x.object == original_subject)
                        .unwrap()
                        .departure_power,
                    Some(0)
                );
            }
        }
    }
}

fn resolved_activate(g: &mut Game, card: CardId, ability: u16, target: Option<Target>) {
    let a=g.legal_actions().into_iter().find(|a| matches!(a,Action::AnnounceActivate {card:id,ability:n,..}if *id==card&&*n==ability)).unwrap();
    checked(g, a);
    while g.state.announcing.is_some() {
        let actions = g.legal_actions();
        let a=target.and_then(|t|actions.iter().find(|a|**a==Action::ChooseTarget {target:t}).cloned())
            .or_else(||target.and_then(|t|if let Target::Card(card)=t {actions.iter().find(|a|matches!(a,Action::ChooseAdditionalCost {object}if object.card==card)).cloned()}else {None}))
            .or_else(||actions.iter().find(|a|matches!(a,Action::FinishTargets|Action::PayCosts {..})).cloned()).unwrap();
        checked(g, a);
    }
}
#[test]
fn resolved_pupu_pays_both_abilities_and_snapshots_casters_towns_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..8 {
            named(&mut g, p, "Shock", Zone::Library);
        }
        let towns = [
            named(&mut g, p, "Capital City", Zone::Battlefield),
            named(&mut g, p, "Capital City", Zone::Battlefield),
        ];
        for _ in 0..5 {
            named(&mut g, p.other(), "Capital City", Zone::Battlefield);
        }
        let sacrificer = named(&mut g, p, "Seismic Monstrosaur", Zone::Hand);
        cast(&mut g, sacrificer, None, &[]);
        settle(&mut g);
        let source = resolved_foreign_source(&mut g, p, "PuPu UFO");
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate {card,ability:0,..}if *card==source))
        );
        g.state.objects[source as usize].summoning_sick = false;
        let town = named(&mut g, p, "Adventurer's Inn", Zone::Hand);
        let other = named(&mut g, p.other(), "Forest", Zone::Hand);
        let life = g.state.players[p.index()].life;
        resolved_activate(&mut g, source, 0, None);
        assert!(g.object_view(source).tapped);
        for _ in 0..100 {
            if g.state
                .resolving
                .as_ref()
                .and_then(|r| r.choice.as_ref())
                .is_some_and(|c| matches!(c.kind, EffectChoiceKind::HandLand { .. }))
            {
                break;
            }
            let a = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                g.legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
            };
            checked(&mut g, a);
        }
        assert_eq!(g.actor(), Some(p));
        assert!(g.observe(p.other()).effect_choice.is_none());
        let invalid = g.object_ref(other);
        reject(&mut g, Action::PutHandLand { object: invalid });
        let object = g.object_ref(town);
        checked(&mut g, Action::PutHandLand { object });
        settle(&mut g);
        assert_eq!(g.object_view(town).zone, Zone::Battlefield);
        assert!(!g.object_view(town).tapped);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        assert!(!g.state.land_played);
        let mana = g.state.players[p.index()]
            .mana
            .iter()
            .map(|n| *n as u64)
            .sum::<u64>();
        resolved_activate(&mut g, source, 1, None);
        assert_eq!(
            g.state.players[p.index()]
                .mana
                .iter()
                .map(|n| *n as u64)
                .sum::<u64>(),
            mana - 3
        );
        resolved_activate(&mut g, sacrificer, 0, Some(Target::Card(towns[0])));
        settle(&mut g);
        assert_eq!(g.object_view(towns[0]).zone, Zone::Graveyard);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (2, 4));
        let later = named(&mut g, p, "Capital City", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: later });
        settle(&mut g);
        assert_eq!(g.stats(source).power, 2);
        resolved_activate(&mut g, source, 1, None);
        settle(&mut g);
        assert_eq!(g.stats(source).power, 3);
        resolved_response(&mut g, p, "Giant Growth", &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (6, 7));
        cleanup_checked(&mut g);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (0, 4));
    }
}
#[test]
fn resolved_atomic_paid_equipment_attack_optional_target_and_departure_keep_layer_order() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = setup(p);
            for _ in 0..8 {
                named(&mut g, p, "Shock", Zone::Library);
            }
            let bearer = named(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, bearer, None, &[]);
            settle(&mut g);
            let source = resolved_foreign_source(&mut g, p, "Atomic Microsizer");
            resolved_activate(&mut g, source, 0, Some(Target::Card(bearer)));
            settle(&mut g);
            assert_eq!(
                g.object_view(source).attached_to,
                Some(g.object_ref(bearer))
            );
            assert_eq!(g.stats(bearer).power, 3);
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            resolved_response(&mut g, p, "Giant Growth", &[Target::Card(target)]);
            settle(&mut g);
            assert_eq!((g.stats(target).power, g.stats(target).toughness), (5, 5));
            g.state.objects[bearer as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            g.state.priority = p;
            checked(&mut g, Action::Attack { card: bearer });
            checked(&mut g, Action::FinishAttackers);
            resolved_stack_pending(&mut g, choose.then_some(Target::Card(target)));
            resolved_response(&mut g, p, "Disenchant", &[Target::Card(source)]);
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.object_view(source).owner, p.other());
            assert_eq!(g.stats(bearer).power, 2);
            assert_eq!(
                (g.stats(target).power, g.stats(target).toughness),
                if choose { (4, 4) } else { (5, 5) }
            );
            assert_eq!(
                g.keywords(target).contains(Keywords::CANT_BE_BLOCKED),
                choose
            );
            cleanup_checked(&mut g);
            assert_eq!((g.stats(target).power, g.stats(target).toughness), (2, 2));
            assert!(!g.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
        }
    }
}
