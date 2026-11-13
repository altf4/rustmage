mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SPIRIT: Card = Card(5705);
const SOLDIER: Card = Card(5706);
const DRAGON: Card = Card(5707);
const BEAR: Card = Card(5708);
const MONK: Card = Card(5709);
const BOULDER: Card = Card(5710);

const fn create(token: Card) -> Instruction {
    Instruction::CreateTokens {
        token,
        amount: 1,
        tapped: false,
    }
}

const FACTORIES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            create(SPIRIT),
            create(SOLDIER),
            create(DRAGON),
            create(BEAR),
            create(MONK),
            create(BOULDER),
        ],
    ),
    ..CardRules::EMPTY
};

fn make_tokens(game: &mut Game, player: Player) {
    let spell = put(game, player, "Opt", Zone::Hand);
    cast(game, spell, None);
    settle(game);
    assert_eq!(game.controlled_token_ids(player).count(), 6);
}

fn token(game: &Game, player: Player, card: Card) -> CardId {
    game.controlled_token_ids(player)
        .find(|&id| game.object_view(id).card == card)
        .expect("actual token entry")
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        game.state.objects[card as usize].summoning_sick = false;
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn drain(game: &mut Game) {
    for _ in 0..32 {
        if game.state.stack.is_empty() && game.state.pending == Pending::Priority {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("token trigger did not resolve");
}

#[test]
fn avatar_factories_make_their_printed_tokens_and_firebending_mana() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &FACTORIES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    make_tokens(&mut game, player);
    for (card, name, subtype, power, toughness, colors, kinds, flying) in [
        (SPIRIT, "Spirit Token", "Spirit", 1, 1, 0, 1, false),
        (SOLDIER, "Soldier Token", "Soldier", 2, 2, 8, 1, false),
        (DRAGON, "Dragon Token", "Dragon", 4, 4, 8, 1, true),
        (BEAR, "Bear Token", "Bear", 4, 4, 16, 1, false),
        (MONK, "Monk Token", "Monk", 1, 1, 8, 1, false),
        (BOULDER, "Ballistic Boulder", "Construct", 2, 1, 0, 17, true),
    ] {
        let id = token(&game, player, card);
        let object = game.object_view(id);
        assert_eq!(object.name, name);
        assert_eq!(object.colors, colors, "{name}");
        assert_eq!(object.kinds, kinds, "{name}");
        assert_eq!(
            (object.power, object.toughness),
            (power, toughness),
            "{name}"
        );
        assert_eq!(object.keywords.contains(Keywords::FLYING), flying, "{name}");
        assert!(game.has_subtype(id, subtype), "{name}");
    }
    let monk = token(&game, player, MONK);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    drain(&mut game);
    assert_eq!(stats(&game, monk), (2, 2));

    let soldier = token(&game, player, SOLDIER);
    let dragon = token(&game, player, DRAGON);
    attack(&mut game, &[soldier, dragon]);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 5);
    assert_eq!(
        game.observe(player).turn_facts.unwrap()[player.index()].bent_elements,
        4
    );
    game.finish_cleanup();
    assert_eq!(stats(&game, monk), (1, 1));
}

#[test]
fn avatar_spirit_tokens_restrict_both_directions_of_blocking() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &FACTORIES)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    make_tokens(&mut game, player);
    let spirit = token(&game, player, SPIRIT);
    let bear = token(&game, player, BEAR);
    let giant = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let changeling = put(&mut game, opponent, "Mutable Explorer", Zone::Battlefield);

    let mut attacking = game.clone();
    attack(&mut attacking, &[spirit]);
    act(&mut attacking, Action::Pass);
    act(&mut attacking, Action::Pass);
    reject(
        &mut attacking,
        Action::Block {
            blocker: giant,
            attacker: spirit,
        },
    );
    act(
        &mut attacking,
        Action::Block {
            blocker: changeling,
            attacker: spirit,
        },
    );

    game.state.active = opponent;
    game.state.priority = opponent;
    attack(&mut game, &[giant, changeling]);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    reject(
        &mut game,
        Action::Block {
            blocker: spirit,
            attacker: giant,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: bear,
            attacker: giant,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: spirit,
            attacker: changeling,
        },
    );
}
