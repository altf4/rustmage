//! Cross-mechanic coverage for the still-gated behold and basic-land rules.
use super::*;

#[test]
fn countersculpt_behold_and_extra_mana_pay_thalia_and_discounts_in_normal_and_free_casts() {
    for actor in [Player::P0, Player::P1] {
        for free in [false, true] {
            for discount in [false, true] {
                for route in [0u16, 1] {
                    let mut game = scenario(actor.other());
                    put(
                        &mut game,
                        actor.other(),
                        "Thalia, the Survivor",
                        Zone::Battlefield,
                    );
                    if discount {
                        put(&mut game, actor, "Geist of Saint Thalia", Zone::Battlefield);
                    }
                    if free {
                        put(&mut game, actor, "Omniscience", Zone::Battlefield);
                    }
                    let witness = put(&mut game, actor, "Jace, Reality Sculptor", Zone::Hand);
                    let target = put(&mut game, actor.other(), "Bear Cub", Zone::Hand);
                    let spell = put(&mut game, actor, "Countersculpt", Zone::Hand);
                    cast(&mut game, target, None);
                    game.state.priority = actor;
                    let generic = u32::from(!discount) + u32::from(route);
                    let mut pool = [0; 6];
                    pool[1] = if free { 0 } else { 2 };
                    pool[5] = u32::from(!discount) + 1;
                    game.state.players[actor.index()].mana = pool;
                    let announcement = if free {
                        Action::AnnounceFree {
                            card: spell,
                            face: CastFace::Normal,
                            flash_surcharge: false,
                        }
                    } else {
                        Action::AnnounceCast { card: spell }
                    };
                    assert!(
                        game.legal_actions().contains(&announcement),
                        "free={free} discount={discount} route={route}"
                    );
                    act(&mut game, announcement);
                    act(&mut game, Action::ChooseSpellCost { cost: route });
                    act(
                        &mut game,
                        Action::ChooseTarget {
                            target: Target::Spell(target),
                        },
                    );
                    if route == 0 {
                        let object = game.object_ref(witness);
                        act(&mut game, Action::ChooseAdditionalCost { object });
                    }
                    if game.legal_actions().contains(&Action::FinishTargets) {
                        act(&mut game, Action::FinishTargets);
                    }
                    let total = game.state.announcing.as_ref().unwrap().total_cost.unwrap();
                    assert_eq!(
                        total.generic, generic,
                        "free={free} discount={discount} route={route}"
                    );
                    assert_eq!(total.colored[1], if free { 0 } else { 2 });
                    let before = game.snapshot();
                    let action = game
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayCosts { .. }))
                        .unwrap();
                    let revision = game.revision();
                    let index = game
                        .legal_actions()
                        .iter()
                        .position(|a| *a == action)
                        .unwrap();
                    let mut indexed = game.clone();
                    indexed.apply_index(actor, revision, index).unwrap();
                    game.apply(actor, action.clone()).unwrap();
                    let paid = game.snapshot();
                    assert_eq!(indexed.snapshot(), paid);
                    let mut remaining = [0; 6];
                    remaining[5] = 1 - u32::from(route);
                    assert_eq!(game.state.players[actor.index()].mana, remaining);
                    if route == 0 {
                        assert!(
                            game.observe(actor.other())
                                .known_hand
                                .contains(&game.object_ref(witness))
                        );
                    }
                    game.restore(&before).unwrap();
                    game.apply(actor, action).unwrap();
                    assert_eq!(game.snapshot(), paid);
                    settle(&mut game);
                    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
                    assert_eq!(game.object_view(witness).zone, Zone::Hand);
                }
            }
        }
    }
}
