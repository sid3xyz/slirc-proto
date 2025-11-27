
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IsupportEntry<'a> {
    pub key: &'a str,
    pub value: Option<&'a str>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Isupport<'a> {
    entries: Vec<IsupportEntry<'a>>, 
}

impl<'a> Isupport<'a> {
    pub fn parse_params(params: &[&'a str]) -> Self {
        let mut entries = Vec::with_capacity(params.len());
        for &p in params {
            if p.starts_with(':') { break; }
            if p.is_empty() { continue; }
            let (k, v) = if let Some(eq) = p.find('=') {
                (&p[..eq], Some(&p[eq + 1..]))
            } else {
                (p, None)
            };

            entries.push(IsupportEntry { key: k, value: v });
        }
        Isupport { entries }
    }

    pub fn from_response_args(args: &[&'a str]) -> Option<Self> {
        if args.is_empty() {
            return None;
        }
        
        let mut tokens = &args[1..];
        
        if let Some(last) = tokens.last() {
            if last.contains(' ') { tokens = &tokens[..tokens.len().saturating_sub(1)]; }
        }
        Some(Self::parse_params(tokens))
    }

    pub fn from_message(msg: &'a crate::Message) -> Option<Self> {
        match &msg.command {
            crate::command::Command::Response(crate::response::Response::RPL_ISUPPORT, ref a) => {
                let borrowed: Vec<&'a str> = a.iter().map(|s| s.as_str()).collect();
                Self::from_response_args(&borrowed)
            }
            _ => None,
        }
    }

    pub fn from_message_ref(msg: &'a crate::MessageRef<'a>) -> Option<Self> {
        if let Ok(resp) = msg.command.name.parse::<crate::response::Response>() {
            if resp == crate::response::Response::RPL_ISUPPORT {
                let borrowed: Vec<&'a str> = msg.command.args.to_vec();
                return Self::from_response_args(&borrowed);
            }
        }
        None
    }

    pub fn iter(&self) -> impl Iterator<Item = &IsupportEntry<'a>> {
        self.entries.iter()
    }

    pub fn get(&self, key: &str) -> Option<Option<&'a str>> {
        self.entries
            .iter()
            .rfind(|e| e.key.eq_ignore_ascii_case(key))
            .map(|e| e.value)
    }



    pub fn casemapping(&self) -> Option<&'a str> { self.get("CASEMAPPING").flatten() }

    pub fn chantypes(&self) -> Option<&'a str> { self.get("CHANTYPES").flatten() }

    pub fn network(&self) -> Option<&'a str> { self.get("NETWORK").flatten() }

    pub fn prefix(&self) -> Option<PrefixSpec<'a>> {
        self.get("PREFIX").flatten().and_then(PrefixSpec::parse)
    }

    pub fn chanmodes(&self) -> Option<ChanModes<'a>> {
        self.get("CHANMODES").flatten().and_then(ChanModes::parse)
    }

    pub fn has_excepts(&self) -> bool { self.get("EXCEPTS").is_some() }

    pub fn excepts_mode(&self) -> Option<char> {
        self.get("EXCEPTS").flatten().and_then(|s| s.chars().next())
    }

    pub fn has_invex(&self) -> bool { self.get("INVEX").is_some() }

    pub fn invex_mode(&self) -> Option<char> {
        self.get("INVEX").flatten().and_then(|s| s.chars().next())
    }

    pub fn targmax(&self) -> Option<TargMax<'a>> {
        self.get("TARGMAX").flatten().and_then(TargMax::parse)
    }

    pub fn maxlist(&self) -> Option<MaxList> {
        self.get("MAXLIST").flatten().and_then(MaxList::parse)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrefixSpec<'a> {
    pub modes: &'a str,
    pub prefixes: &'a str,
}

