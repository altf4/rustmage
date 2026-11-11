//! Targeted quotas round up and freeze before a simultaneous sacrifice or discard.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const OPPONENT: TargetSpec = TargetSpec::one(TargetRule::Player {
    opponent_only: true,
});
const ALL: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[OPPONENT, OPPONENT, OPPONENT]),
        &[
            Instruction::ChooseHalfRoundedUp {
                players: PlayerSet::Targeted,
                kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter { kinds: 1, ..ALL },
            },
            Instruction::SelectGroup(1),
            Instruction::ChooseHalfRoundedUp {
                players: PlayerSet::Targeted,
                kind: EffectChoiceKind::Discard,
                filter: ALL,
            },
            Instruction::SelectGroup(2),
            Instruction::LoseHalfLifeRoundedUp(PlayerSet::Targeted),
        ],
    ),
    ..CardRules::EMPTY
};
const IMMUNE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::CANT_BE_SACRIFICED,
    ..CardRules::EMPTY
};

#[test]
fn targeted_half_choices_round_up_use_only_opponent_and_preserve_hidden_selections() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Divination").unwrap(), &SPELL),
        (Card::HillGiant, &IMMUNE),
    ]);
    for (available, prohibited, hand) in [(5usize, 0, 5usize), (2, 3, 1), (0, 0, 0)] {
        let player = Player::P1;
        let opponent = player.other();
        let mut g = scenario(player);
        let own = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let land = put(&mut g, opponent, "Forest", Zone::Battlefield);
        let creatures: Vec<_> = (0..available)
            .map(|_| put(&mut g, opponent, "Bear Cub", Zone::Battlefield))
            .collect();
        for _ in 0..prohibited {
            put(&mut g, opponent, "Hill Giant", Zone::Battlefield);
        }
        let cards: Vec<_> = (0..hand)
            .map(|_| put(&mut g, opponent, "Goblin Piker", Zone::Hand))
            .collect();
        let spell = put(&mut g, player, "Divination", Zone::Hand);
        g.state.players[opponent.index()].life = 21;
        announce(&mut g, spell, &[Target::Player(opponent); 3]);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        let required = (available + prohibited).div_ceil(2).min(available);
        if required != 0 {
            assert_eq!(g.state.pending, Pending::EffectChoice { player: opponent });
            for excluded in [g.object_ref(own), g.object_ref(land)] {
                reject(&mut g, Action::Sacrifice { object: excluded });
            }
        }
        for (index, &card) in creatures.iter().take(required).enumerate() {
            let object = g.object_ref(card);
            act(&mut g, Action::Sacrifice { object });
            if index + 1 < required {
                assert_eq!(g.state.objects[card as usize].zone, Zone::Battlefield);
            }
        }
        let discard_count = hand.div_ceil(2);
        for (index, &card) in cards.iter().take(discard_count).enumerate() {
            assert_eq!(g.state.pending, Pending::EffectChoice { player: opponent });
            assert!(g.observe(player).effect_choice.is_none());
            let action = Action::Discard { card };
            let mut indexed = g.clone();
            let action_index = indexed
                .legal_actions()
                .iter()
                .position(|a| *a == action)
                .unwrap();
            indexed
                .apply_index(opponent, indexed.revision(), action_index)
                .unwrap();
            act(&mut g, action);
            assert_eq!(g.snapshot(), indexed.snapshot());
            if index + 1 < discard_count {
                assert_eq!(g.state.objects[card as usize].zone, Zone::Hand);
            }
        }
        settle(&mut g);
        assert_eq!(g.state.players[opponent.index()].life, 10);
        assert_eq!(g.state.players[player.index()].life, 20);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Battlefield);
        assert_eq!(
            creatures
                .iter()
                .filter(|&&id| g.state.objects[id as usize].zone == Zone::Graveyard)
                .count(),
            required
        );
        assert_eq!(
            cards
                .iter()
                .filter(|&&id| g.state.objects[id as usize].zone == Zone::Graveyard)
                .count(),
            discard_count
        );
    }
}
