//! Actual Brazen Bluff targets an opponent's noncreature spell and branches on their choice.
mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

fn start_opposing_spell(game: &mut Game, caster: Player) -> CardId {
    act(game, Action::Pass);
    let opposing = put(game, caster.other(), "Opt", Zone::Hand);
    cast(game, opposing, None);
    act(game, Action::Pass);
    assert_eq!(game.actor(), Some(caster));
    opposing
}

fn reach_opponent_choice(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert!(matches!(game.state.pending, Pending::EffectOptional { .. }));
}

#[test]
fn actual_card_exiles_on_accept_or_offers_private_immediate_noncreature_cast_on_decline() {
    for caster in [Player::P0, Player::P1] {
        let mut game = scenario(caster);
        let opposing = start_opposing_spell(&mut game, caster);
        let creature = put(&mut game, caster, "Bear Cub", Zone::Hand);
        let sorcery = put(&mut game, caster, "Divination", Zone::Hand);
        let bluff = put(&mut game, caster, "Brazen Bluff", Zone::Hand);
        cast(&mut game, bluff, Some(Target::Spell(opposing)));
        reach_opponent_choice(&mut game);
        assert_eq!(game.actor(), Some(caster.other()));

        let mut accepted = game.clone();
        act(&mut accepted, Action::ChooseOptional { accept: true });
        assert_eq!(accepted.object_view(opposing).zone, Zone::Exile);
        assert!(accepted.observe(caster).resolution_free_casts.is_none());
        assert_eq!(accepted.object_view(bluff).zone, Zone::Graveyard);

        act(&mut game, Action::ChooseOptional { accept: false });
        assert_eq!(game.object_view(opposing).zone, Zone::Stack);
        assert_eq!(
            game.state.pending,
            Pending::ResolutionCasts { player: caster }
        );
        assert!(game.observe(caster).resolution_free_casts.is_some());
        assert!(game.observe(caster.other()).resolution_free_casts.is_none());
        let creature_ref = game.object_ref(creature);
        reject(
            &mut game,
            Action::CastFreeDuringResolution {
                object: creature_ref,
                face: CastFace::Normal,
            },
        );
        let before_mana = game.state.players[caster.index()].mana;
        let sorcery_ref = game.object_ref(sorcery);
        act(
            &mut game,
            Action::CastFreeDuringResolution {
                object: sorcery_ref,
                face: CastFace::Normal,
            },
        );
        pay(&mut game);
        assert_eq!(game.state.players[caster.index()].mana, before_mana);
        act(&mut game, Action::FinishResolutionCasts);
        assert_eq!(game.object_view(opposing).zone, Zone::Stack);
        // Stop after the free spell resolves; the older opposing Opt can then
        // begin its own separate library choice.
        for _ in 0..4 {
            if game.object_view(sorcery).zone == Zone::Graveyard {
                break;
            }
            act(&mut game, Action::Pass);
        }
        assert_eq!(game.object_view(sorcery).zone, Zone::Graveyard);
        assert_eq!(game.object_view(bluff).zone, Zone::Graveyard);
    }
}

#[test]
fn actual_card_fizzles_if_its_only_target_stops_being_opponent_controlled() {
    let caster = Player::P0;
    let mut game = scenario(caster);
    let opposing = start_opposing_spell(&mut game, caster);
    let bluff = put(&mut game, caster, "Brazen Bluff", Zone::Hand);
    cast(&mut game, bluff, Some(Target::Spell(opposing)));
    let target = game
        .state
        .stack
        .iter_mut()
        .find(|item| item.source.is_some_and(|source| source.id == opposing))
        .unwrap();
    target.controller = caster;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(bluff).zone, Zone::Graveyard);
    assert_eq!(game.object_view(opposing).zone, Zone::Stack);
    assert!(game.observe(caster).resolution_free_casts.is_none());
}