impl<'a> PrefixSpec<'a> {
    pub fn parse(s: &'a str) -> Option<Self> {

        if let Some(open) = s.find('(') {
            if let Some(close) = s[open + 1..].find(')') {
                let close = open + 1 + close;
                let modes = &s[open + 1..close];
                let prefixes = &s[close + 1..];
                if !modes.is_empty() && !prefixes.is_empty() { return Some(PrefixSpec { modes, prefixes }); }
            }
        } else if !s.is_empty() {
            return Some(PrefixSpec { modes: "", prefixes: s });
        }
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChanModes<'a> {
    pub a: &'a str,
    pub b: &'a str,
    pub c: &'a str,
    pub d: &'a str,
}

impl<'a> ChanModes<'a> {
    pub fn parse(s: &'a str) -> Option<Self> {
        let mut parts = s.splitn(4, ',');
        let (a,b,c,d) = (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
        Some(ChanModes { a, b, c, d })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargMax<'a> {
    entries: Vec<(&'a str, Option<usize>)>,
}

impl<'a> TargMax<'a> {
    pub fn parse(s: &'a str) -> Option<Self> {
        if s.is_empty() { return Some(TargMax { entries: Vec::new() }); }
        let mut entries = Vec::new();
        for part in s.split(',') {
            if part.is_empty() { continue; }
            if let Some(colon) = part.find(':') {
                let (cmd, num) = (&part[..colon], &part[colon+1..]);
                let val = num.parse::<usize>().ok();
                if !cmd.is_empty() { entries.push((cmd, val)); }
            } else {
                
                entries.push((part, None));
            }
        }
        Some(TargMax { entries })
    }

    pub fn get(&self, cmd: &str) -> Option<Option<usize>> {
        self.entries.iter().find(|(k, _)| k.eq_ignore_ascii_case(cmd)).map(|(_, v)| *v)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'a str, Option<usize>)> + '_ {
        self.entries.iter().copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaxList {
    entries: Vec<(char, usize)>,
}

impl MaxList {
    pub fn parse(s: &str) -> Option<Self> {
        if s.is_empty() { return Some(MaxList { entries: Vec::new() }); }
        let mut entries: Vec<(char, usize)> = Vec::new();
        for part in s.split(',') {
            if part.is_empty() { continue; }
            let (modes, limit_str) = part.split_once(':')?;


            let limit: usize = match limit_str.parse() { Ok(n) => n, Err(_) => continue };
            for ch in modes.chars() {
                
                entries.retain(|(c, _)| *c != ch);
                entries.push((ch, limit));
            }
        }
        Some(MaxList { entries })
    }

    pub fn limit_for(&self, mode: char) -> Option<usize> {
        self.entries.iter().rev().find(|(c, _)| *c == mode).map(|(_, n)| *n)
    }

    pub fn iter(&self) -> impl Iterator<Item = (char, usize)> + '_ {
        self.entries.iter().copied()
    }
}







#[derive(Debug, Clone, Default)]
pub struct IsupportBuilder {
    tokens: Vec<String>,
}

impl IsupportBuilder {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn network(mut self, name: &str) -> Self {
        self.tokens.push(format!("NETWORK={}", name));
        self
    }

    pub fn chantypes(mut self, types: &str) -> Self {
        self.tokens.push(format!("CHANTYPES={}", types));
        self
    }

    pub fn chanmodes(mut self, modes: &str) -> Self {
        self.tokens.push(format!("CHANMODES={}", modes));
        self
    }

    pub fn prefix(mut self, symbols: &str, letters: &str) -> Self {
        self.tokens.push(format!("PREFIX=({}){}",letters, symbols));
        self
    }

    pub fn casemapping(mut self, mapping: &str) -> Self {
        self.tokens.push(format!("CASEMAPPING={}", mapping));
        self
    }

    pub fn max_channels(mut self, count: u32) -> Self {
        self.tokens.push(format!("MAXCHANNELS={}", count));
        self
    }

    pub fn max_nick_length(mut self, len: u32) -> Self {
        self.tokens.push(format!("NICKLEN={}", len));
        self
    }

    pub fn max_topic_length(mut self, len: u32) -> Self {
        self.tokens.push(format!("TOPICLEN={}", len));
        self
    }


    pub fn modes_count(mut self, count: u32) -> Self {
        self.tokens.push(format!("MODES={}", count));
        self
    }

    pub fn status_msg(mut self, symbols: &str) -> Self {
        self.tokens.push(format!("STATUSMSG={}", symbols));
        self
    }

    pub fn excepts(mut self, mode_char: Option<char>) -> Self {
        if let Some(c) = mode_char {
            self.tokens.push(format!("EXCEPTS={}", c));
        } else {
            self.tokens.push("EXCEPTS".to_string());
        }
        self
    }

    pub fn invex(mut self, mode_char: Option<char>) -> Self {
        if let Some(c) = mode_char {
            self.tokens.push(format!("INVEX={}", c));
        } else {
            self.tokens.push("INVEX".to_string());
        }
        self
    }
    pub fn custom(mut self, key: &str, value: Option<&str>) -> Self {
        if let Some(v) = value {
            self.tokens.push(format!("{}={}", key, v));
        } else {
            self.tokens.push(key.to_string());
        }
        self
    }

    pub fn build(self) -> String {
        self.tokens.join(" ")
    }

    pub fn build_lines(self, max_per_line: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current = Vec::new();

        for token in self.tokens {
            current.push(token);
            if current.len() >= max_per_line {
                lines.push(current.join(" "));
                current.clear();
            }
        }

        if !current.is_empty() {
            lines.push(current.join(" "));
        }

        lines
    }
}
