//! A first cast can be classified independently of its observer's arrival.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FIRST_CASTS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FirstSpellKindOrOtherCreatureType {
            kinds: 4 | 8,
            subtype: "Otter",
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[
            Instruction::MayDraw { skip: 2 },
            Instruction::Draw(PlayerSet::Targeted, 1),
            Instruction::CopyCapturedSpell {
                count: SpellCopyCount::Fixed(1),
                retarget: true,
            },
        ],
    }],
    ..CardRules::EMPTY
};

fn begin_optional(game: &mut Game) {
    for _ in 0..12 {
        if matches!(game.state.pending, Pending::EffectOptional { .. }) {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("first-cast optional draw did not reach resolution");
}

fn target_opponent(game: &mut Game, controller: Player) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Player(controller.other()),
        },
    );
}

#[test]
fn a_prior_instant_counts_and_the_exact_spell_that_entered_as_the_source_is_excluded() {
    let observer = Card::from_name("Coruscation Mage").unwrap();
    let _rules = fixture_rules::activate(&[(observer, &FIRST_CASTS)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let first = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, first, Some(Target::Player(player.other())));
        settle(&mut game);

        let source = spawn(&mut game, player, observer, Zone::Hand);
        cast(&mut game, source, None);
        settle(&mut game);

        let second = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, second, Some(Target::Player(player.other())));
        settle(&mut game);

        let sorcery = put(&mut game, player, "Dragon Fodder", Zone::Hand);
        cast(&mut game, sorcery, None);
        target_opponent(&mut game, player);
        begin_optional(&mut game);
        act(&mut game, Action::ChooseOptional { accept: false });
        settle(&mut game);

        let mut blinked = game.clone();
        blinked.move_object(source, Zone::Exile);
        blinked.move_object(source, Zone::Battlefield);
        let other_otter = put(&mut blinked, player, "Alania's Pathmaker", Zone::Hand);
        cast(&mut blinked, other_otter, None);
        settle(&mut blinked);

        let other_otter = put(&mut game, player, "Alania's Pathmaker", Zone::Hand);
        cast(&mut game, other_otter, None);
        target_opponent(&mut game, player);
        begin_optional(&mut game);
        act(&mut game, Action::ChooseOptional { accept: false });
        settle(&mut game);

        let later_otter = put(&mut game, player, "Alania's Pathmaker", Zone::Hand);
        cast(&mut game, later_otter, None);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.other().index()].cards_drawn_this_turn,
            0
        );
    }
}

#[test]
fn taking_the_draw_option_copies_a_countered_first_spell_without_casting_it() {
    let observer = Card::from_name("Coruscation Mage").unwrap();
    let _rules = fixture_rules::activate(&[(observer, &FIRST_CASTS)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    spawn(&mut game, player, observer, Zone::Battlefield);
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(opponent)));
    target_opponent(&mut game, player);
    let negate = put(&mut game, opponent, "Negate", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, negate, Some(Target::Spell(spell)));
    begin_optional(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    act(&mut game, Action::ChooseOptional { accept: true });
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    settle(&mut game);
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        1
    );
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}
