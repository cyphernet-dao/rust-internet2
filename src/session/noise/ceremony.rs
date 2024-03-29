// LNP/BP Core Library implementing LNPBP specifications & standards
// Written in 2020 by
//     Julian Knutsen
//     Rajarshi Maitra
//     Arik Sosman
//     Matt Corallo
//     Antoine Riard
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::{cmp, ops};

pub const ACT_ONE_LENGTH: usize = 50;
pub const ACT_TWO_LENGTH: usize = 50;
pub const ACT_THREE_LENGTH: usize = 66;
pub const EMPTY_ACT_ONE: ActOne = [0; ACT_ONE_LENGTH];
pub const EMPTY_ACT_TWO: ActTwo = [0; ACT_TWO_LENGTH];
pub const EMPTY_ACT_THREE: ActThree = [0; ACT_THREE_LENGTH];
type ActOne = [u8; ACT_ONE_LENGTH];
type ActTwo = [u8; ACT_TWO_LENGTH];
type ActThree = [u8; ACT_THREE_LENGTH];

/// Wrapper for any act message
#[derive(Copy, Clone, Debug)]
pub enum Act {
    One(ActOne),
    Two(ActTwo),
    Three(ActThree),
}

impl Act {
    /// Returns the size of the underlying array
    fn len(&self) -> usize { self.as_ref().len() }
}

impl From<ActBuilder> for Act {
    /// Convert a finished ActBuilder into an Act
    fn from(act_builder: ActBuilder) -> Self {
        assert!(act_builder.is_finished());
        act_builder.partial_act
    }
}

impl ops::Deref for Act {
    type Target = [u8];

    /// Allows automatic coercion to slices in function calls
    /// &Act -> &[u8]
    fn deref(&self) -> &Self::Target {
        match self {
            Act::One(ref act) => act,
            Act::Two(ref act) => act,
            Act::Three(ref act) => act,
        }
    }
}

impl AsRef<[u8]> for Act {
    /// Allow convenient exposure of the underlying array through as_ref()
    /// Act.as_ref() -> &[u8]
    fn as_ref(&self) -> &[u8] { self }
}

/// Light wrapper around an Act that allows multiple fill() calls before finally
/// converting to an Act via Act::from(act_builder). Handles all of the
/// bookkeeping and edge cases of the array fill
#[derive(Debug)]
pub struct ActBuilder {
    partial_act: Act,
    write_pos: usize,
}

impl ActBuilder {
    /// Returns a new ActBuilder for Act::One
    pub fn new(empty_act: Act) -> Self {
        Self {
            partial_act: empty_act,
            write_pos: 0,
        }
    }

    /// Fills the Act with bytes from input and returns the number of bytes
    /// consumed from input.
    pub fn fill(&mut self, input: &[u8]) -> usize {
        // Simple fill implementation for both almost-identical structs to
        // deduplicate code $act: Act[One|Two|Three], $input: &[u8];
        // returns &[u8] of remaining input that was not processed
        macro_rules! fill_act_content {
            ($act:expr, $write_pos:expr, $input:expr) => {{
                let fill_amount =
                    cmp::min($act.len() - $write_pos, $input.len());

                $act[$write_pos..$write_pos + fill_amount]
                    .copy_from_slice(&$input[..fill_amount]);

                $write_pos += fill_amount;
                fill_amount
            }};
        }

        match self.partial_act {
            Act::One(ref mut act) => {
                fill_act_content!(act, self.write_pos, input)
            }
            Act::Two(ref mut act) => {
                fill_act_content!(act, self.write_pos, input)
            }
            Act::Three(ref mut act) => {
                fill_act_content!(act, self.write_pos, input)
            }
        }
    }

    /// Returns true if the Act is finished building (enough bytes via fill())
    pub fn is_finished(&self) -> bool {
        self.write_pos == self.partial_act.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test bookkeeping of partial fill
    #[test]
    fn partial_fill() {
        let mut builder = ActBuilder::new(Act::One(EMPTY_ACT_ONE));

        let input = [1, 2, 3];
        let bytes_read = builder.fill(&input);
        assert_eq!(builder.partial_act.len(), ACT_ONE_LENGTH);
        assert_eq!(builder.write_pos, 3);
        assert!(!builder.is_finished());
        assert_eq!(bytes_read, input.len());
    }

    // Test bookkeeping of exact fill
    #[test]
    fn exact_fill() {
        let mut builder = ActBuilder::new(Act::One(EMPTY_ACT_ONE));

        let input = [0; ACT_ONE_LENGTH];
        let bytes_read = builder.fill(&input);
        assert_eq!(builder.partial_act.len(), ACT_ONE_LENGTH);
        assert_eq!(builder.write_pos, ACT_ONE_LENGTH);
        assert!(builder.is_finished());
        assert_eq!(Act::from(builder).as_ref(), &input[..]);
        assert_eq!(bytes_read, input.len());
    }

    // Test bookkeeping of overfill
    #[test]
    fn over_fill() {
        let mut builder = ActBuilder::new(Act::One(EMPTY_ACT_ONE));

        let input = [0; ACT_ONE_LENGTH + 1];
        let bytes_read = builder.fill(&input);

        assert_eq!(builder.partial_act.len(), ACT_ONE_LENGTH);
        assert_eq!(builder.write_pos, ACT_ONE_LENGTH);
        assert!(builder.is_finished());
        assert_eq!(Act::from(builder).as_ref(), &input[..ACT_ONE_LENGTH]);
        assert_eq!(bytes_read, ACT_ONE_LENGTH);
    }

    // Converting an unfinished ActBuilder panics
    #[test]
    #[should_panic(expected = "assertion failed: act_builder.is_finished()")]
    fn convert_not_finished_panics() {
        let builder = ActBuilder::new(Act::One(EMPTY_ACT_ONE));
        let _should_panic = Act::from(builder);
    }
}
