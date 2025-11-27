use crate::error::MessageParseError;

use super::types::{ChannelMode, Mode, ModeType, UserMode};

enum PlusMinus {
    Plus,
    Minus,
    NoPrefix,
}

impl Mode<UserMode> {
    pub fn as_user_modes(pieces: &[&str]) -> Result<Vec<Mode<UserMode>>, MessageParseError> {
        parse_modes(pieces)
    }
}

impl Mode<ChannelMode> {
    pub fn as_channel_modes(pieces: &[&str]) -> Result<Vec<Mode<ChannelMode>>, MessageParseError> {
        parse_modes(pieces)
    }
}

fn parse_modes<T>(pieces: &[&str]) -> Result<Vec<Mode<T>>, MessageParseError>
where
    T: ModeType,
{
    use self::PlusMinus::*;

    let mut res = vec![];

    if let Some((first, rest)) = pieces.split_first() {
        let mut modes = first.chars();
        let mut args = rest.iter().copied().peekable();

        let mut cur_mod = match modes.next() {
            Some('+') => Plus,
            Some('-') => Minus,
            Some(_) => {
                modes = first.chars();
                NoPrefix
            }
            None => {
                return Ok(res);
            }
        };

        for c in modes {
            match c {
                '+' => cur_mod = Plus,
                '-' => cur_mod = Minus,
                _ => {
                    let mode = T::from_char(c);
                    let arg = if mode.takes_arg() {
                        match args.next() {
                            Some(arg) => Some(arg.to_string()),
                            None => {
                                return Err(MessageParseError::InvalidModeArg(format!(
                                    "Mode '{}' requires an argument but none provided",
                                    c
                                )));
                            }
                        }
                    } else {
                        None
                    };
                    res.push(match cur_mod {
                        Plus => Mode::Plus(mode, arg),
                        Minus => Mode::Minus(mode, arg),
                        NoPrefix => Mode::NoPrefix(mode),
                    })
                }
            }
        }

        if args.peek().is_some() {
            return Err(MessageParseError::InvalidModeArg(
                "Unused arguments provided for mode parsing".to_string(),
            ));
        }
    }

    Ok(res)
}
