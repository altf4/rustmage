//! Focused printed outcomes; shared engine invariants live in core tests.
//! Whole-card Secrets of Strixhaven white/blue outcomes, through paid actions.
use super::*;

fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
        .map(|(i, _)| i as CardId)
        .collect()
}

#[test]
fn sos_harsh_annotation_uses_controller_lki_even_when_destroy_fails() {
    {
        let p = Player::P0;
        for indestructible in [false, true] {
            let mut g = scenario(p);
            let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[victim as usize].controller = p.other();
            if indestructible {
                g.state.objects[victim as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            let spell = put(&mut g, p, "Harsh Annotation", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(victim)));
            settle(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if indestructible {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            let made = tokens(&g, p.other());
            assert_eq!(made.len(), 1);
            let token = made[0];
            assert_eq!(stats(&g, token), (1, 1));
            assert!(g.keywords(token).contains(Keywords::FLYING));
            assert_eq!(g.object_view(token).colors, 1 | 4);
            assert!(g.has_subtype(token, "Inkling"));
            assert!(tokens(&g, p).is_empty());
        }
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Harsh Annotation", Zone::Hand);
        // Representative paid announcement checks raw/index ownership and restore.
        big_helpers::cast(&mut g, spell, None, &[Target::Card(victim)]);
        let old = g.object_ref(victim);
        g.move_object(victim, Zone::Hand);
        g.move_object(victim, Zone::Battlefield);
        assert_ne!(g.object_ref(victim), old);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
        assert!(tokens(&g, p.other()).is_empty());
    }
}

fn homesickness(g: &mut Game, spell: CardId, player: Player, creatures: &[CardId]) {
    act(g, Action::AnnounceCast { card: spell });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    for &creature in creatures {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn sos_homesickness_zero_two_and_partial_targets() {
    {
        let p = Player::P0;
        for count in 0..=2 {
            let mut g = scenario(p);
            let creatures: Vec<_> = (0..count)
                .map(|_| put(&mut g, p.other(), "Bear Cub", Zone::Battlefield))
                .collect();
            let spell = put(&mut g, p, "Homesickness", Zone::Hand);
            let hand = g.hand_count(p.other());
            homesickness(&mut g, spell, p.other(), &creatures);
            if count == 2 {
                g.move_object(creatures[0], Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p.other()), hand + 2 + usize::from(count == 2));
            for (index, &creature) in creatures.iter().enumerate() {
                if count == 2 && index == 0 {
                    continue;
                }
                assert!(g.object_view(creature).tapped);
                assert_eq!(g.counter_count(creature, CounterKind::Stun), 1);
            }
        }
    }
}

#[test]
fn sos_wisdom_returns_only_own_spells_exiles_and_persists_hand_limit() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let instant = put(&mut g, p, "Unsummon", Zone::Graveyard);
        let sorcery = put(&mut g, p, "Divination", Zone::Graveyard);
        let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let opposing = put(&mut g, p.other(), "Divination", Zone::Graveyard);
        let spell = put(&mut g, p, "Wisdom of Ages", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.object_view(instant).zone, Zone::Hand);
        assert_eq!(g.object_view(sorcery).zone, Zone::Hand);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(g.object_view(opposing).zone, Zone::Graveyard);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert!(g.state.players[p.index()].no_maximum_hand_size);
        assert!(!g.state.players[p.other().index()].no_maximum_hand_size);
        for _ in 0..10 {
            put(&mut g, p, "Forest", Zone::Hand);
        }
        g.finish_cleanup();
        assert_ne!(g.state.pending, Pending::Discard);
    }
}

#[test]
fn sos_preparation_survives_ability_type_and_controller_changes() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Landscape Painter", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        let copy = g.state.preparations.as_ref().unwrap()[0].spell_copy.card;
        let aura = put(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(source)));
        settle(&mut g);
        assert!(!g.is_creature(source));
        assert!(!g.printed_abilities_available(source));
        assert!(g.object_view(source).prepared);
        assert!(g.can_cast_prepared(p, copy));
        let source_owner = g.state.objects[source as usize].owner;
        g.record_control(g.object_ref(source), p.other(), None);
        assert_eq!(g.state.objects[source as usize].owner, source_owner);
        assert_eq!(g.state.objects[copy as usize].owner, p);
        assert!(!g.can_cast_prepared(p, copy));
        assert!(g.can_cast_prepared(p.other(), copy));
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [8; 6];
        let hand = g.hand_count(p.other());
        act(&mut g, Action::AnnounceExile { card: copy });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(p.other()), hand + 2);
        assert!(!g.object_view(source).prepared);
    }
}

