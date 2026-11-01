//! Focused printed outcomes; shared engine invariants live in core tests.
//! Whole-card dragon mechanics, paid casts and owned continuations.
use super::*;
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    big_helpers::checked(g, action);
}
fn offer(g: &mut Game) {
    for _ in 0..32 {
        if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
            return;
        }
        big_helpers::checked(g, Action::Pass);
    }
    panic!("cascade did not offer a cast");
}
fn free_card(g: &mut Game, face: CastFace) -> ObjectRef {
    let object = g.state.resolution_free_casts.as_ref().unwrap().remaining[0];
    big_helpers::checked(g, Action::CastExiledDuringResolution { object, face });
    object
}
fn choose_extra(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    big_helpers::checked(g, Action::ChooseAdditionalCost { object });
}

#[test]
fn sos_dragon_quandrix_cast_cascade_public_sequence_decline_and_random_bottom() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let untouched = g.state.players[p.index()].library.clone();
        let hit = put(&mut g, p, "Bear Cub", Zone::Library);
        let expensive = put(&mut g, p, "Koma, World-Eater", Zone::Library);
        let land = put(&mut g, p, "Forest", Zone::Library);
        let dragon = put(&mut g, p, "Quandrix, the Proof", Zone::Hand);
        big_helpers::cast(&mut g, dragon, None, &[]);
        offer(&mut g);
        let refs = [
            g.object_ref(land),
            g.object_ref(expensive),
            g.object_ref(hit),
        ];
        let view = g.observe(p.other());
        let cascade = view.cascade.unwrap();
        assert_eq!(cascade.exiled, refs);
        assert_eq!(cascade.maximum_mana_value, 6);
        assert!(refs.iter().all(|r| {
            view.objects
                .iter()
                .any(|o| o.reference == *r && o.zone == Zone::Exile)
        }));
        assert_eq!(g.object_view(dragon).zone, Zone::Stack);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        assert!(g.observe(p).cascade.is_none());
        let library = &g.state.players[p.index()].library;
        assert_eq!(&library[3..], untouched.as_slice());
        let mut bottom = library[..3].to_vec();
        bottom.sort_unstable();
        let mut expected = vec![hit, expensive, land];
        expected.sort_unstable();
        assert_eq!(bottom, expected);
        big_helpers::settle(&mut g);
        assert_eq!(stats(&g, dragon), (6, 6));
        assert!(
            g.keywords(dragon)
                .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
        );
    }
}

#[test]
fn sos_dragon_cascade_survives_countered_source_and_casts_before_original() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let hit = put(&mut g, p, "Bear Cub", Zone::Library);
        let dragon = put(&mut g, p, "Quandrix, the Proof", Zone::Hand);
        big_helpers::cast(&mut g, dragon, None, &[]);
        let counter = put(&mut g, p, "Essence Scatter", Zone::Hand);
        big_helpers::cast(&mut g, counter, None, &[Target::Spell(dragon)]);
        offer(&mut g);
        assert_eq!(g.object_view(dragon).zone, Zone::Graveyard);
        assert_eq!(g.observe(p).cascade.unwrap().maximum_mana_value, 6);
        let before = g.state.players[p.index()].mana;
        let object = free_card(&mut g, CastFace::Normal);
        assert_eq!(object.card, hit);
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, before);
        assert_eq!(g.object_view(hit).zone, Zone::Stack);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(hit).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 3);
    }
}

#[test]
fn sos_dragon_hand_grant_frozen_x_and_strict_resulting_adventure_mana_value() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let dragon = put(&mut g, p, "Quandrix, the Proof", Zone::Battlefield);
        put(&mut g, p, "Unsummon", Zone::Graveyard);
        let hit = put(&mut g, p, "Aquatic Alchemist", Zone::Library);
        let spell = put(&mut g, p, "Fractalize", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(&mut g, Action::ChooseX { value: 2 });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(dragon),
            },
        );
        pay(&mut g);
        offer(&mut g);
        assert_eq!(g.observe(p).cascade.unwrap().maximum_mana_value, 3);
        let object = g.object_ref(hit);
        reject(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Adventure,
            },
        );
        free_card(&mut g, CastFace::Normal);
        pay(&mut g);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(hit).zone, Zone::Battlefield);
        assert_eq!(stats(&g, dragon), (3, 3));
    }
}

fn first_draw(g: &mut Game) {
    for _ in 0..32 {
        if matches!(g.state.pending, Pending::MiracleReveal { .. }) {
            return;
        }
        big_helpers::checked(g, Action::Pass);
    }
    panic!("no first-draw decision {:?}", g.state.pending);
}
fn lorehold(g: &mut Game, p: Player) -> CardId {
    let dragon = put(g, p, "Lorehold, the Historian", Zone::Hand);
    big_helpers::cast(g, dragon, None, &[]);
    big_helpers::settle(g);
    assert_eq!(stats(g, dragon), (5, 5));
    assert!(
        g.keywords(dragon)
            .contains(Keywords::FLYING.union(Keywords::HASTE))
    );
    dragon
}
fn draw_spell(g: &mut Game, p: Player, name: &str) {
    let spell = put(g, p, name, Zone::Hand);
    big_helpers::cast(g, spell, None, &[]);
    first_draw(g);
}
fn reveal_offer(g: &mut Game) {
    big_helpers::checked(g, Action::RevealMiracle);
    offer(g);
    let w = g.state.resolution_free_casts.as_ref().unwrap();
    assert_eq!(w.origin, Zone::Hand);
    assert_eq!(
        w.alternative_cost,
        Some(ManaCost {
            colored: [0; 6],
            generic: 2
        })
    );
}

