//! Immutable printed symbols, independently of payment alternatives or reductions.
use super::{Card, ChoiceColor};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn count(cost: &str, color: ChoiceColor) -> u64 {
    let wanted = b"WUBRG"[color.index()];
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Card {
    /// Chosen immutable face mana cost, not the actual mana spent to cast it.
    pub(crate) fn mana_symbols_of_color(self, color: ChoiceColor) -> u64 {
        count(self.info().mana_cost, color)
    }
}

/// Payment alternatives of printed mana symbols, without alternate spell costs.
/// This is mana notation metadata; no Oracle rules text is interpreted.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintedManaCost {
    pub options: Vec<super::ManaCost>,
    pub x_symbols: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) fn printed_cost(cost: &str) -> Option<PrintedManaCost> {
    use super::ManaCost;
    if cost.is_empty() {
        return None;
    }
    let mut result = PrintedManaCost {
        options: vec![ManaCost {
            colored: [0; 6],
            generic: 0,
        }],
        x_symbols: 0,
    };
    let mut input = cost;
    while !input.is_empty() {
        let (symbol, remaining) = input.strip_prefix('{')?.split_once('}')?;
        input = remaining;
        if symbol == "X" {
            result.x_symbols = result.x_symbols.checked_add(1)?;
            continue;
        }
        let mut alternatives = Vec::new();
        for part in symbol.split('/') {
            let mut mana = ManaCost {
                colored: [0; 6],
                generic: 0,
            };
            if let Some(color) = b"WUBRGC".iter().position(|&c| part.as_bytes() == [c]) {
                mana.colored[color] = 1
            } else {
                mana.generic = part.parse().ok()?
            }
            alternatives.push(mana);
        }
        let mut expanded = Vec::new();
        for base in result.options {
            for addition in &alternatives {
                let mut next = base;
                next.generic = next.generic.checked_add(addition.generic)?;
                for color in 0..6 {
                    next.colored[color] =
                        next.colored[color].checked_add(addition.colored[color])?
                }
                if !expanded.contains(&next) {
                    expanded.push(next)
                }
            }
        }
        result.options = expanded;
    }
    Some(result)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) enum ManaOptions {
    Static(super::ManaCostOptions),
    Printed(std::vec::IntoIter<super::ManaCost>),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Iterator for ManaOptions {
    type Item = super::ManaCost;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Static(costs) => costs.next(),
            Self::Printed(costs) => costs.next(),
        }
    }
}
