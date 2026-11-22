//! Opponent-owned stack choice and the immediate private free-cast continuation.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};

const BLUFF: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::SpellKindsControlled {
            kinds: 0,
            excluded: 1,
            relation: 1,
        }),
        &[
            Instruction::MayTargetSpellControllerExile,
            Instruction::CastFilteredHandFree(CastFilter::Noncreature),
        ],
    ),
    ..CardRules::EMPTY
};

fn start_opponent_opt(game: &mut Game, caster: Player) -> CardId {
    act(game, Action::Pass);
    let opt = put(game, caster.other(), "Opt", Zone::Hand);
    cast(game, opt, None);
    act(game, Action::Pass);
    assert_eq!(game.actor(), Some(caster));
    opt
}

fn reach_choice(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert!(matches!(game.state.pending, Pending::EffectOptional { .. }));
}

#[test]
fn decline_offers_only_a_private_noncreature_cast_during_resolution() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &BLUFF)]);
    for caster in [Player::P0, Player::P1] {
        let mut game = scenario(caster);
        let opposing = start_opponent_opt(&mut game, caster);
        let creature = put(&mut game, caster, "Bear Cub", Zone::Hand);
        let sorcery = put(&mut game, caster, "Divination", Zone::Hand);
        let bluff = put(&mut game, caster, "Shock", Zone::Hand);
        cast(&mut game, bluff, Some(Target::Spell(opposing)));
        reach_choice(&mut game);
        assert_eq!(game.actor(), Some(caster.other()));

        let mut accepted = game.clone();
        act(&mut accepted, Action::ChooseOptional { accept: true });
        assert_eq!(accepted.object_view(opposing).zone, Zone::Exile);
        assert!(accepted.observe(caster).resolution_free_casts.is_none());
        assert!(accepted.observe(caster).plotted_cards.is_empty());

        let root = game.snapshot();
        let action = Action::ChooseOptional { accept: false };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut indexed = game.clone();
        indexed
            .apply_index(caster.other(), game.revision(), index)
            .unwrap();
        act(&mut game, action.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&root).unwrap();
        assert_eq!(indexed.snapshot(), root);
        act(&mut indexed, action);
        assert_eq!(game.snapshot(), indexed.snapshot());

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
        let mana = game.state.players[caster.index()].mana;
        let sorcery_ref = game.object_ref(sorcery);
        act(
            &mut game,
            Action::CastFreeDuringResolution {
                object: sorcery_ref,
                face: CastFace::Normal,
            },
        );
        pay(&mut game);
        assert_eq!(game.state.players[caster.index()].mana, mana);
        act(&mut game, Action::FinishResolutionCasts);
        assert_eq!(game.object_view(opposing).zone, Zone::Stack);
        for _ in 0..4 {
            if game.object_view(sorcery).zone == Zone::Graveyard {
                break;
            }
            act(&mut game, Action::Pass);
        }
        assert_eq!(game.object_view(sorcery).zone, Zone::Graveyard);
    }
}

#[test]
fn accepting_exiles_exact_spell_copy_without_plot_or_hand_offer() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &BLUFF)]);
    let caster = Player::P0;
    let mut game = scenario(caster);
    let opposing = start_opponent_opt(&mut game, caster);
    let template = game.state.stack.last().unwrap().clone();
    let own_copy = game.allocate_spell_copy(&template, caster);
    let own_copy_id = own_copy.id;
    game.state.stack.push(own_copy);
    let opposing_copy = game.allocate_spell_copy(&template, caster.other());
    let copy_id = opposing_copy.id;
    let copy_source = opposing_copy.source.unwrap().id;
    game.state.stack.push(opposing_copy);
    let bluff = put(&mut game, caster, "Shock", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: bluff });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Stack(own_copy_id),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Stack(copy_id),
        },
    );
    pay(&mut game);
    reach_choice(&mut game);
    assert_eq!(game.actor(), Some(caster.other()));
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    assert_eq!(
        decline.state.pending,
        Pending::ResolutionCasts { player: caster }
    );
    assert_eq!(decline.object_view(copy_source).zone, Zone::Stack);

    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(game.observe(caster).resolution_free_casts.is_none());
    // A spell copy is exiled as chosen, then ceases to exist as a token.
    assert_eq!(game.object_view(copy_source).zone, Zone::Outside);
    assert_eq!(game.object_view(opposing).zone, Zone::Stack);
    assert!(game.observe(caster).plotted_cards.is_empty());
    assert!(!game.state.stack.iter().any(|item| item.id == copy_id));
}

#[test]
fn chosen_adventure_face_is_a_noncreature_spell_but_creature_face_is_not() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &BLUFF)]);
    let caster = Player::P0;
    let mut game = scenario(caster);
    act(&mut game, Action::Pass);
    let adventure = put(&mut game, caster.other(), "Questing Druid", Zone::Hand);
    act(&mut game, Action::AnnounceAdventure { card: adventure });
    pay(&mut game);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(caster));
    let bluff = put(&mut game, caster, "Shock", Zone::Hand);
    cast(&mut game, bluff, Some(Target::Spell(adventure)));
    reach_choice(&mut game);
    assert_eq!(game.actor(), Some(caster.other()));
}

#[test]
fn losing_the_opponent_or_noncreature_qualification_fizzles_the_whole_spell() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &BLUFF)]);
    let caster = Player::P0;
    let mut game = scenario(caster);
    let opposing = start_opponent_opt(&mut game, caster);
    let bluff = put(&mut game, caster, "Shock", Zone::Hand);
    cast(&mut game, bluff, Some(Target::Spell(opposing)));
    for change_controller in [true, false] {
        let mut branch = game.clone();
        let target = branch
            .state
            .stack
            .iter_mut()
            .find(|item| item.source.is_some_and(|source| source.id == opposing))
            .unwrap();
        if change_controller {
            target.controller = caster;
        } else {
            target.source.as_mut().unwrap().card = Card::GrizzlyBears;
        }
        act(&mut branch, Action::Pass);
        act(&mut branch, Action::Pass);
        assert_eq!(branch.object_view(bluff).zone, Zone::Graveyard);
        assert!(branch.observe(caster).resolution_free_casts.is_none());
        assert_eq!(branch.state.pending, Pending::Priority);
    }
}
