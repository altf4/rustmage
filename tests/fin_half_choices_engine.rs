mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const HALF_CREATURES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ChooseHalf {
            kind: EffectChoiceKind::SacrificeCreaturesExceptSubtype("God"),
        }],
    ),
    ..CardRules::EMPTY
};

const UNSACRIFICABLE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::CANT_BE_SACRIFICED,
    ..CardRules::EMPTY
};

fn start_half_creature_choice(game: &mut Game, player: Player) {
    let spell = put(game, player, "Opt", Zone::Hand);
    cast(game, spell, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::EffectChoice { player });
}

#[test]
fn filtered_half_sacrifice_collects_each_players_quota_before_any_move() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &HALF_CREATURES)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let ours = (0..3)
        .map(|_| put(&mut game, player, "Bear Cub", Zone::Battlefield))
        .collect::<Vec<_>>();
    let theirs = (0..4)
        .map(|_| put(&mut game, player.other(), "Hill Giant", Zone::Battlefield))
        .collect::<Vec<_>>();
    let changeling = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    start_half_creature_choice(&mut game, player);

    let excluded = game.object_ref(changeling);
    reject(&mut game, Action::Sacrifice { object: excluded });
    let own_pick = game.object_ref(ours[0]);
    act(&mut game, Action::Sacrifice { object: own_pick });
    assert_eq!(game.actor(), Some(player.other()));
    assert_eq!(game.object_view(ours[0]).zone, Zone::Battlefield);
    let excluded = game.object_ref(land);
    reject(&mut game, Action::Sacrifice { object: excluded });

    let first = game.object_ref(theirs[0]);
    let second = game.object_ref(theirs[1]);
    act(&mut game, Action::Sacrifice { object: first });
    assert_eq!(game.object_view(theirs[0]).zone, Zone::Battlefield);
    act(&mut game, Action::Sacrifice { object: second });
    settle(&mut game);
    for id in [ours[0], theirs[0], theirs[1]] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    for id in [ours[1], ours[2], theirs[2], theirs[3], changeling, land] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    }
}

#[test]
fn filtered_half_sacrifice_finishes_when_legal_choices_are_below_the_quota() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &HALF_CREATURES),
        (Card::from_name("Hill Giant").unwrap(), &UNSACRIFICABLE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let available = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    for _ in 0..3 {
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
    }
    start_half_creature_choice(&mut game, player);
    let object = game.object_ref(available);
    let choices: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter(|action| matches!(action, Action::Sacrifice { .. }))
        .collect();
    assert_eq!(choices, vec![Action::Sacrifice { object }]);
    act(&mut game, Action::Sacrifice { object });
    settle(&mut game);
    assert_eq!(game.object_view(available).zone, Zone::Graveyard);
}