fn prepared_cast(g: &mut Game, source: CardId, targets: &[Target]) -> CardId {
    let row = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.permanent == g.object_ref(source))
        .copied()
        .unwrap();
    let id = row.spell_copy.card;
    act(g, Action::AnnounceExile { card: id });
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    id
}

#[test]
fn sos_prepared_whole_card_bodies_and_inset_effects() {
    {
        let p = Player::P0;
        for (name, power, toughness, keyword, bonus_keyword) in [
            (
                "Honorbound Page",
                3,
                3,
                Keywords::FIRST_STRIKE,
                Keywords::FLYING,
            ),
            (
                "Quill-Blade Laureate",
                1,
                1,
                Keywords::DOUBLE_STRIKE,
                Keywords::DOUBLE_STRIKE,
            ),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None);
            settle(&mut g);
            assert_eq!(stats(&g, source), (power, toughness));
            assert!(g.keywords(source).contains(keyword));
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            prepared_cast(&mut g, source, &[Target::Card(target)]);
            settle(&mut g);
            assert_eq!(stats(&g, target), (3, 2));
            assert!(g.keywords(target).contains(bonus_keyword));
            g.finish_cleanup();
            assert_eq!(stats(&g, target), (2, 2));
            assert!(!g.keywords(target).contains(bonus_keyword));
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Campus Composer", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (3, 4));
        prepared_cast(&mut g, source, &[]);
        settle(&mut g);
        let made = tokens(&g, p);
        assert_eq!(made.len(), 1);
        assert_eq!(stats(&g, made[0]), (3, 3));
        assert_eq!(g.object_view(made[0]).colors, 2 | 8);
        assert!(g.keywords(made[0]).contains(Keywords::FLYING));
        let mut g = scenario(p);
        let source = put(&mut g, p, "Spellbook Seeker", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (3, 3));

        let draws = g.state.players[p.index()].cards_drawn_this_turn;
        prepared_cast(&mut g, source, &[]);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 2);
        assert_eq!(g.hand_count(p), 0);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 2);
    }
}

#[test]
fn sos_prepared_ward_counter_decline_and_attack_reprepare() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Campus Composer", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        g.state.priority = p.other();
        let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
        cast(&mut g, bounce, Some(Target::Card(source)));
        for _ in 0..16 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
        act(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bounce).zone, Zone::Graveyard);
        let mut g = scenario(p);
        let source = put(&mut g, p, "Encouraging Aviator", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (2, 3));

        assert!(!g.object_view(source).prepared);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert!(g.object_view(source).prepared);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        prepared_cast(&mut g, source, &[Target::Card(target)]);
        settle(&mut g);
        assert!(g.keywords(target).contains(Keywords::FLYING));
        assert!(!g.object_view(source).prepared);
    }
}

fn until_pending_choice(g: &mut Game) {
    for _ in 0..24 {
        if g.state.pending != Pending::Priority {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("missing resolution choice");
}

#[test]
fn sos_professor_increment_rechecks_live_stats_and_counter_batches_draw_once() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let professor = put(&mut g, p, "Pensive Professor", Zone::Hand);
        cast(&mut g, professor, None);
        settle(&mut g);
        assert_eq!(stats(&g, professor), (0, 2));
        let draw = put(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, draw, None);
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, Some(Target::Card(professor)));
        settle(&mut g);
        assert_eq!(g.counter_count(professor, CounterKind::PlusOne), 1);
        assert_eq!(stats(&g, professor), (4, 6));
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        g.finish_cleanup();
        let prior = g.state.players[p.index()].cards_drawn_this_turn;
        g.effect_counter(
            professor,
            CounterKind::PlusOne,
            3,
            replacements::ResumeMode::Pending,
        );
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, prior + 1);
    }
}