#[test]
fn sos_dragon_lorehold_optional_opponent_upkeep_discard_miracle_sorcery() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        lorehold(&mut g, p);
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        assert!(g.state.stack.is_empty());
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        g.state.active = p.other();
        g.begin_step(Step::Upkeep);
        g.give_priority();
        for _ in 0..16 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        let mut decline = g.clone();
        big_helpers::checked(&mut decline, Action::DeclineEffectPayment);
        big_helpers::settle(&mut decline);
        assert_eq!(decline.state.players[p.index()].cards_drawn_this_turn, 0);
        let enemy = put(&mut g, p.other(), "Forest", Zone::Hand);
        let spell = put(&mut g, p, "Group Project", Zone::Library);
        reject(&mut g, Action::Discard { card: enemy });
        big_helpers::checked(&mut g, Action::Discard { card: discard });
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        reveal_offer(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        free_card(&mut g, CastFace::Normal);
        pay(&mut g);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
    }
}
#[test]
fn sos_dragon_miracle_mandatory_additional_cost_or_mana_alternative() {
    {
        let p = Player::P0;
        for mana in [false, true] {
            let mut g = scenario(p);
            lorehold(&mut g, p);
            let sacrifice = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Eaten Alive", Zone::Library);
            draw_spell(&mut g, p, "Divination");
            reveal_offer(&mut g);
            g.state.players[p.index()].mana = if mana {
                [0, 0, 1, 0, 0, 5]
            } else {
                [0, 0, 0, 0, 0, 2]
            };
            free_card(&mut g, CastFace::Normal);
            if mana {
                big_helpers::checked(&mut g, Action::ChooseSpellCost { cost: 1 });
            } else {
                big_helpers::checked(&mut g, Action::ChooseSpellCost { cost: 0 });
            }
            big_helpers::checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if !mana {
                choose_extra(&mut g, sacrifice);
            }
            pay(&mut g);
            assert_eq!(g.state.stack.last().unwrap().source.unwrap().id, spell);
            big_helpers::checked(&mut g, Action::FinishResolutionCasts);
            big_helpers::settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Exile);
            assert_eq!(
                g.object_view(sacrifice).zone,
                if mana {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

#[test]
fn sos_dragon_miracle_ability_loss_first_draw_only_and_observed_policy() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let dragon = lorehold(&mut g, p);
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        big_helpers::cast(&mut g, aura, None, &[Target::Card(dragon)]);
        big_helpers::settle(&mut g);
        put(&mut g, p, "Group Project", Zone::Library);
        let draw = put(&mut g, p, "Divination", Zone::Hand);
        big_helpers::cast(&mut g, draw, None, &[]);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert!(g.observe(p).miracle.is_none());

        let mut g = scenario(p);
        lorehold(&mut g, p);
        put(&mut g, p, "Group Project", Zone::Library);
        draw_spell(&mut g, p, "Divination");
        let actions = g.legal_actions();
        let selected = crate::policy::smoke_index(&g, &actions);
        assert_eq!(actions[selected], Action::RevealMiracle);
        big_helpers::checked(&mut g, Action::DeclineMiracle);
        big_helpers::settle(&mut g);
        let second = put(&mut g, p, "Group Project", Zone::Library);
        let draw = put(&mut g, p, "Divination", Zone::Hand);
        big_helpers::cast(&mut g, draw, None, &[]);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
        assert_eq!(g.object_view(second).zone, Zone::Hand);
        assert!(g.observe(p).miracle.is_none());
    }
}

fn two_lorehold_miracle_draw(p: Player) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    let original = put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
    let copy_spell = put(&mut g, p, "Multiversal Incursion", Zone::Hand);
    big_helpers::cast(&mut g, copy_spell, None, &[]);
    big_helpers::settle(&mut g);
    let second_draw = put(&mut g, p, "Forest", Zone::Library);
    let first = put(&mut g, p, "Group Project", Zone::Library);
    draw_spell(&mut g, p, "Divination");
    assert_eq!(g.observe(p).miracle.unwrap().object.card, first);
    (g, original, second_draw)
}

#[test]
fn multiple_miracle_triggers_survive_providers_and_offer_two_casts() {
    {
        let p = Player::P0;
        let (mut g, original, _) = two_lorehold_miracle_draw(p);
        let drawn = g.observe(p).miracle.unwrap().object;
        big_helpers::checked(&mut g, Action::RevealMiracle);
        big_helpers::checked(&mut g, Action::RevealMiracle);
        g.prepare_triggers();
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
        }
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|item| item
                    .attachment_trigger
                    .as_ref()
                    .is_some_and(|grant| grant.recipient == drawn))
                .count(),
            2
        );
        let providers: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                (object.zone == Zone::Battlefield
                    && object.card.name() == "Lorehold, the Historian")
                    .then_some(id as CardId)
            })
            .collect();
        assert_eq!(providers.len(), 2);
        assert!(providers.contains(&original));
        for provider in providers {
            g.state.players[p.index()].mana = [8; 6];
            let bounce = put(&mut g, p, "Unsummon", Zone::Hand);
            big_helpers::cast(&mut g, bounce, None, &[Target::Card(provider)]);
            for _ in 0..8 {
                if !g.on_battlefield(provider) {
                    break;
                }
                big_helpers::checked(&mut g, Action::Pass);
            }
            assert!(!g.on_battlefield(provider));
        }
        offer(&mut g);
        assert_eq!(
            g.state.resolution_free_casts.as_ref().unwrap().remaining[0],
            drawn
        );
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        offer(&mut g);
        assert_eq!(
            g.state.resolution_free_casts.as_ref().unwrap().remaining[0],
            drawn
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        assert_eq!(free_card(&mut g, CastFace::Normal), drawn);
        pay(&mut g);
        assert_eq!(g.object_view(drawn.card).zone, Zone::Stack);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
    }
}
