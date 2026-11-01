//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn thalia_spell_tax_uses_opponent_and_noncreature_characteristics() {
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        let thalia = put(
            &mut game,
            actor.other(),
            "Thalia, the Survivor",
            Zone::Battlefield,
        );
        let shock = put(&mut game, actor, "Shock", Zone::Hand);
        let bear = put(&mut game, actor, "Bear Cub", Zone::Hand);
        let shock_card = game.effective_spell_card(shock);
        let bear_card = game.effective_spell_card(bear);
        assert_eq!(game.opponent_noncreature_spell_tax(actor, shock_card), 1);
        assert_eq!(
            game.opponent_noncreature_spell_tax(actor.other(), shock_card),
            0
        );
        assert_eq!(game.opponent_noncreature_spell_tax(actor, bear_card), 0);
        let second = put(
            &mut game,
            actor.other(),
            "Thalia, the Survivor",
            Zone::Battlefield,
        );
        assert_eq!(game.opponent_noncreature_spell_tax(actor, shock_card), 2);
        game.move_object(thalia, Zone::Exile);
        assert_eq!(game.opponent_noncreature_spell_tax(actor, shock_card), 1);
        game.move_object(second, Zone::Exile);
        assert_eq!(game.opponent_noncreature_spell_tax(actor, shock_card), 0);
    }
}

#[test]
fn thalia_tax_is_added_before_payment_width_is_checked() {
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        put(
            &mut game,
            actor.other(),
            "Thalia, the Survivor",
            Zone::Battlefield,
        );
        let shock = put(&mut game, actor, "Shock", Zone::Hand);
        let definition = game.effective_spell_card(shock);
        let base = ManaCost {
            colored: [0; 6],
            generic: 0,
        };
        assert_eq!(
            game.spell_total_for_definition(actor, shock, base, definition)
                .unwrap()
                .generic,
            1
        );
        let maximum = ManaCost {
            generic: u32::MAX,
            ..base
        };
        assert!(
            game.spell_total_for_definition(actor, shock, maximum, definition)
                .is_none()
        );
    }
}

#[test]
fn thalia_tax_tracks_control_and_ability_removal() {
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        let thalia = put(
            &mut game,
            actor.other(),
            "Thalia, the Survivor",
            Zone::Battlefield,
        );
        let shock = put(&mut game, actor, "Shock", Zone::Hand);
        let definition = game.effective_spell_card(shock);
        let reference = game.object_ref(thalia);
        assert_eq!(game.opponent_noncreature_spell_tax(actor, definition), 1);
        game.change_controller(reference, actor);
        assert_eq!(game.opponent_noncreature_spell_tax(actor, definition), 0);
        game.change_controller(reference, actor.other());
        let aura = put(&mut game, actor, "Fresh Start", Zone::Battlefield);
        game.state.objects[aura as usize].attached_to = Some(reference);
        assert_eq!(game.opponent_noncreature_spell_tax(actor, definition), 0);
        game.move_object(aura, Zone::Graveyard);
        assert_eq!(game.opponent_noncreature_spell_tax(actor, definition), 1);
    }
}

#[test]
fn thalia_tax_limits_x_and_is_paid_with_the_spell() {
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        put(
            &mut game,
            actor.other(),
            "Thalia, the Survivor",
            Zone::Battlefield,
        );
        let spell = put(&mut game, actor, "Mind Spring", Zone::Hand);
        game.state.players[actor.index()].mana = [0, 2, 0, 0, 0, 3];
        let drawn = game.state.players[actor.index()].cards_drawn_this_turn;
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(&mut game, Action::ChooseX { value: 3 });
        act(&mut game, Action::ChooseX { value: 2 });
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(
            game.state.players[actor.index()].cards_drawn_this_turn,
            drawn + 2
        );
        assert_eq!(game.state.players[actor.index()].mana, [0; 6]);
    }
}

#[test]
fn thalia_tax_can_be_offset_by_bargain_on_a_free_cast() {
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor.other());
        put(
            &mut game,
            actor.other(),
            "Thalia, the Survivor",
            Zone::Battlefield,
        );
        put(&mut game, actor, "Omniscience", Zone::Battlefield);
        let fodder = put(&mut game, actor, "Goldvein Pick", Zone::Battlefield);
        let enemy = put(&mut game, actor.other(), "Shock", Zone::Hand);
        cast(&mut game, enemy, Some(Target::Player(actor)));
        game.state.priority = actor;
        let ice = put(&mut game, actor, "Ice Out", Zone::Hand);
        game.state.players[actor.index()].mana = [0; 6];
        act(
            &mut game,
            Action::AnnounceFree {
                card: ice,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        let sacrifice = game.object_ref(fodder);
        act(
            &mut game,
            Action::ChooseBargain {
                sacrifice: Some(sacrifice),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Spell(enemy),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(game.object_view(fodder).zone, Zone::Graveyard);
        assert_eq!(game.state.players[actor.index()].mana, [0; 6]);
    }
}

#[test]
fn thalia_tax_applies_to_flashback() {
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        put(
            &mut game,
            actor.other(),
            "Thalia, the Survivor",
            Zone::Battlefield,
        );
        let spell = put(&mut game, actor, "Inspiration from Beyond", Zone::Graveyard);
        game.state.players[actor.index()].mana = [0, 2, 0, 0, 0, 5];
        reject(&mut game, Action::AnnounceFlashback { card: spell });
        game.state.players[actor.index()].mana[5] = 6;
        act(&mut game, Action::AnnounceFlashback { card: spell });
        pay(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Stack);
        assert_eq!(game.state.players[actor.index()].mana, [0; 6]);
    }
}