#[test]
fn sos_primary_research_limits_reanimation_and_end_step_history() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let valid = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        let spellcard = put(&mut g, p, "Divination", Zone::Graveyard);
        let costly = put(&mut g, p, "Serra Angel", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let source = put(&mut g, p, "Primary Research", Zone::Hand);
        cast(&mut g, source, None);
        until_pending_choice(&mut g);
        for target in [land, spellcard, costly, enemy] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(valid),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(valid).zone, Zone::Battlefield);
        let hand = g.hand_count(p);
        g.begin_step(Step::End);
        big_helpers::settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        g.finish_cleanup();
        let hand = g.hand_count(p);
        g.begin_step(Step::End);
        big_helpers::settle(&mut g);
        assert_eq!(g.hand_count(p), hand);
    }
}

#[test]
fn sos_ideation_reprepares_only_after_exact_optional_eight_card_payment() {
    {
        let p = Player::P0;
        for accept in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Emeritus of Ideation", Zone::Hand);
            cast(&mut g, source, None);
            settle(&mut g);
            assert_eq!(stats(&g, source), (5, 5));

            let drawn = g.state.players[p.other().index()].cards_drawn_this_turn;
            prepared_cast(&mut g, source, &[Target::Player(p.other())]);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].cards_drawn_this_turn,
                drawn + 3
            );
            assert!(!g.object_view(source).prepared);
            let grave: Vec<_> = (0..8)
                .map(|_| put(&mut g, p, "Forest", Zone::Graveyard))
                .collect();
            let enemy = put(&mut g, p.other(), "Forest", Zone::Graveyard);
            g.state.objects[source as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: source });
            act(&mut g, Action::FinishAttackers);
            until_pending_choice(&mut g);
            let enemy_object = g.object_ref(enemy);
            reject(
                &mut g,
                Action::Sacrifice {
                    object: enemy_object,
                },
            );
            if accept {
                for &id in &grave {
                    let object = g.object_ref(id);
                    act(&mut g, Action::Sacrifice { object });
                }
            } else {
                act(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(g.object_view(source).prepared, accept);
            for id in grave {
                assert_eq!(
                    g.object_view(id).zone,
                    if accept { Zone::Exile } else { Zone::Graveyard }
                );
            }
        }
    }
}

#[test]
fn sos_fractal_anomaly_uses_actual_draw_history_and_only_new_tokens_get_counters() {
    {
        let p = Player::P0;
        for draws in [0, 2] {
            let mut g = scenario(p);
            let old = put(&mut g, p, "@token/sos-fractal", Zone::Battlefield);
            g.set_counters(old, Counters::of(CounterKind::PlusOne, 1));
            if draws == 2 {
                let draw = put(&mut g, p, "Divination", Zone::Hand);
                cast(&mut g, draw, None);
                settle(&mut g);
            }
            let spell = put(&mut g, p, "Fractal Anomaly", Zone::Hand);
            cast(&mut g, spell, None);
            big_helpers::settle(&mut g);
            assert_eq!(g.counter_count(old, CounterKind::PlusOne), 1);
            let made: Vec<_> = tokens(&g, p).into_iter().filter(|id| *id != old).collect();
            assert_eq!(made.len(), usize::from(draws > 0));
            if draws > 0 {
                assert_eq!(g.counter_count(made[0], CounterKind::PlusOne), draws);
                assert_eq!(stats(&g, made[0]), (2, 2));
                assert_eq!(g.object_view(made[0]).colors, 2 | 16);
            }
        }
    }
}

#[test]
fn sos_echocasting_target_player_owns_plain_copiable_token_and_partial_target_still_paradigms() {
    {
        let p = Player::P0;
        for recipient in [p, p.other()] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, growth, Some(Target::Card(source)));
            settle(&mut g);
            assert_eq!(stats(&g, source), (5, 5));
            let spell = put(&mut g, p, "Echocasting Symposium", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(recipient),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(source),
                },
            );
            pay(&mut g);
            big_helpers::settle(&mut g);
            let copies = tokens(&g, recipient);
            assert_eq!(copies.len(), 1);
            assert_eq!(stats(&g, copies[0]), (2, 2));
            assert_eq!(g.object_view(copies[0]).owner, recipient);
            assert_eq!(g.object_view(copies[0]).name(), "Bear Cub");
            assert_eq!(g.observe(p).paradigms.len(), 1);
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Echocasting Symposium", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        pay(&mut g);
        g.move_object(source, Zone::Hand);
        big_helpers::settle(&mut g);
        assert!(tokens(&g, p).is_empty());
        assert_eq!(g.observe(p).paradigms.len(), 1);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}

