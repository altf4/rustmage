//! Immutable printed symbols, independently of payment alternatives or reductions.
use super::{Card, ChoiceColor};

pub(super) fn count(cost: &str, color: ChoiceColor) -> u64 {
    let wanted = [b'W', b'U', b'B', b'R', b'G'][color.index()];
    cost.split('{')
        .skip(1)
        .filter(|part| {
            part.split_once('}').is_some_and(|(symbol, _)| {
                symbol
                    .split('/')
                    .any(|component| component.as_bytes() == [wanted])
            })
        })
        .count() as u64
}
impl Card {
    /// Chosen immutable face mana cost, not the actual mana spent to cast it.
    pub(crate) fn mana_symbols_of_color(self, color: ChoiceColor) -> u64 {
        count(self.info().mana_cost, color)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn msh_namor_printed_colored_symbol_components_are_independent_of_payment() {
        for (cost, expected) in [
            ("{U}{U}", 2),
            ("{W/U}{2/U}{U/P}", 3),
            ("{W/U/P}", 1),
            ("{C/U}", 1),
            ("{X}{5}{C}{S}", 0),
            ("", 0),
        ] {
            assert_eq!(count(cost, ChoiceColor::Blue), expected);
        }
        assert_eq!(count("{W/U}{2/U}{U/P}", ChoiceColor::White), 1);
        assert_eq!(count("{R}{R}", ChoiceColor::Blue), 0);
    }
}
