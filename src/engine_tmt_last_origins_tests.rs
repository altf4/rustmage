//! Explicit physical outside-game pools preserve legality, ownership and private observations.
use super::*;
fn config(rules: DeckRules) -> Config {
    Config {
        decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
        seed: 913,
        starting_player: Player::P0,
        deck_rules: rules,
    }
}
#[test]
fn tmt_last_outside_private_supplied_cards_are_distinct_from_ceased_tokens() {
    let plains = Card::from_name("Plains").unwrap();
    let island = Card::from_name("Island").unwrap();
    let g = Game::new_with_sideboards(config(DeckRules::Standard), [vec![plains], vec![island]])
        .unwrap();
    let own = g.outside_game_cards(Player::P0).next().unwrap();
    let other = g.outside_game_cards(Player::P1).next().unwrap();
    assert_ne!(own, other);
    for p in [Player::P0, Player::P1] {
        let observation = g.observe(p);
        assert_eq!(observation.outside_game_cards.len(), 1);
        assert_eq!(observation.outside_game_cards[0].owner, p);
        assert!(!observation.objects.iter().any(|o| o.zone == Zone::Outside));
    }
    assert!(!g.outside_game_card_matches(Player::P0, other));
    let empty = Game::new(config(DeckRules::Standard)).unwrap();
    assert!(empty.outside_game_cards(Player::P0).next().is_none());
}
#[test]
fn tmt_last_outside_combined_copy_limits_sideboard_size_and_physical_faces_are_checked() {
    let bear = Card::from_name("Bear Cub").unwrap();
    let token = Card::from_name("@token/283fd83a9aec83a51b736926").unwrap();
    let mut c = config(DeckRules::Constructed);
    c.decks[0].splice(0..4, [bear; 4]);
    assert!(Game::new_with_sideboards(c, [vec![bear], vec![]]).is_err());
    assert!(
        Game::new_with_sideboards(
            config(DeckRules::Standard),
            [vec![Card::Forest; 16], vec![]]
        )
        .is_err()
    );
    assert!(Game::new_with_sideboards(config(DeckRules::Freeform), [vec![token], vec![]]).is_err());
    assert!(
        Game::new_with_sideboards(
            config(DeckRules::Freeform),
            [vec![Card::Forest; 16], vec![]]
        )
        .is_ok()
    );
}
#[test]
fn tmt_last_outside_exact_membership_movement_branch_and_restore() {
    let p = Player::P0;
    let mut g =
        Game::new_with_sideboards(config(DeckRules::Standard), [vec![Card::Forest], vec![]])
            .unwrap();
    let object = g.outside_game_cards(p).next().unwrap();
    let snapshot = g.snapshot();
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    let event = g.move_object(object.card, Zone::Hand);
    branch.move_object(object.card, Zone::Hand);
    assert!(event.moved);
    assert_eq!(g.snapshot(), branch.snapshot());
    assert!(!g.outside_game_card_matches(p, object));
    assert!(g.outside_game_cards(p).next().is_none());
    assert!(
        !g.observe(p.other())
            .objects
            .iter()
            .any(|o| o.id == object.card)
    );
    g.restore(&snapshot).unwrap();
    assert!(g.outside_game_card_matches(p, object));
    assert_eq!(g.observe(p).outside_game_cards[0].reference, object);
}
#[test]
fn tmt_last_outside_ceased_token_or_unregistered_outside_card_is_never_selectable() {
    let p = Player::P0;
    let mut g =
        Game::new_with_sideboards(config(DeckRules::Standard), [vec![Card::Forest], vec![]])
            .unwrap();
    let id = super::tests::spawn(&mut g, p, Card::Forest, Zone::Outside);
    let object = g.object_ref(id);
    assert!(!g.outside_game_card_matches(p, object));
    assert_eq!(g.outside_game_cards(p).count(), 1);
    let token = Card::from_name("@token/283fd83a9aec83a51b736926").unwrap();
    let id = super::tests::spawn(&mut g, p, token, Zone::Outside);
    let object = g.object_ref(id);
    assert!(!g.outside_game_card_matches(p, object));
    assert_eq!(g.outside_game_cards(p).count(), 1);
}
#[test]
fn tmt_last_entry_attacking_defender_chooses_independent_walker_and_restores() {
    use super::ability_timeline_tests::{checked, named, setup};
    for p in [Player::P0, Player::P1] {
        for choose_walker in [false, true] {
            let mut g = setup(p);
            let entering = named(&mut g, p, "Bear Cub", Zone::Exile);
            let walker = named(
                &mut g,
                p.other(),
                "Kaito, Cunning Infiltrator",
                Zone::Battlefield,
            );
            g.state.objects[walker as usize].counters.loyalty = 3;
            let mut movement = g.entry_move(entering, Zone::Battlefield, true);
            movement.controller = p;
            movement.choose_attacking_defender = true;
            g.stage_entries(vec![movement], None);
            assert!(g.advance_entries());
            assert_eq!(g.actor(), Some(p));
            assert_eq!(g.state.objects[entering as usize].zone, Zone::Exile);
            let defender = if choose_walker {
                CombatDefender::Planeswalker(g.object_ref(walker))
            } else {
                CombatDefender::Player(p.other())
            };
            let action = Action::ChooseHandDefender { defender };
            checked(&mut g, action);
            assert_eq!(g.state.objects[entering as usize].zone, Zone::Battlefield);
            assert!(g.state.objects[entering as usize].tapped);
            assert!(
                g.state.attackers.iter().any(
                    |attacker| attacker.card == entering && attacker.defender == Some(defender)
                )
            );
            assert_eq!(std::mem::size_of::<Object>(), 144);
            assert_eq!(std::mem::size_of::<Pending>(), 40);
        }
    }
}
#[test]
fn tmt_last_entry_attacking_only_player_auto_selects_and_noncreature_never_attacks() {
    use super::ability_timeline_tests::{named, setup};
    let p = Player::P0;
    let mut g = setup(p);
    let entering = named(&mut g, p, "Bear Cub", Zone::Exile);
    let mut movement = g.entry_move(entering, Zone::Battlefield, true);
    movement.choose_attacking_defender = true;
    g.stage_entries(vec![movement], None);
    assert!(!g.advance_entries());
    assert_eq!(
        g.state.attackers[0].defender,
        Some(CombatDefender::Player(p.other()))
    );
    let artifact = named(&mut g, p, "Mazemind Tome", Zone::Exile);
    let mut movement = g.entry_move(artifact, Zone::Battlefield, true);
    movement.choose_attacking_defender = true;
    g.stage_entries(vec![movement], None);
    assert!(!g.advance_entries());
    assert!(
        !g.state
            .attackers
            .iter()
            .any(|attacker| attacker.card == artifact)
    );
}