#[test]
fn sos_jadzi_x_counts_all_own_fractals_and_two_x_symbols_are_paid() {
    {
        let p = Player::P0;
        for x in [0, 2] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Jadzi, Steward of Fate", Zone::Hand);
            cast(&mut g, source, None);
            big_helpers::settle(&mut g);
            assert_eq!(stats(&g, source), (2, 4));
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 2);
            let old = put(&mut g, p, "@token/sos-fractal", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "@token/sos-fractal", Zone::Battlefield);
            g.set_counters(old, Counters::of(CounterKind::PlusOne, 1));
            g.set_counters(enemy, Counters::of(CounterKind::PlusOne, 1));
            let copy = g
                .state
                .preparations
                .as_ref()
                .unwrap()
                .iter()
                .find(|r| r.permanent.card == source)
                .unwrap()
                .spell_copy
                .card;
            let mana: u32 = g.state.players[p.index()].mana.iter().sum();
            act(&mut g, Action::AnnounceExile { card: copy });
            act(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            assert_eq!(
                mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                1 + 2 * x
            );
            big_helpers::settle(&mut g);
            assert_eq!(g.counter_count(old, CounterKind::PlusOne), 1 + u64::from(x));
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 1);
            let new: Vec<_> = tokens(&g, p).into_iter().filter(|id| *id != old).collect();
            assert_eq!(new.len(), x as usize);
            for id in new {
                assert_eq!(g.counter_count(id, CounterKind::PlusOne), u64::from(x));
            }
            assert!(!g.object_view(source).prepared);
        }
    }
}

fn hydro_mana(g: &mut Game, source: CardId, ability: u16, production: Mana) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
        Action::ActivateMana { card, ability: i, production: p, .. } |
        Action::AnnounceMana { card, ability: i, production: p, .. }
            if *card == source && *i == ability && *p == production)
        })
        .unwrap();
    let announce = matches!(action, Action::AnnounceMana { .. });
    big_helpers::checked(g, action);
    if announce {
        pay(g);
    }
}

#[test]
fn sos_hydro_restricted_mana_cannot_pay_ward_during_resolution() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        g.state.players[p.index()].mana = [0; 6];
        for _ in 0..3 {
            let hydro = put(&mut g, p, "Hydro-Channeler", Zone::Battlefield);
            hydro_mana(&mut g, hydro, 0, [0, 1, 0, 0, 0, 0]);
        }
        let warded = put(&mut g, p.other(), "Campus Composer", Zone::Battlefield);
        let spell = put(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(warded)));
        until_pending_choice(&mut g);
        assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
        assert_eq!(g.state.players[p.index()].mana, [0, 2, 0, 0, 0, 0]);
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 2, 0, 0, 0, 0],
            },
        );
        act(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(warded).zone, Zone::Battlefield);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}

