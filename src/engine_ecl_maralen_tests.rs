//! Paid entry, exact approving source, chosen-face value and once-per-player turn use.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
const NAME: &str = "Maralen, Fae Ascendant";
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    g.state.players[p.index()].library.clear();
    names
        .iter()
        .map(|name| named(g, p, name, Zone::Library))
        .collect()
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, NAME, Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 1, 0, 1, 2];
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn action(g: &Game, source: CardId, card: CardId, face: CastFace) -> Action {
    Action::AnnounceLinkedFree {
        card,
        source: g.object_ref(source),
        face,
        flash_surcharge: false,
    }
}
fn free(g: &mut Game, source: CardId, card: CardId, face: CastFace, targets: &[Target]) {
    checked(g, action(g, source, card, face));
    for &target in targets {
        for _ in 0..4 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("unpaid free cast");
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), r);
}
#[test]
fn ecl_maralen_paid_entry_targets_opponent_exiles_two_and_free_casts_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cards = library(&mut g, p.other(), &["Forest", "Shock", "Shock"]);
        let source = paid(&mut g, p);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 4,
                toughness: 5
            }
        );
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert_eq!(g.state.objects[cards[0] as usize].zone, Zone::Library);
        assert_eq!(g.observe(p).linked_exile, g.observe(p.other()).linked_exile);
        assert!(
            g.observe(p)
                .linked_exile
                .iter()
                .all(|row| row.source == g.object_ref(source) && row.exiled_turn == g.state.turn)
        );
        g.state.players[p.index()].mana = [0; 6];
        let a = action(&g, source, cards[2], CastFace::Normal);
        assert!(g.legal_actions().contains(&a));
        free(
            &mut g,
            source,
            cards[2],
            CastFace::Normal,
            &[Target::Player(p.other())],
        );
        assert_eq!(
            g.observe(p).linked_free_cast_uses,
            vec![LinkedFreeCastUseView {
                source: g.object_ref(source),
                ability_origin: 0,
                player: p
            }]
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        let a = action(&g, source, cards[1], CastFace::Normal);
        reject(&mut g, a);
        assert_eq!(
            g.observe(p).linked_free_cast_uses,
            g.observe(p.other()).linked_free_cast_uses
        );
    }
}
#[test]
fn ecl_maralen_union_counts_double_typed_once_and_noncreature_kindred_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cards = library(&mut g, p.other(), &["Bear Cub", "Serra Angel"]);
        let source = paid(&mut g, p);
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[0], CastFace::Normal)),
            "Maralen's Elf/Faerie types count once"
        );
        let ordinary = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[0], CastFace::Normal))
        );
        let kindred = named(&mut g, p, "Firdoch Core", Zone::Hand);
        g.move_object(kindred, Zone::Battlefield);
        settle(&mut g);
        assert!(
            g.legal_actions()
                .contains(&action(&g, source, cards[0], CastFace::Normal)),
            "noncreature changeling contributes one"
        );
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[1], CastFace::Normal))
        );
        g.move_object(ordinary, Zone::Graveyard);
        g.move_object(kindred, Zone::Graveyard);
        settle(&mut g);
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[0], CastFace::Normal)),
            "dynamic union count rechecks"
        );
    }
}
#[test]
fn ecl_maralen_chosen_adventure_face_value_and_normal_timing_and_x_zero() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cards = library(
            &mut g,
            p.other(),
            &[
                "Wildwood Scourge",
                "Bilbo Baggins, Burglar // Take a Glance",
            ],
        );
        let source = paid(&mut g, p);
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[1], CastFace::Normal))
        );
        assert!(
            g.legal_actions()
                .contains(&action(&g, source, cards[1], CastFace::Adventure))
        );
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[1], CastFace::Normal))
        );
        assert!(
            g.legal_actions()
                .contains(&action(&g, source, cards[1], CastFace::Adventure)),
            "adventure costs1 while physical body costs3"
        );
        g.state.active = p.other();
        g.state.priority = p;
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[1], CastFace::Normal)),
            "sorcery timing still applies"
        );
        g.state.active = p;
        g.state.step = Step::Main1;
        g.state.priority = p;
        g.move_object(elf, Zone::Graveyard);
        let a = action(&g, source, cards[0], CastFace::Normal);
        checked(&mut g, a);
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
        reject(&mut g, Action::ChooseX { value: 1 });
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        settle(&mut g);
        assert_eq!(g.state.objects[cards[0] as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn ecl_maralen_once_each_player_control_transfer_and_source_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cards = library(&mut g, p.other(), &["Shock", "Shock"]);
        let source = paid(&mut g, p);
        free(
            &mut g,
            source,
            cards[1],
            CastFace::Normal,
            &[Target::Player(p.other())],
        );
        settle(&mut g);
        g.state.objects[source as usize].controller = p.other();
        g.state.priority = p.other();
        assert!(
            g.legal_actions()
                .contains(&action(&g, source, cards[0], CastFace::Normal))
        );
        free(
            &mut g,
            source,
            cards[0],
            CastFace::Normal,
            &[Target::Player(p)],
        );
        settle(&mut g);
        assert_eq!(g.observe(p).linked_free_cast_uses.len(), 2);
        // A new incarnation cannot use the previous incarnation's links.
        let extra = named(&mut g, p.other(), "Shock", Zone::Library);
        g.move_object(extra, Zone::Exile);
        let old = g.object_ref(source);
        g.remember_linked_exile_for(old, &[extra]);
        g.move_object(source, Zone::Hand);
        g.state.objects[source as usize].controller = p;
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        g.state.priority = p;
        assert_ne!(g.object_ref(source), old);
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, extra, CastFace::Normal))
        );
    }
}
#[test]
fn ecl_maralen_old_turn_links_and_stale_exile_and_ability_loss_do_not_authorize() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cards = library(&mut g, p.other(), &["Shock", "Shock"]);
        let source = paid(&mut g, p);
        let old = g.object_ref(cards[0]);
        g.move_object(cards[0], Zone::Hand);
        g.move_object(cards[0], Zone::Exile);
        assert_ne!(g.object_ref(cards[0]), old);
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[0], CastFace::Normal))
        );
        let object = g.object_ref(source);
        g.remove_abilities_exact(&[object], crate::cards::AnimationDuration::EndOfTurn);
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[1], CastFace::Normal))
        );
        g.finish_cleanup();
        g.state.turn += 1;
        g.begin_turn();
        g.state.active = p;
        g.state.step = Step::Main1;
        g.state.priority = p;
        assert!(g.observe(p).linked_free_cast_uses.is_empty());
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[1], CastFace::Normal)),
            "previous-turn linked cards stay linked but not eligible"
        );
        let fresh = library(&mut g, p.other(), &["Shock"])[0];
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        g.move_object(elf, Zone::Battlefield);
        settle(&mut g);
        assert!(
            g.legal_actions()
                .contains(&action(&g, source, fresh, CastFace::Normal))
        );
    }
}
#[test]
fn ecl_maralen_additional_cost_remains_and_approving_source_can_leave_as_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cards = library(&mut g, p.other(), &["Eaten Alive"]);
        let source = paid(&mut g, p);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let a = action(&g, source, cards[0], CastFace::Normal);
        checked(&mut g, a);
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        let object = g.object_ref(source);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.observe(p).linked_free_cast_uses,
            vec![LinkedFreeCastUseView {
                source: object,
                ability_origin: 0,
                player: p
            }]
        );
        assert!(g.observe(p).linked_free_cast_approval.is_none());
        settle(&mut g);
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Exile);
    }
}
#[test]
fn ecl_maralen_free_adventure_reuses_independent_paid_permission_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cards = library(
            &mut g,
            p.other(),
            &["Bilbo Baggins, Burglar // Take a Glance"],
        );
        let source = paid(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        free(&mut g, source, cards[0], CastFace::Adventure, &[]);
        settle(&mut g);
        assert!(g.can_play_adventure(p, cards[0]));
        g.move_object(source, Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        checked(&mut g, Action::AnnounceExile { card: cards[0] });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 2],
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[cards[0] as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[cards[0] as usize].controller, p);
        assert_eq!(g.state.objects[cards[0] as usize].owner, p.other());
    }
}
#[test]
fn ecl_maralen_simultaneous_kindred_and_source_entries_trigger_once_each_and_departure_preserves_exile()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cards = library(&mut g, p.other(), &["Shock", "Shock", "Shock", "Shock"]);
        let source = named(&mut g, p, NAME, Zone::Hand);
        let kindred = named(&mut g, p, "Firdoch Core", Zone::Hand);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        let moves = [source, kindred, bear]
            .into_iter()
            .map(|id| g.entry_move(id, Zone::Battlefield, false))
            .collect();
        g.stage_entries(moves, None);
        // The source itself and the double-subtype changeling each trigger once.
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|s| s.card == source)
                .count()
                + g.state
                    .ordering_triggers
                    .iter()
                    .filter(|s| s.card == source)
                    .count(),
            2
        );
        let old = g.object_ref(source);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert!(
            cards
                .iter()
                .all(|&id| g.state.objects[id as usize].zone == Zone::Exile)
        );
        assert_eq!(g.observe(p).linked_exile.len(), 4);
        assert!(
            g.observe(p)
                .linked_exile
                .iter()
                .all(|row| row.source == old)
        );
        assert!(!g.has_linked_free_cast(p, cards[0]));
    }
}
#[test]
fn ecl_maralen_new_copy_ability_instance_uses_only_new_links_and_gets_independent_use() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cards = library(&mut g, p.other(), &["Shock", "Shock"]);
        let source = paid(&mut g, p);
        free(
            &mut g,
            source,
            cards[1],
            CastFace::Normal,
            &[Target::Player(p.other())],
        );
        settle(&mut g);
        let object = g.object_ref(source);
        let definition = g.state.objects[source as usize].card;
        g.state.copy_recipes = Some(
            vec![copies::CopyRow {
                object,
                recipe: CopyRecipe::new(definition),
                ability_origin: 42,
            }]
            .into(),
        );
        assert!(
            !g.legal_actions()
                .contains(&action(&g, source, cards[0], CastFace::Normal)),
            "old link is a different ability instance"
        );
        let fresh = library(&mut g, p.other(), &["Shock"])[0];
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        g.move_object(elf, Zone::Battlefield);
        settle(&mut g);
        assert!(
            g.observe(p)
                .linked_exile
                .iter()
                .any(|row| row.exiled.card == fresh && row.ability_origin == 42)
        );
        free(
            &mut g,
            source,
            fresh,
            CastFace::Normal,
            &[Target::Player(p.other())],
        );
        settle(&mut g);
        assert_eq!(
            g.observe(p)
                .linked_free_cast_uses
                .iter()
                .map(|row| row.ability_origin)
                .collect::<Vec<_>>(),
            vec![0, 42]
        );
    }
}
