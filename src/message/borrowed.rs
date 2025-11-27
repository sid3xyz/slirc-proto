
use crate::command::CommandRef;
use crate::error::MessageParseError;
use crate::prefix::PrefixRef;

use super::nom_parser::ParsedMessage;

#[derive(Clone, PartialEq, Debug)]
pub struct MessageRef<'a> {
    pub tags: Option<&'a str>,
    pub prefix: Option<PrefixRef<'a>>,
    pub command: CommandRef<'a>,
    pub raw: &'a str,
}

impl<'a> MessageRef<'a> {
    pub fn parse(s: &'a str) -> Result<MessageRef<'a>, MessageParseError> {
        if s.is_empty() {
            return Err(MessageParseError::EmptyMessage);
        }

        let trimmed = s.trim_end_matches(['\r', '\n']);

        let parsed = match ParsedMessage::parse(trimmed) {
            Ok(m) => m,
            Err(_e) => return Err(MessageParseError::InvalidCommand),
        };

        let prefix = parsed.prefix.map(PrefixRef::parse);
        let command = CommandRef::new(parsed.command, parsed.params.clone());

        Ok(MessageRef {
            tags: parsed.tags,
            prefix,
            command,
            raw: s,
        })
    }

    pub fn to_raw_owned(&self) -> String {
        let mut s = String::new();
        if let Some(tags) = &self.tags {
            s.push('@');
            s.push_str(tags);
            s.push(' ');
        }
        if let Some(prefix) = &self.prefix {
            s.push(':');
            s.push_str(prefix.raw);
            s.push(' ');
        }
        s.push_str(self.command.name);
        if !self.command.args.is_empty() {
            s.push(' ');
            s.push_str(&self.command.args.join(" "));
        }
        s
    }
}