fn offense(
    g: &mut Game,
    source: CardId,
    player: Player,
    target: CardId,
    flashback: bool,
    double_strike: bool,
) {
    act(
        g,
        if flashback {
            Action::AnnounceFlashback { card: source }
        } else {
            Action::AnnounceCast { card: source }
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
    until_pending_choice(g);
    act(
        g,
        Action::ChooseOptional {
            accept: double_strike,
        },
    );
    settle(g);
}

#[test]
fn sos_practiced_offense_player_counters_resolution_keyword_and_flashback() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let ours = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let theirs = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Practiced Offense", Zone::Hand);
        offense(&mut g, spell, p.other(), ours, false, true);
        assert_eq!(g.counter_count(ours, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(theirs, CounterKind::PlusOne), 1);
        assert!(g.keywords(ours).contains(Keywords::DOUBLE_STRIKE));
        g.finish_cleanup();
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        g.state.players[p.index()].mana = [8; 6];
        assert!(!g.keywords(ours).contains(Keywords::DOUBLE_STRIKE));
        offense(&mut g, spell, p, theirs, true, false);
        assert_eq!(g.counter_count(ours, CounterKind::PlusOne), 1);
        assert!(g.keywords(theirs).contains(Keywords::LIFELINK));
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}

#[test]
fn sos_daydream_returns_owner_with_replaced_entry_counters_and_flashback() {
    {
        let p = Player::P0;
        {
            let mut g = scenario(p);
            let owner = { p };
            put(&mut g, owner, "Doubling Season", Zone::Battlefield);
            let target = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            g.state.objects[target as usize].controller = p;
            let spell = put(&mut g, p, "Daydream", Zone::Hand);
            let before = g.object_ref(target);
            big_helpers::cast(&mut g, spell, None, &[Target::Card(target)]);
            big_helpers::settle(&mut g);
            assert_ne!(g.object_ref(target), before);
            assert_eq!(g.object_view(target).controller, owner);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            {
                act(&mut g, Action::AnnounceFlashback { card: spell });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                pay(&mut g);
                big_helpers::settle(&mut g);
                assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
                assert_eq!(g.object_view(spell).zone, Zone::Exile);
            }
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Harsh Annotation", Zone::Hand);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, source, Some(Target::Card(victim)));
        settle(&mut g);
        let token = tokens(&g, p.other())[0];
        g.state.objects[token as usize].controller = p;
        let spell = put(&mut g, p, "Daydream", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(token)));
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
}

#[test]
fn sos_skycoach_flash_body_and_nonpilot_blink_are_whole() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        g.state.active = p.other();
        let source = put(&mut g, p, "Skycoach Conductor", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (2, 3));
        assert!(
            g.keywords(source).contains(
                Keywords::FLASH
                    .union(Keywords::FLYING)
                    .union(Keywords::VIGILANCE)
            )
        );
        if g.actor() != Some(p) {
            act(&mut g, Action::Pass);
        }
        let row = g.state.preparations.as_ref().unwrap()[0];
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[victim as usize].controller = p;
        act(
            &mut g,
            Action::AnnounceExile {
                card: row.spell_copy.card,
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        pay(&mut g);
        let before = g.object_ref(victim);
        big_helpers::settle(&mut g);
        assert_ne!(g.object_ref(victim), before);
        assert_eq!(g.object_view(victim).controller, p.other());
        assert_eq!(g.counter_count(victim, CounterKind::PlusOne), 0);
        assert!(!g.object_view(source).prepared);
    }
}

#[test]
fn sos_joined_hand_condition_checks_trigger_and_resolution_on_either_turn() {
    {
        let p = Player::P0;
        for active in [p, p.other()] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Joined Researchers", Zone::Battlefield);
            assert_eq!(stats(&g, source), (2, 2));

            g.state.active = active;
            g.begin_step(Step::End);
            assert!(g.state.pending_triggers.is_empty());
            let card = put(&mut g, p.other(), "Forest", Zone::Hand);
            g.begin_step(Step::End);
            assert_eq!(g.state.pending_triggers.len(), 1);
            g.move_object(card, Zone::Graveyard);
            big_helpers::settle(&mut g);
            assert!(!g.object_view(source).prepared);
            g.move_object(card, Zone::Hand);
            g.begin_step(Step::End);
            big_helpers::settle(&mut g);
            assert!(g.object_view(source).prepared);
            g.state.active = p;
            g.state.step = Step::Main1;
            g.state.priority = p;
            let own = g.hand_count(p);
            let other = g.hand_count(p.other());
            let copy = g.state.preparations.as_ref().unwrap()[0].spell_copy.card;
            act(&mut g, Action::AnnounceExile { card: copy });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.hand_count(p), own + 3);
            assert_eq!(g.hand_count(p.other()), other + 3);
        }
    }
}

#[test]
fn sos_matterbending_optional_other_bounce_and_frozen_triggering_x() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, p, "Matterbending Mage", Zone::Hand);
        cast(&mut g, source, None);
        until_pending_choice(&mut g);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Hand);
        for x in [0, 2] {
            let spell = put(&mut g, p, "Mind Spring", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            let packet = g
                .state
                .stack
                .iter()
                .find(|t| t.source.is_some_and(|identity| identity.id == source))
                .unwrap();
            assert_eq!(packet.x, Some(x));
            assert_eq!(g.item_amount(AmountValue::ChosenX, packet), u128::from(x));
            let id = g
                .state
                .stack
                .iter()
                .find(|t| t.source.is_some_and(|identity| identity.id == spell))
                .unwrap()
                .id;
            assert!(g.counter_stack_item(id, false));
            big_helpers::settle(&mut g);
            assert!(g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
            g.finish_cleanup();
            assert!(!g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
            g.state.active = p;
            g.state.priority = p;
            g.state.step = Step::Main1;
            g.state.pending = Pending::Priority;
            g.state.players[p.index()].mana = [8; 6];
        }
        let regular = put(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, regular, None);
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .any(|t| t.source.is_some_and(|identity| identity.id == source))
        );
        settle(&mut g);
    }
}

