//! Whole-body casting-origin rules, with paid actions and owned replay.
use super::ability_timeline_tests::{checked, settle, setup};
use super::tests::spawn;
use super::*;
fn put(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn finish(g: &mut Game) {
    for _ in 0..40 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unfinished announcement");
}
#[test]
fn cast_origins_vizier_private_live_top_paid_cast_and_hidden_next_during_announcement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vizier = put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let next = put(&mut g, p, "Shock", Zone::Library);
        let lion = put(&mut g, p, "Savannah Lions", Zone::Library);
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            g.object_ref(lion)
        );
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, Action::AnnounceCast { card: lion });
        assert!(g.observe(p).known_library_top[p.index()].is_none());
        finish(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 0, 0, 0, 0, 1]
        );
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            g.object_ref(next)
        );
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: next })
        );
        settle(&mut g);
        assert_eq!(g.state.objects[lion as usize].zone, Zone::Battlefield);
        g.move_object(vizier, Zone::Graveyard);
        assert!(g.observe(p).known_library_top[p.index()].is_none());
    }
}
#[test]
fn cast_origins_vizier_all_origins_creature_only_and_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let lion = put(&mut g, p, "Savannah Lions", Zone::Hand);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: lion })
        );
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: shock })
        );
        g.state.active = p.other();
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: lion })
        );
        let flash = put(&mut g, p, "Brineborn Cutthroat", Zone::Library);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: flash })
        );
        checked(&mut g, Action::AnnounceCast { card: flash });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[flash as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn cast_origins_vizier_lost_ability_and_required_provider_reservation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vizier = put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let lion = put(&mut g, p, "Savannah Lions", Zone::Hand);
        let land = put(&mut g, p, "Wastes", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: lion })
        );
        let turn = put(&mut g, p.other(), "Witness Protection", Zone::Hand);
        // Normal public entry attaches the ability-removing Aura.
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 0];
        g.state.active = p.other();
        g.state.priority = p.other();
        super::ability_timeline_tests::cast(&mut g, turn, None, &[Target::Card(vizier)]);
        settle(&mut g);
        g.state.active = p;
        g.state.priority = p;
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: lion })
        );
        assert_eq!(g.state.objects[land as usize].zone, Zone::Battlefield);
    }
}
fn graveyard_play(card: CardId, permission: ObjectRef, permanent_type: PermanentType) -> Action {
    Action::PlayGraveyardPermanent {
        card,
        permission,
        permanent_type,
    }
}
#[test]
fn cast_origins_greatwurm_counter_permission_and_muldrotha_quota_are_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let muldrotha = put(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        let vizier = put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let permission = g.object_ref(muldrotha);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(bear, CounterKind::Bait, 6);
        let counters = g.object_ref(bear);
        let first = put(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
        let second = put(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
        let third = put(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
        // Vizier substitutes actual mana for the green pips on either route;
        // Greatwurm's own permission pays six counters without using Muldrotha.
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
        checked(&mut g, Action::AnnounceCounterGraveyard { card: first });
        for _ in 0..6 {
            checked(
                &mut g,
                Action::ChooseCreatureCounterCost {
                    object: counters,
                    counter: CounterKind::Bait,
                },
            );
        }
        finish(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::Bait), 0);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 0, 0, 0, 0, 6]
        );
        settle(&mut g);
        assert!(g.on_battlefield(first));
        assert!(g.observe(p).graveyard_type_uses.is_empty());

        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCounterGraveyard { card: second })
        );
        checked(
            &mut g,
            graveyard_play(second, permission, PermanentType::Creature),
        );
        finish(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::Bait), 0);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert!(g.on_battlefield(second));
        assert_eq!(g.observe(p).graveyard_type_uses.len(), 1);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
        let before = g.snapshot();
        let revision = g.revision();
        for action in [
            Action::AnnounceCounterGraveyard { card: third },
            graveyard_play(third, permission, PermanentType::Creature),
        ] {
            assert_eq!(g.apply(p, action), Err(Error::IllegalAction));
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), revision);
        }
        g.add_counter(bear, CounterKind::Bait, 6);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCounterGraveyard { card: third })
        );
        g.move_object(vizier, Zone::Graveyard);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCounterGraveyard { card: third })
        );
    }
}
#[test]
fn cast_origins_muldrotha_multitype_choice_and_independent_source_incarnations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = put(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        let source = g.object_ref(m);
        let a = put(&mut g, p, "Solemn Simulacrum", Zone::Graveyard);
        let b = put(&mut g, p, "Meteor Golem", Zone::Graveyard);
        let c = put(&mut g, p, "Savannah Lions", Zone::Graveyard);
        g.state.players[p.index()].mana = [10; 6];
        assert!(
            g.legal_actions()
                .contains(&graveyard_play(a, source, PermanentType::Artifact))
        );
        assert!(
            g.legal_actions()
                .contains(&graveyard_play(a, source, PermanentType::Creature))
        );
        checked(&mut g, graveyard_play(a, source, PermanentType::Artifact));
        finish(&mut g);
        settle(&mut g);
        assert!(
            !g.legal_actions()
                .contains(&graveyard_play(b, source, PermanentType::Artifact))
        );
        assert!(
            g.legal_actions()
                .contains(&graveyard_play(c, source, PermanentType::Creature))
        );
        checked(&mut g, graveyard_play(c, source, PermanentType::Creature));
        finish(&mut g);
        settle(&mut g);
        assert!(
            !g.legal_actions()
                .contains(&graveyard_play(b, source, PermanentType::Creature))
        );
        g.move_object(m, Zone::Graveyard);
        g.move_object(m, Zone::Battlefield);
        let fresh = g.object_ref(m);
        assert_ne!(fresh, source);
        assert!(
            g.legal_actions()
                .contains(&graveyard_play(b, fresh, PermanentType::Artifact))
        );
        let saved = g.snapshot();
        let rev = g.revision();
        assert_eq!(
            g.apply(p, graveyard_play(b, source, PermanentType::Artifact)),
            Err(Error::IllegalAction)
        );
        assert_eq!(saved, g.snapshot());
        assert_eq!(rev, g.revision());
    }
}
#[test]
fn cast_origins_muldrotha_land_allowance_turn_reset_owner_and_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = put(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        let source = g.object_ref(m);
        let land = put(&mut g, p, "Plains", Zone::Graveyard);
        let land2 = put(&mut g, p, "Island", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Savannah Lions", Zone::Graveyard);
        let instant = put(&mut g, p, "Shock", Zone::Graveyard);
        let flash = put(&mut g, p, "Brineborn Cutthroat", Zone::Graveyard);
        g.state.players[p.index()].mana = [10; 6];
        assert!(!g.legal_actions().contains(&graveyard_play(
            enemy,
            source,
            PermanentType::Creature
        )));
        assert!(!g.legal_actions().contains(&graveyard_play(
            instant,
            source,
            PermanentType::Creature
        )));
        checked(&mut g, graveyard_play(land, source, PermanentType::Land));
        assert!(g.state.land_played);
        assert!(
            !g.legal_actions()
                .contains(&graveyard_play(land2, source, PermanentType::Land))
        );
        g.state.active = p.other();
        assert!(!g.legal_actions().contains(&graveyard_play(
            flash,
            source,
            PermanentType::Creature
        )));
        g.state.active = p;
        g.state.step = Step::End;
        assert!(g.legal_actions().contains(&graveyard_play(
            flash,
            source,
            PermanentType::Creature
        )));
        g.state.step = Step::Main1;
        g.state.turn += 2;
        g.state.land_played = false;
        assert!(
            g.legal_actions()
                .contains(&graveyard_play(land2, source, PermanentType::Land))
        );
    }
}
#[test]
fn cast_origins_tinybones_paid_sorcery_discard_exact_stash_and_foreign_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tiny = put(&mut g, p, "Tinybones, Bauble Burglar", Zone::Hand);
        super::ability_timeline_tests::cast(&mut g, tiny, None, &[]);
        settle(&mut g);
        g.state.objects[tiny as usize].summoning_sick = false;
        let card = put(&mut g, p.other(), "Savannah Lions", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: tiny,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        finish(&mut g);
        assert!(g.state.objects[tiny as usize].tapped);
        for _ in 0..50 {
            if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(&mut g, Action::Discard { card });
        settle(&mut g);
        assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
        assert_eq!(g.counter_count(card, CounterKind::Stash), 1);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        assert!(g.legal_actions().contains(&Action::AnnounceExile { card }));
        checked(&mut g, Action::AnnounceExile { card });
        finish(&mut g);
        assert!(g.state.stack.last().unwrap().cast.unwrap().stash_origin);
        assert_eq!(g.counter_count(card, CounterKind::Stash), 0);
        settle(&mut g);
        assert_eq!(g.state.objects[card as usize].controller, p);
        assert_eq!(g.state.objects[card as usize].owner, p.other());
        g.move_object(card, Zone::Graveyard);
        assert_eq!(g.state.objects[card as usize].owner, p.other());
    }
}
#[test]
fn cast_origins_tinybones_exact_discard_generation_source_departure_and_shared_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tiny = put(&mut g, p, "Tinybones, Bauble Burglar", Zone::Battlefield);
        let card = put(&mut g, p.other(), "Savannah Lions", Zone::Hand);
        g.discard_batch(&[card], None);
        assert_eq!(g.state.pending_triggers.len(), 1);
        let bound = g.state.pending_triggers[0]
            .reflexive_subject
            .as_ref()
            .unwrap()
            .object;
        assert_eq!(bound, g.object_ref(card));
        g.move_object(tiny, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
        assert_eq!(g.counter_count(card, CounterKind::Stash), 1);
        assert!(!g.legal_actions().contains(&Action::AnnounceExile { card }));
        g.move_object(tiny, Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        assert!(g.legal_actions().contains(&Action::AnnounceExile { card }));
        g.state.active = p.other();
        assert!(!g.legal_actions().contains(&Action::AnnounceExile { card }));
        g.state.active = p;
        let own = put(&mut g, p, "Savannah Lions", Zone::Exile);
        g.add_counter(own, CounterKind::Stash, 1);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceExile { card: own })
        );
        let stale = put(&mut g, p.other(), "Shock", Zone::Hand);
        g.discard_batch(&[stale], None);
        g.move_object(stale, Zone::Hand);
        g.move_object(stale, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.objects[stale as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn cast_origins_tinybones_paid_stash_allows_adventure_and_omen_with_actual_mana() {
    for p in [Player::P0, Player::P1] {
        for omen in [false, true] {
            let mut g = setup(p);
            let tiny = put(&mut g, p, "Tinybones, Bauble Burglar", Zone::Battlefield);
            g.state.objects[tiny as usize].summoning_sick = false;
            let (name, target) = if omen {
                (
                    "Twinmaw Stormbrood // Charring Bite",
                    put(&mut g, p.other(), "Rampaging Baloths", Zone::Battlefield),
                )
            } else {
                (
                    "Fell Horseman // Deathly Ride",
                    put(&mut g, p, "Bear Cub", Zone::Graveyard),
                )
            };
            let card = put(&mut g, p.other(), name, Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: tiny,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            finish(&mut g);
            for _ in 0..50 {
                if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            checked(&mut g, Action::Discard { card });
            settle(&mut g);
            assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
            assert_eq!(g.counter_count(card, CounterKind::Stash), 1);
            let announce = if omen {
                Action::AnnounceOmen { card }
            } else {
                Action::AnnounceAdventure { card }
            };
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            g.state.active = p.other();
            assert!(!g.legal_actions().contains(&announce));
            g.state.active = p;
            checked(&mut g, announce);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish(&mut g);
            let cast = g.state.stack.last().unwrap().cast.unwrap();
            assert!(cast.stash_origin);
            assert_eq!(
                cast.face,
                if omen {
                    CastFace::Omen
                } else {
                    CastFace::Adventure
                }
            );
            assert_eq!(cast.payment, [0, 0, 0, 0, 0, 2]);
            assert_eq!(g.counter_count(card, CounterKind::Stash), 0);
            settle(&mut g);
            assert_eq!(g.state.objects[card as usize].owner, p.other());
            if omen {
                assert_eq!(g.state.objects[target as usize].damage, 5);
                assert_eq!(g.state.objects[card as usize].zone, Zone::Library);
                assert!(g.state.players[p.other().index()].library.contains(&card));
            } else {
                assert_eq!(g.state.objects[target as usize].zone, Zone::Hand);
                assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
                assert!(g.can_play_adventure(p, card));
                assert!(!g.can_play_foreign_stash(p, card));
            }
            // A creature-only library permission must not admit its spell face.
            put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
            let library = put(&mut g, p, name, Zone::Library);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 10];
            let alternate = if omen {
                Action::AnnounceOmen { card: library }
            } else {
                Action::AnnounceAdventure { card: library }
            };
            assert!(!g.legal_actions().contains(&alternate));
        }
    }
}
#[test]
fn cast_origins_tinybones_stash_land_and_activation_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tiny = put(&mut g, p, "Tinybones, Bauble Burglar", Zone::Battlefield);
        let land = put(&mut g, p.other(), "Plains", Zone::Exile);
        g.add_counter(land, CounterKind::Stash, 1);
        let action = Action::AnnounceActivate {
            card: tiny,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.step = Step::End;
        assert!(!g.legal_actions().contains(&action));
        assert!(!g.legal_actions().contains(&Action::PlayLand { card: land }));
        g.state.step = Step::Main1;
        checked(&mut g, Action::PlayLand { card: land });
        assert_eq!(g.state.objects[land as usize].controller, p);
        assert_eq!(g.state.objects[land as usize].owner, p.other());
        assert_eq!(g.counter_count(land, CounterKind::Stash), 0);
        assert!(g.state.land_played);
    }
}
fn etali_attack(g: &mut Game, p: Player, etali: CardId) {
    g.state.players[p.index()].mana = [0; 6];
    g.state.objects[etali as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card: etali });
    checked(g, Action::FinishAttackers);
    for _ in 0..50 {
        if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("no Etali resolution");
}
fn effect_cast(object: ObjectRef) -> Action {
    Action::CastExiledDuringResolution {
        object,
        face: CastFace::Normal,
    }
}
#[test]
fn cast_origins_etali_paid_attack_foreign_free_cast_order_and_target_new_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let etali = put(&mut g, p, "Etali, Primal Storm", Zone::Hand);
        super::ability_timeline_tests::cast(&mut g, etali, None, &[]);
        settle(&mut g);
        let negate = put(&mut g, p, "Negate", Zone::Library);
        let shock = put(&mut g, p.other(), "Shock", Zone::Library);
        etali_attack(&mut g, p, etali);
        let shock_ref = g.object_ref(shock);
        let negate_ref = g.object_ref(negate);
        assert!(g.state.resolving.is_some());
        assert!(!g.legal_actions().contains(&Action::Pass));
        assert!(!g.legal_actions().contains(&effect_cast(negate_ref))); // no noncreature spell yet
        checked(&mut g, effect_cast(shock_ref));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish(&mut g);
        assert_eq!(g.state.stack.last().unwrap().controller, p);
        assert_eq!(g.state.stack.last().unwrap().cast.unwrap().payment, [0; 6]);
        checked(&mut g, effect_cast(negate_ref));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(shock),
            },
        );
        finish(&mut g);
        checked(&mut g, Action::FinishResolutionCasts);
        assert!(g.state.resolving.is_none());
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(g.state.objects[shock as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[shock as usize].owner, p.other());
    }
}
#[test]
fn cast_origins_etali_optional_decline_land_and_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let etali = put(&mut g, p, "Etali, Primal Storm", Zone::Battlefield);
        let land = put(&mut g, p, "Plains", Zone::Library);
        let lion = put(&mut g, p.other(), "Savannah Lions", Zone::Library);
        etali_attack(&mut g, p, etali);
        assert!(!g.legal_actions().contains(&effect_cast(g.object_ref(land))));
        let old = g.object_ref(lion);
        g.move_object(lion, Zone::Graveyard);
        g.move_object(lion, Zone::Exile);
        let saved = g.snapshot();
        assert_eq!(g.apply(p, effect_cast(old)), Err(Error::IllegalAction));
        assert_eq!(g.snapshot(), saved);
        assert!(!g.legal_actions().contains(&effect_cast(g.object_ref(lion))));
        checked(&mut g, Action::FinishResolutionCasts);
        assert_eq!(g.state.objects[land as usize].zone, Zone::Exile);
        assert_eq!(g.state.objects[lion as usize].zone, Zone::Exile);
    }
}
#[test]
fn cast_origins_etali_free_x_zero_and_mandatory_additional_discard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let etali = put(&mut g, p, "Etali, Primal Storm", Zone::Battlefield);
        let x = put(&mut g, p, "Exsanguinate", Zone::Library);
        let thrill = put(&mut g, p.other(), "Thrill of Possibility", Zone::Library);
        etali_attack(&mut g, p, etali);
        assert!(
            !g.legal_actions()
                .contains(&effect_cast(g.object_ref(thrill)))
        );
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        let object = g.object_ref(thrill);
        checked(&mut g, effect_cast(object));
        let object = g.object_ref(discard);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        finish(&mut g);
        let object = g.object_ref(x);
        checked(&mut g, effect_cast(object));
        finish(&mut g);
        assert_eq!(g.state.stack.last().unwrap().x, Some(0));
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.state.objects[discard as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn cast_origins_vizier_mana_substitution_does_not_change_convoke_colors() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let mut bears = Vec::new();
        for _ in 0..4 {
            bears.push(put(&mut g, p, "Bear Cub", Zone::Battlefield));
        }
        let card = put(&mut g, p, "Sun-Dappled Celebrant", Zone::Hand); // {4}{W}{W}, convoke
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        assert!(!g.legal_actions().contains(&Action::AnnounceCast { card }));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, Action::AnnounceCast { card });
        checked(&mut g, Action::BeginConvoke);
        for &bear in &bears {
            let object = g.object_ref(bear);
            assert!(!g.legal_actions().contains(&Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::White
            }));
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution: ConvokeContribution::Generic,
                },
            );
        }
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[card as usize].zone, Zone::Battlefield);
        assert!(
            bears
                .into_iter()
                .all(|id| g.state.objects[id as usize].tapped)
        );
    }
}
#[test]
fn cast_origins_etali_free_time_stop_exiles_remaining_stack_and_resets_actual_turn_quota() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let muldrotha = put(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        let source = g.object_ref(muldrotha);
        let lion = put(&mut g, p, "Savannah Lions", Zone::Graveyard);
        checked(
            &mut g,
            graveyard_play(lion, source, PermanentType::Creature),
        );
        finish(&mut g);
        settle(&mut g);
        let next = put(&mut g, p, "Brineborn Cutthroat", Zone::Graveyard);
        assert!(!g.legal_actions().contains(&graveyard_play(
            next,
            source,
            PermanentType::Creature
        )));
        let etali = put(&mut g, p, "Etali, Primal Storm", Zone::Battlefield);
        let stop = put(&mut g, p, "Time Stop", Zone::Library);
        let shock = put(&mut g, p.other(), "Shock", Zone::Library);
        etali_attack(&mut g, p, etali);
        let object = g.object_ref(shock);
        checked(&mut g, effect_cast(object));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish(&mut g);
        let object = g.object_ref(stop);
        checked(&mut g, effect_cast(object));
        finish(&mut g);
        assert_eq!(g.state.stack.len(), 2);
        checked(&mut g, Action::FinishResolutionCasts);
        let turn = g.state.turn;
        settle(&mut g);
        assert_eq!(g.state.turn, turn + 1);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.state.objects[shock as usize].zone, Zone::Exile);
        assert_eq!(g.state.objects[stop as usize].zone, Zone::Exile);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert!(g.state.resolution_free_casts.is_none());
        assert!(g.state.graveyard_type_uses.is_none());
        assert!(!g.legal_actions().contains(&graveyard_play(
            next,
            source,
            PermanentType::Creature
        )));
        super::ability_timeline_tests::next_turn(&mut g);
        assert_eq!(g.state.active, p);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        assert!(
            g.legal_actions()
                .contains(&graveyard_play(next, source, PermanentType::Creature))
        );
    }
}
#[test]
fn cast_origins_muldrotha_quotas_survive_cleanup_priority_and_retire_departed_sources() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = put(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        let source = g.object_ref(m);
        let card = put(&mut g, p, "Brineborn Cutthroat", Zone::Graveyard);
        checked(
            &mut g,
            graveyard_play(card, source, PermanentType::Creature),
        );
        finish(&mut g);
        settle(&mut g);
        let next = put(&mut g, p, "Brineborn Cutthroat", Zone::Graveyard);
        // A trigger awaiting placement gives a cleanup priority cycle; it does
        // not make this a fresh turn or replenish the source's printed quota.
        let rat = put(&mut g, p, "Burglar Rat", Zone::Battlefield);
        let rat_view = g.object_view(rat);
        assert!(g.queue_trigger(&rat_view, 0));
        g.state.step = Step::Cleanup;
        let turn = g.state.turn;
        g.finish_cleanup();
        assert_eq!(g.state.turn, turn);
        assert!(
            g.state
                .graveyard_type_uses
                .as_ref()
                .is_some_and(|rows| !rows.is_empty())
        );
        assert!(!g.legal_actions().contains(&graveyard_play(
            next,
            source,
            PermanentType::Creature
        )));
        g.move_object(m, Zone::Graveyard);
        assert!(g.state.graveyard_type_uses.is_none());
    }
}
