//! IRC mode parsing.

use crate::error::MessageParseError;

use super::types::{ChannelMode, Mode, ModeType, UserMode};

enum PlusMinus {
    Plus,
    Minus,
    NoPrefix,
}

impl Mode<UserMode> {
    /// Parse user mode strings like `+iw` into a vector of modes.
    pub fn as_user_modes(pieces: &[&str]) -> Result<Vec<Mode<UserMode>>, MessageParseError> {
        parse_modes(pieces)
    }
}

impl Mode<ChannelMode> {
    /// Parse channel mode strings like `+o nick` into a vector of modes.
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
                                // List modes (Type A) can be queried without an argument
                                // e.g., MODE #channel +b queries the ban list
                                if mode.is_list_mode() {
                                    None
                                } else {
                                    return Err(MessageParseError::InvalidModeArg(format!(
                                        "Mode '{}' requires an argument but none provided",
                                        c
                                    )));
                                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ban_list_query_no_arg() {
        // MODE #channel +b (query ban list) - no mask argument
        let modes = Mode::<ChannelMode>::as_channel_modes(&["+b"]).unwrap();
        assert_eq!(modes.len(), 1);
        assert_eq!(modes[0], Mode::Plus(ChannelMode::Ban, None));
    }

    #[test]
    fn test_ban_with_mask() {
        // MODE #channel +b *!*@example.com (add ban)
        let modes = Mode::<ChannelMode>::as_channel_modes(&["+b", "*!*@example.com"]).unwrap();
        assert_eq!(modes.len(), 1);
        assert_eq!(
            modes[0],
            Mode::Plus(ChannelMode::Ban, Some("*!*@example.com".to_string()))
        );
    }

    #[test]
    fn test_exception_list_query() {
        // MODE #channel +e (query exception list)
        let modes = Mode::<ChannelMode>::as_channel_modes(&["+e"]).unwrap();
        assert_eq!(modes.len(), 1);
        assert_eq!(modes[0], Mode::Plus(ChannelMode::Exception, None));
    }

    #[test]
    fn test_invite_exception_list_query() {
        // MODE #channel +I (query invite exception list)
        let modes = Mode::<ChannelMode>::as_channel_modes(&["+I"]).unwrap();
        assert_eq!(modes.len(), 1);
        assert_eq!(modes[0], Mode::Plus(ChannelMode::InviteException, None));
    }

    #[test]
    fn test_quiet_list_query() {
        // MODE #channel +q (query quiet list)
        let modes = Mode::<ChannelMode>::as_channel_modes(&["+q"]).unwrap();
        assert_eq!(modes.len(), 1);
        assert_eq!(modes[0], Mode::Plus(ChannelMode::Quiet, None));
    }

    #[test]
    fn test_non_list_mode_requires_arg() {
        // MODE #channel +k (key mode requires argument)
        let result = Mode::<ChannelMode>::as_channel_modes(&["+k"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_limit_mode_requires_arg() {
        // MODE #channel +l (limit mode requires argument)
        let result = Mode::<ChannelMode>::as_channel_modes(&["+l"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_op_mode_requires_arg() {
        // MODE #channel +o (op mode requires argument)
        let result = Mode::<ChannelMode>::as_channel_modes(&["+o"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_mixed_list_and_regular_modes() {
        // MODE #channel +bi nick (invite-only + ban query)
        let modes = Mode::<ChannelMode>::as_channel_modes(&["+ib"]).unwrap();
        assert_eq!(modes.len(), 2);
        assert_eq!(modes[0], Mode::Plus(ChannelMode::InviteOnly, None));
        assert_eq!(modes[1], Mode::Plus(ChannelMode::Ban, None));
    }

    #[test]
    fn test_minus_ban_still_needs_no_arg_for_query() {
        // MODE #channel -b (can also query, though unusual)
        let modes = Mode::<ChannelMode>::as_channel_modes(&["-b"]).unwrap();
        assert_eq!(modes.len(), 1);
        assert_eq!(modes[0], Mode::Minus(ChannelMode::Ban, None));
    }
}