#[test]
fn sos_antiquities_hand_and_flashback_apply_counters_to_all_own_spirits() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Antiquities on the Loose", Zone::Hand);
        big_helpers::cast(&mut g, spell, None, &[]);
        big_helpers::settle(&mut g);
        let first = tokens(&g, p);
        assert_eq!(first.len(), 2);
        for id in &first {
            assert_eq!(stats(&g, *id), (2, 2));
            assert_eq!(g.object_view(*id).colors, 1 | 8);
        }
        // A separate opponent Spirit verifies controller filtering without affecting the cast.
        g.create_tokens(p.other(), Card::SosSpiritToken, 1, false);
        let enemy = tokens(&g, p.other())[0];
        act(&mut g, Action::AnnounceFlashback { card: spell });
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        let all = tokens(&g, p);
        assert_eq!(all.len(), 4);
        for id in all {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
    }
}

#[test]
fn sos_erode_freezes_controller_optional_search_and_failed_destruction() {
    {
        let p = Player::P0;
        for indestructible in [false, true] {
            for accept in [false, true] {
                let mut g = scenario(p);
                let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                g.state.objects[target as usize].controller = p.other();
                if indestructible {
                    g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
                }
                let basic = put(&mut g, p.other(), "Island", Zone::Library);
                let hidden = put(&mut g, p.other(), "Bear Cub", Zone::Library);
                let spell = put(&mut g, p, "Erode", Zone::Hand);
                big_helpers::cast(&mut g, spell, None, &[Target::Card(target)]);
                until_pending_choice(&mut g);
                assert_eq!(g.actor(), Some(p.other()));
                assert_eq!(
                    g.object_view(target).zone,
                    if indestructible {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
                big_helpers::checked(&mut g, Action::ChooseOptional { accept });
                if accept {
                    assert!(g.observe(p).search_choice.is_none());
                    assert!(
                        g.observe(p)
                            .objects
                            .iter()
                            .all(|o| o.id != basic && o.id != hidden)
                    );
                    reject(&mut g, Action::ChooseSearch { card: hidden });
                    big_helpers::checked(&mut g, Action::ChooseSearch { card: basic });
                    big_helpers::checked(&mut g, Action::FinishSearch);
                }
                big_helpers::settle(&mut g);
                assert_eq!(
                    g.object_view(basic).zone,
                    if accept {
                        Zone::Battlefield
                    } else {
                        Zone::Library
                    }
                );
                if accept {
                    assert!(g.object_view(basic).tapped);
                    assert_eq!(g.object_view(basic).controller, p.other());
                }
            }
        }
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Erode", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        g.move_object(target, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.pending, Pending::Priority);
    }
}

#[test]
fn sos_truce_token_recipient_changes_live_condition_and_swords_uses_power_controller() {
    {
        let p = Player::P0;
        for recipient in [p, p.other()] {
            let mut g = scenario(p);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = put(&mut g, p, "Emeritus of Truce", Zone::Hand);
            cast(&mut g, source, None);
            until_pending_choice(&mut g);
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(recipient),
                },
            );
            big_helpers::settle(&mut g);
            assert_eq!(stats(&g, source), (3, 3));
            let ink = tokens(&g, recipient);
            assert_eq!(ink.len(), 1);
            assert_eq!(stats(&g, ink[0]), (1, 1));
            assert_eq!(g.object_view(ink[0]).colors, 1 | 4);
            assert!(g.keywords(ink[0]).contains(Keywords::FLYING));
            assert_eq!(g.object_view(source).prepared, recipient == p.other());
            if recipient == p.other() {
                g.set_counters(enemy, Counters::of(CounterKind::PlusOne, 2));
                let life = g.state.players[p.other().index()].life;
                prepared_cast(&mut g, source, &[Target::Card(enemy)]);
                settle(&mut g);
                assert_eq!(g.object_view(enemy).zone, Zone::Exile);
                assert_eq!(g.state.players[p.other().index()].life, life + 4);
                assert!(!g.object_view(source).prepared);
            }
        }
    }
}

fn finish_direct_entries(g: &mut Game) {
    for _ in 0..8 {
        if g.state.replacements.is_some() {
            g.advance_replacements(true);
        }
        if g.state.entry_batch.is_some() {
            g.advance_entries();
        }
        if g.state.replacements.is_none() && g.state.entry_batch.is_none() {
            return;
        }
    }
    panic!("direct entry did not settle");
}

