//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn ready(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if predicate(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("inherited Adventure resolution budget");
}
fn paid(g: &mut Game, card: CardId, adventure: bool, target: Option<Target>, mana: Mana) {
    g.state.players[g.actor().unwrap().index()].mana = mana;
    act(
        g,
        if adventure {
            Action::AnnounceAdventure { card }
        } else if g.object_view(card).zone == Zone::Exile {
            Action::AnnounceExile { card }
        } else {
            Action::AnnounceCast { card }
        },
    );
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        choose(g, |a| matches!(a, Action::ChooseSpellCost { .. }));
    }
    if let Some(target) = target {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    let payer = g.state.stack.last().unwrap().controller;
    assert_eq!(g.state.players[payer.index()].mana, [0; 6]);
}
fn no_permission(g: &mut Game, p: Player, card: CardId) {
    assert!(!g.can_play_adventure(p, card));
    assert!(!g.can_play_adventure(p.other(), card));
    reject(g, Action::AnnounceExile { card });
}

#[test]
fn hob_inherited_countered_take_a_glance_never_scries_or_grants_permission() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bilbo = put(&mut g, p, "Bilbo Baggins, Burglar", Zone::Hand);
        let counter = put(&mut g, p.other(), "Negate", Zone::Hand);
        let library = g.state.players[p.index()].library.clone();
        paid(&mut g, bilbo, true, None, [0, 1, 0, 0, 0, 0]);

        act(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p.other()));
        paid(
            &mut g,
            counter,
            false,
            Some(Target::Spell(bilbo)),
            [0, 1, 0, 0, 0, 1],
        );
        settle(&mut g);
        assert_eq!(g.object_view(bilbo).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library, library);
        assert!(g.observe(p).library_choice.is_none());
        no_permission(&mut g, p, bilbo);
    }
}

#[test]
fn hob_inherited_smaug_paid_creature_only_five_damage_then_flying_creature() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Galewind Moose", Zone::Battlefield);
        let smaug = put(&mut g, p, "Smaug, the Great Calamity", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
        act(&mut g, Action::AnnounceAdventure { card: smaug });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.spell_characteristics().last().unwrap().name, "Spew Flame");
        settle(&mut g);
        assert_eq!(g.object_view(target).damage, 5);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert!(g.can_play_adventure(p, smaug));
        paid(&mut g, smaug, false, None, [0, 0, 0, 2, 0, 5]);
        settle(&mut g);
        assert_eq!(stats(&g, smaug), (5, 5));
        assert!(g.keywords(smaug).contains(Keywords::FLYING));
        assert_eq!(g.object_view(smaug).zone, Zone::Battlefield);
        assert!(!g.can_play_adventure(p, smaug));
    }
}

#[test]
fn hob_inherited_thranduil_each_hybrid_face_exact_mill_cardinality_and_owned_choice() {
    {
        let p = Player::P0;
        for (count, colors) in [[0, 0, 0, 0, 2, 0], [0, 1, 0, 0, 1, 0], [0, 2, 0, 0, 0, 0]]
            .into_iter()
            .enumerate()
        {
            let mut g = scenario(p);
            let old_land = put(&mut g, p, "Forest", Zone::Graveyard);
            let one = put(&mut g, p, "Forest", Zone::Library);
            let spell = put(&mut g, p, "Shock", Zone::Library);
            let two = put(&mut g, p, "Island", Zone::Library);
            let creature = put(&mut g, p, "Grizzly Bears", Zone::Library);
            let thranduil = put(&mut g, p, "Thranduil, Sindarin Liege", Zone::Hand);
            let mut mana = colors;
            mana[5] = 1;
            paid(&mut g, thranduil, true, None, mana);
            ready(&mut g, |g| g.observe(p).milled_choice.is_some());
            let choice = g.observe(p).milled_choice.unwrap();
            assert_eq!(choice.milled.len(), 4);
            assert_eq!(
                choice
                    .milled
                    .iter()
                    .map(|r| r.card)
                    .collect::<std::collections::BTreeSet<_>>(),
                [one, spell, two, creature].into_iter().collect()
            );
            assert_eq!(
                choice
                    .candidates
                    .iter()
                    .map(|r| r.card)
                    .collect::<std::collections::BTreeSet<_>>(),
                [one, two].into_iter().collect()
            );
            assert_eq!((choice.minimum, choice.maximum), (0, 2));
            assert!(g.observe(p.other()).milled_choice.is_none());
            for id in [old_land, spell, creature] {
                let object = g.object_ref(id);
                reject(&mut g, Action::SelectMilled { object });
            }

            for id in [one, two].into_iter().take(count) {
                let object = g.object_ref(id);
                act(&mut g, Action::SelectMilled { object });
                reject(&mut g, Action::SelectMilled { object });
            }
            if g.legal_actions().contains(&Action::FinishMilledSelection) {
                act(&mut g, Action::FinishMilledSelection);
            }
            settle(&mut g);
            for (index, id) in [one, two].into_iter().enumerate() {
                assert_eq!(
                    g.object_view(id).zone,
                    if index < count {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            for id in [old_land, spell, creature] {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
            assert!(g.can_play_adventure(p, thranduil));
            let mut mana = colors;
            mana[5] = 2;
            paid(&mut g, thranduil, false, None, mana);
            settle(&mut g);
            assert_eq!(stats(&g, thranduil), (2, 3));
            assert_eq!(g.object_view(thranduil).zone, Zone::Battlefield);
            assert!(!g.can_play_adventure(p, thranduil));
        }
    }
}