use super::ability_timeline_tests::{checked, named, next_turn, settle, setup};
use super::tmt_blink_permissions_tests::cast;
fn advance(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if predicate(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .into_iter()
            .find(|action| {
                matches!(
                    action,
                    Action::Pass
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::PayCosts { .. }
                        | Action::ChooseSpellCost { .. }
                        | Action::PutTrigger { .. }
                )
            })
            .unwrap_or_else(|| panic!("advance {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("advance exhausted {:?}", g.state.pending);
}
fn rejected(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn paid_last(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn mature_last(g: &mut Game) {
    next_turn(g);
    next_turn(g);
    while g.state.step != Step::Main1 {
        checked(g, Action::Pass);
    }
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
}
fn end_last(g: &mut Game) {
    advance(g, |g| g.state.step == Step::End);
}
fn kitsune_targets(g: &mut Game, left: CardId, right: CardId, accept: bool) {
    advance(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(left),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(right),
        },
    );
    advance(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    checked(g, Action::ChooseOptional { accept });
    settle(g);
}
fn koya_last(g: &mut Game, p: Player, target: Option<CardId>) -> CardId {
    let id = named(g, p, "Koya, Death from Above", Zone::Hand);
    cast(g, id, None, &[]);
    advance(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    if let Some(target) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    settle(g);
    id
}
#[test]
fn tmt_last_kitsune_paid_targets_different_controllers_optional_exchange_is_permanent() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let own = paid_last(&mut g, p, "Llanowar Elves");
            let second = paid_last(&mut g, p, "Bear Cub");
            let enemy = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let source = named(&mut g, p, "Kitsune, Dragon's Daughter", Zone::Hand);
            cast(&mut g, source, None, &[]);
            advance(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            rejected(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(source),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            rejected(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(second),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            advance(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            checked(&mut g, Action::ChooseOptional { accept });
            settle(&mut g);
            assert_eq!(
                g.state.objects[own as usize].controller,
                if accept { p.other() } else { p }
            );
            assert_eq!(
                g.state.objects[enemy as usize].controller,
                if accept { p } else { p.other() }
            );
            assert!(g.keywords(source).contains(Keywords::VIGILANCE));
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (6, 6));
            g.move_object(source, Zone::Graveyard);
            next_turn(&mut g);
            assert_eq!(
                g.state.objects[own as usize].controller,
                if accept { p.other() } else { p }
            );
        }
    }
}
#[test]
fn tmt_last_kitsune_partial_target_failure_never_exchanges_the_survivor() {
    let p = Player::P0;
    let mut g = setup(p);
    let own = paid_last(&mut g, p, "Llanowar Elves");
    let enemy = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
    let source = named(&mut g, p, "Kitsune, Dragon's Daughter", Zone::Hand);
    cast(&mut g, source, None, &[]);
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    advance(&mut g, |g| g.state.announcing.is_none());
    g.move_object(enemy, Zone::Hand);
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    checked(&mut g, Action::ChooseOptional { accept: true });
    settle(&mut g);
    assert_eq!(g.state.objects[own as usize].controller, p);
}
#[test]
fn tmt_last_kitsune_actual_combat_damage_creates_another_exchange_and_vigilance() {
    let p = Player::P0;
    let mut g = setup(p);
    let own = paid_last(&mut g, p, "Llanowar Elves");
    let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let source = named(&mut g, p, "Kitsune, Dragon's Daughter", Zone::Hand);
    cast(&mut g, source, None, &[]);
    kitsune_targets(&mut g, own, enemy, false);
    mature_last(&mut g);
    super::tmt_sneak_tests::unblocked(&mut g, source);
    assert!(!g.state.objects[source as usize].tapped);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    kitsune_targets(&mut g, own, enemy, true);
    assert_eq!(g.state.objects[own as usize].controller, p.other());
    assert_eq!(g.state.players[p.other().index()].life, 14);
}
#[test]
fn tmt_last_koya_paid_flying_optional_exile_and_delayed_payment_or_owner_return() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let creature = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let source = koya_last(&mut g, p, Some(creature));
            assert_eq!(g.state.objects[creature as usize].zone, Zone::Exile);
            assert_eq!(g.observe(p).delayed_steps.len(), 1);
            assert!(g.keywords(source).contains(Keywords::FLYING));
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (2, 1));
            g.move_object(source, Zone::Graveyard);
            end_last(&mut g);
            advance(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            g.state.players[p.index()].mana = [30; 6];
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
                g.state.objects[creature as usize].zone,
                if pay { Zone::Exile } else { Zone::Battlefield }
            );
            if !pay {
                assert_eq!(g.state.objects[creature as usize].controller, p.other());
            }
            assert!(g.observe(p).delayed_steps.is_empty());
        }
    }
}
#[test]
fn tmt_last_koya_zero_targets_still_schedules_optional_payment() {
    let p = Player::P0;
    let mut g = setup(p);
    named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
    koya_last(&mut g, p, None);
    assert_eq!(g.observe(p).delayed_steps.len(), 1);
    end_last(&mut g);
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
    checked(&mut g, Action::DeclineResolutionMana);
    settle(&mut g);
    assert!(g.observe(p).delayed_steps.is_empty());
}
#[test]
fn tmt_last_koya_exile_departure_and_new_incarnation_are_not_returned() {
    let p = Player::P0;
    let mut g = setup(p);
    let creature = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
    koya_last(&mut g, p, Some(creature));
    let old = g.object_ref(creature);
    g.move_object(creature, Zone::Graveyard);
    g.move_object(creature, Zone::Exile);
    assert_ne!(old, g.object_ref(creature));
    end_last(&mut g);
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
    checked(&mut g, Action::DeclineResolutionMana);
    settle(&mut g);
    assert_eq!(g.state.objects[creature as usize].zone, Zone::Exile);
}
#[test]
fn tmt_last_krang_hybrid_paid_entry_exiles_lands_until_nonland_with_exact_links() {
    for colors in [[0, 2, 0, 0, 0, 4], [0, 0, 2, 0, 0, 4], [0, 1, 1, 0, 0, 4]] {
        let p = Player::P0;
        let mut g = setup(p);
        let nonland = named(&mut g, p.other(), "Mazemind Tome", Zone::Library);
        let land1 = named(&mut g, p.other(), "Island", Zone::Library);
        let land2 = named(&mut g, p.other(), "Swamp", Zone::Library);
        g.state.players[p.index()].mana = colors;
        let source = paid_last(&mut g, p, "Krang & Shredder");
        for id in [nonland, land1, land2] {
            assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
        }
        assert_eq!(
            g.observe(p)
                .linked_exile
                .iter()
                .filter(|link| link.source == g.object_ref(source))
                .count(),
            3
        );
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (6, 7));
        end_last(&mut g);
        settle(&mut g);
        assert!(g.state.resolution_free_casts.is_none());
    }
}
#[test]
fn tmt_last_krang_disappear_casts_only_one_linked_spell_free_after_source_departure() {
    let p = Player::P0;
    let mut g = setup(p);
    let first = named(&mut g, p.other(), "Mazemind Tome", Zone::Library);
    let source = paid_last(&mut g, p, "Krang & Shredder");
    let own = paid_last(&mut g, p, "Llanowar Elves");
    let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
    cast(&mut g, bounce, None, &[Target::Card(own)]);
    settle(&mut g);
    end_last(&mut g);
    let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
    g.state.players[p.index()].mana = [30; 6];
    cast(&mut g, bounce, None, &[Target::Card(source)]);
    advance(&mut g, |g| g.state.resolution_free_casts.is_some());
    assert_eq!(g.state.objects[source as usize].zone, Zone::Hand);
    let object = g.object_ref(first);
    checked(
        &mut g,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::CastExiledDuringResolution { .. }))
    );
    checked(&mut g, Action::FinishResolutionCasts);
    settle(&mut g);
    assert_eq!(g.state.objects[first as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[first as usize].owner, p.other());
    assert_eq!(g.state.objects[first as usize].controller, p);
}
#[test]
fn tmt_last_krang_attack_exiles_again_and_new_source_cannot_cast_old_links() {
    let p = Player::P0;
    let mut g = setup(p);
    let old = named(&mut g, p.other(), "Mazemind Tome", Zone::Library);
    let source = paid_last(&mut g, p, "Krang & Shredder");
    mature_last(&mut g);
    let next = named(&mut g, p.other(), "Negate", Zone::Library);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(&mut g, Action::Attack { card: source });
    checked(&mut g, Action::FinishAttackers);
    settle(&mut g);
    assert_eq!(g.state.objects[next as usize].zone, Zone::Exile);
    g.move_object(source, Zone::Exile);
    g.move_object(source, Zone::Battlefield);
    settle(&mut g);
    end_last(&mut g);
    advance(&mut g, |g| g.state.resolution_free_casts.is_some());
    assert!(
        !g.legal_actions()
            .contains(&Action::CastExiledDuringResolution {
                object: g.object_ref(old),
                face: CastFace::Normal
            })
    );
    checked(&mut g, Action::FinishResolutionCasts);
    settle(&mut g);
}
#[test]
fn tmt_last_neutrinos_paid_alliance_growth_and_attack_blink_owns_not_controls() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid_last(&mut g, p, "The Neutrinos");
        assert!(g.keywords(source).contains(Keywords::FLYING));
        let own = paid_last(&mut g, p, "Llanowar Elves");
        assert_eq!(g.stats(source).power, 3);
        mature_last(&mut g);
        assert_eq!(g.stats(source).power, 2);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.record_control(g.object_ref(own), p.other(), None);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: source });
        checked(&mut g, Action::FinishAttackers);
        advance(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        rejected(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        settle(&mut g);
        assert!(g.state.objects[own as usize].tapped);
        assert_eq!(g.state.objects[own as usize].controller, p);
        assert!(g.state.attackers.iter().any(|a| a.card == own));
        assert_eq!(g.stats(source).power, 3);
    }
}
#[test]
fn tmt_last_neutrinos_blinks_itself_once_without_another_attack_trigger() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = paid_last(&mut g, p, "The Neutrinos");
    mature_last(&mut g);
    let before = g.object_ref(source);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(&mut g, Action::Attack { card: source });
    checked(&mut g, Action::FinishAttackers);
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    settle(&mut g);
    assert_ne!(g.object_ref(source), before);
    assert_eq!(
        g.state
            .attackers
            .iter()
            .filter(|a| a.card == source)
            .count(),
        1
    );
    assert!(g.state.objects[source as usize].tapped);
}
fn outside_setup(p: Player, sideboard: Vec<Card>) -> Game {
    let mut sideboards = [vec![Card::Forest], vec![Card::Forest]];
    sideboards[p.index()] = sideboard;
    let mut g = Game::new_with_sideboards(config(DeckRules::Standard), sideboards).unwrap();
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.turn = 3;
    for player in &mut g.state.players {
        player.kept = true;
        player.mana = [30; 6];
    }
    g
}
#[test]
fn tmt_last_north_wind_paid_wish_is_private_optional_and_can_choose_any_owned_card() {
    for p in [Player::P0, Player::P1] {
        for name in ["Forest", "Negate", "Mazemind Tome"] {
            let mut g = outside_setup(p, vec![Card::from_name(name).unwrap()]);
            let object = g.outside_game_cards(p).next().unwrap();
            let source = named(&mut g, p, "North Wind Avatar", Zone::Hand);
            cast(&mut g, source, None, &[]);
            advance(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            assert!(g.observe(p.other()).effect_choice.is_none());
            assert_eq!(
                g.observe(p).effect_choice.as_ref().unwrap().candidates,
                vec![object]
            );
            checked(&mut g, Action::ChooseOutsideGameCard { object });
            settle(&mut g);
            assert_eq!(g.state.objects[object.card as usize].zone, Zone::Hand);
            assert!(g.outside_game_cards(p).next().is_none());
            assert!(
                !g.observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.id == object.card)
            );
            assert!(
                !g.observe(p.other())
                    .known_hand
                    .contains(&g.object_ref(object.card))
            );
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (5, 5));
            assert!(g.keywords(source).contains(Keywords::FLYING));
        }
    }
}
#[test]
fn tmt_last_north_wind_empty_and_nonempty_outside_choices_have_same_peer_phase() {
    let p = Player::P0;
    for cards in [vec![], vec![Card::Forest]] {
        let mut g = outside_setup(p, cards);
        let source = named(&mut g, p, "North Wind Avatar", Zone::Hand);
        cast(&mut g, source, None, &[]);
        advance(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        assert!(g.observe(p.other()).effect_choice.is_none());
        assert_eq!(g.actor(), Some(p));
        assert!(g.legal_actions().contains(&Action::DeclineEffectPayment));
        checked(&mut g, Action::DeclineEffectPayment);
        settle(&mut g);
        assert!(g.on_battlefield(source));
    }
}
#[test]
fn tmt_last_north_wind_noncast_return_does_not_wish_and_exile_cast_does() {
    let p = Player::P0;
    let mut g = outside_setup(p, vec![Card::Forest]);
    let source = named(&mut g, p, "North Wind Avatar", Zone::Exile);
    g.move_object(source, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.outside_game_cards(p).count(), 1);
    assert!(g.state.resolving.is_none());
    g.move_object(source, Zone::Exile);
    g.grant_exile_cast_permission(p, g.object_ref(source));
    checked(&mut g, Action::AnnounceExile { card: source });
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    checked(&mut g, Action::DeclineEffectPayment);
    settle(&mut g);
    assert!(g.on_battlefield(source));
}
#[test]
fn tmt_last_north_wind_spell_copy_was_not_cast_but_original_wishes() {
    let p = Player::P0;
    let mut g = outside_setup(p, vec![Card::Forest]);
    let source = named(&mut g, p, "North Wind Avatar", Zone::Hand);
    cast(&mut g, source, None, &[]);
    let spell = g.state.stack.last().unwrap().clone();
    let mut driver = g.activation_item(p, source, 0);
    driver.copy_event = Some(Box::new(stack_copies::CopyEvent { spell, amount: 1 }));
    assert!(!g.copy_captured_spell(&driver));
    let copy = g.state.stack.last().unwrap().card;
    assert_ne!(copy, source);
    advance(&mut g, |g| g.on_battlefield(copy));
    assert_eq!(g.outside_game_cards(p).count(), 1);
    assert!(g.state.resolving.is_none());
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    checked(&mut g, Action::DeclineEffectPayment);
    settle(&mut g);
}
#[test]
fn tmt_last_north_wind_foreign_and_stale_outside_selection_reject_atomically() {
    let p = Player::P0;
    let mut g = outside_setup(p, vec![Card::from_name("Negate").unwrap()]);
    let own = g.outside_game_cards(p).next().unwrap();
    let foreign = g.outside_game_cards(p.other()).next().unwrap();
    let source = named(&mut g, p, "North Wind Avatar", Zone::Hand);
    cast(&mut g, source, None, &[]);
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    rejected(&mut g, Action::ChooseOutsideGameCard { object: foreign });
    g.move_object(own.card, Zone::Hand);
    rejected(&mut g, Action::ChooseOutsideGameCard { object: own });
    checked(&mut g, Action::DeclineEffectPayment);
    settle(&mut g);
}
#[test]
fn tmt_last_neutrinos_paid_blink_can_choose_a_different_planeswalker_defender() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = paid_last(&mut g, p, "The Neutrinos");
    mature_last(&mut g);
    let creature = paid_last(&mut g, p, "Bear Cub");
    let walker = named(
        &mut g,
        p.other(),
        "Kaito, Cunning Infiltrator",
        Zone::Battlefield,
    );
    g.state.objects[walker as usize].counters.loyalty = 3;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(&mut g, Action::Attack { card: source });
    checked(&mut g, Action::FinishAttackers);
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::EntryAttackingDefender { .. })
    });
    let defender = CombatDefender::Planeswalker(g.object_ref(walker));
    rejected(
        &mut g,
        Action::ChooseHandDefender {
            defender: CombatDefender::Player(p),
        },
    );
    checked(&mut g, Action::ChooseHandDefender { defender });
    settle(&mut g);
    assert!(
        g.state
            .attackers
            .iter()
            .any(|attacker| attacker.card == creature && attacker.defender == Some(defender))
    );
    assert!(
        g.state
            .attackers
            .iter()
            .any(|attacker| attacker.card == source
                && attacker.defender == Some(CombatDefender::Player(p.other())))
    );
}
#[test]
fn tmt_last_koya_invalidated_selected_target_fizzles_without_delayed_payment() {
    let p = Player::P0;
    let mut g = setup(p);
    let creature = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
    let source = named(&mut g, p, "Koya, Death from Above", Zone::Hand);
    cast(&mut g, source, None, &[]);
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    rejected(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    advance(&mut g, |g| g.state.announcing.is_none());
    g.move_object(creature, Zone::Hand);
    settle(&mut g);
    assert!(g.observe(p).delayed_steps.is_empty());
    assert_eq!(g.state.objects[creature as usize].zone, Zone::Hand);
}
#[test]
fn tmt_last_krang_new_copy_ability_origin_cannot_cast_previous_links() {
    let p = Player::P0;
    let mut g = setup(p);
    let old = named(&mut g, p.other(), "Mazemind Tome", Zone::Library);
    let source = paid_last(&mut g, p, "Krang & Shredder");
    let object = g.object_ref(source);
    let recipe = g.copiable_recipe(object);
    g.copy_permanents_exact(&[object], recipe);
    let permanent = paid_last(&mut g, p, "Llanowar Elves");
    let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
    cast(&mut g, bounce, None, &[Target::Card(permanent)]);
    settle(&mut g);
    end_last(&mut g);
    settle(&mut g);
    assert!(g.state.resolution_free_casts.is_none());
    assert_eq!(g.state.objects[old as usize].zone, Zone::Exile);
}
#[test]
fn tmt_last_private_wish_query_footprint_is_bounded_and_owned() {
    let p = Player::P0;
    let mut g = outside_setup(p, vec![Card::Forest; 15]);
    let source = named(&mut g, p, "North Wind Avatar", Zone::Hand);
    cast(&mut g, source, None, &[]);
    advance(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    let bytes = g.state_bytes();
    let start = std::time::Instant::now();
    for _ in 0..100 {
        std::hint::black_box(g.legal_actions());
    }
    eprintln!(
        "TMT wish15 100legalqueries {:?}; initialized bytes {}; no state growth",
        start.elapsed(),
        bytes
    );
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<CastInfo>(), 40);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
    checked(&mut g, Action::DeclineEffectPayment);
    settle(&mut g);
}