#[test]
fn sos_spiritcall_batches_noncreature_tokens_and_observes_its_own_copy_entry() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Spiritcall Enthusiast", Zone::Battlefield);
        g.create_tokens(p.other(), Card::SosSpiritToken, 2, false);
        finish_direct_entries(&mut g);
        assert!(g.state.pending_triggers.is_empty());
        g.create_tokens(p, Card::SosSpiritToken, 2, false);
        finish_direct_entries(&mut g);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|t| t.source.is_some_and(|identity| identity.id == source))
                .count(),
            1
        );
        g.create_tokens(
            p,
            Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap(),
            2,
            false,
        );
        finish_direct_entries(&mut g);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|t| t.source.is_some_and(|identity| identity.id == source))
                .count(),
            2
        );
        big_helpers::settle(&mut g);
        assert!(g.object_view(source).prepared);
        let mut g = scenario(p);
        put(&mut g, p, "Torpor Orb", Zone::Battlefield);
        let source = put(&mut g, p, "Spiritcall Enthusiast", Zone::Battlefield);
        g.create_tokens(p, Card::SosSpiritToken, 2, false);
        finish_direct_entries(&mut g);
        assert!(g.state.pending_triggers.is_empty());
        g.create_tokens(
            p,
            Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap(),
            1,
            false,
        );
        finish_direct_entries(&mut g);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|t| t.source.is_some_and(|identity| identity.id == source))
                .count(),
            1
        );
        let mut g = scenario(p);
        let definition = Card::from_name("Spiritcall Enthusiast").unwrap();
        g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(definition))],
            None,
        );
        finish_direct_entries(&mut g);
        let source = tokens(&g, p)[0];
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|t| t.source.is_some_and(|identity| identity.id == source))
                .count(),
            1
        );
        big_helpers::settle(&mut g);
        assert!(g.object_view(source).prepared);
    }
}

#[test]
fn sos_ennis_history_excludes_tokens_and_copies_reexile_counts_and_cleanup_preserves() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ennis, Debate Moderator", Zone::Battlefield);
        let prepared = put(&mut g, p, "Encouraging Aviator", Zone::Battlefield);
        g.prepare_permanent(g.object_ref(prepared));
        assert!(!g.observe(p).cards_exiled_this_turn);
        let copy = g.state.preparations.as_ref().unwrap()[0].spell_copy.card;
        g.move_object(copy, Zone::Exile);
        g.create_tokens(
            p,
            Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap(),
            1,
            false,
        );
        finish_direct_entries(&mut g);
        let token = tokens(&g, p)[0];
        g.move_object(token, Zone::Exile);
        assert!(!g.observe(p).cards_exiled_this_turn);
        g.begin_step(Step::End);
        big_helpers::settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        let card = put(&mut g, p.other(), "Forest", Zone::Exile);
        g.move_object(card, Zone::Exile);
        assert!(g.observe(p).cards_exiled_this_turn);
        g.begin_step(Step::End);
        big_helpers::settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        // A cleanup-induced SBA gives priority without starting the next turn.
        g.set_counters(source, Counters::of(CounterKind::MinusOne, 3));
        g.state.objects[source as usize].growth.toughness = 3;
        let turn = g.state.turn;
        g.finish_cleanup();
        assert_eq!(g.state.turn, turn);
        assert!(g.observe(p).cards_exiled_this_turn);
        g.begin_turn();
        assert!(!g.observe(p).cards_exiled_this_turn);
    }
}

#[test]
fn sos_mathemagics_powers_two_double_x_payment_and_huge_x_failed_draw_are_bounded() {
    {
        let p = Player::P0;
        for x in [0, 1, 2, 3, 128] {
            let mut g = scenario(p);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[1] = 2 * x + 2;
            let spell = put(&mut g, p, "Mathemagics", Zone::Hand);
            big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
            big_helpers::checked(&mut g, Action::ChooseX { value: x });
            big_helpers::checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana[1], 0);
            big_helpers::settle(&mut g);
            let expected = if x >= 128 { 8 } else { 1usize << x };
            assert_eq!(g.hand_count(p.other()), expected);
            if x >= 128 {
                assert!(g.outcome().is_some());
                assert!(g.state.players[p.other().index()].attempted_empty_draw);
            } else {
                assert!(g.outcome().is_none());
            }
        }
    }
}
