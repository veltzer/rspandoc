//! Syntax highlighting of code blocks, with syntect and the syntax
//! definitions bat ships (through two-face). syntect runs on Oniguruma:
//! compiling a syntax's patterns and matching lines is several times
//! faster with it than with the pure-Rust engine, which made highlighting
//! cost more than the rest of a conversion.
//!
//! The tokens are syntect's, so where a token starts and ends is not
//! always where pandoc's highlighter (skylighting) puts it. What matches
//! pandoc is everything around them: the token classes (`kw`, `st`, `co`,
//! ... and the DOCX styles `KeywordTok`, `StringTok`, ...), the markup
//! that carries them and the stylesheet that colours them, so a document
//! or style sheet written for pandoc's highlighted output works unchanged.

use std::sync::LazyLock;

use syntect::parsing::{ParseState, Scope, ScopeStack, SyntaxReference, SyntaxSet};

/// One of pandoc's token types: its HTML class, its DOCX style name, and
/// its look in pandoc's default style (`pygments`).
#[derive(Debug, PartialEq, Eq)]
pub struct TokenType {
    pub class: &'static str,
    pub name: &'static str,
    pub color: Option<&'static str>,
    pub bold: bool,
    pub italic: bool,
}

const fn tok(
    class: &'static str,
    name: &'static str,
    color: Option<&'static str>,
    bold: bool,
    italic: bool,
) -> TokenType {
    TokenType {
        class,
        name,
        color,
        bold,
        italic,
    }
}

/// pandoc's token types, in the order its stylesheet lists them, with the
/// colours of the `pygments` style.
pub const TOKEN_TYPES: &[TokenType] = &[
    tok("al", "Alert", Some("#ff0000"), true, false),
    tok("an", "Annotation", Some("#60a0b0"), true, true),
    tok("at", "Attribute", Some("#7d9029"), false, false),
    tok("bn", "BaseN", Some("#40a070"), false, false),
    tok("bu", "BuiltIn", Some("#008000"), false, false),
    tok("cf", "ControlFlow", Some("#007020"), true, false),
    tok("ch", "Char", Some("#4070a0"), false, false),
    tok("cn", "Constant", Some("#880000"), false, false),
    tok("co", "Comment", Some("#60a0b0"), false, true),
    tok("cv", "CommentVar", Some("#60a0b0"), true, true),
    tok("do", "Documentation", Some("#ba2121"), false, true),
    tok("dt", "DataType", Some("#902000"), false, false),
    tok("dv", "DecVal", Some("#40a070"), false, false),
    tok("er", "Error", Some("#ff0000"), true, false),
    tok("ex", "Extension", None, false, false),
    tok("fl", "Float", Some("#40a070"), false, false),
    tok("fu", "Function", Some("#06287e"), false, false),
    tok("im", "Import", Some("#008000"), true, false),
    tok("in", "Information", Some("#60a0b0"), true, true),
    tok("kw", "Keyword", Some("#007020"), true, false),
    tok("op", "Operator", Some("#666666"), false, false),
    tok("ot", "Other", Some("#007020"), false, false),
    tok("pp", "Preprocessor", Some("#bc7a00"), false, false),
    tok("sc", "SpecialChar", Some("#4070a0"), false, false),
    tok("ss", "SpecialString", Some("#bb6688"), false, false),
    tok("st", "String", Some("#4070a0"), false, false),
    tok("va", "Variable", Some("#19177c"), false, false),
    tok("vs", "VerbatimString", Some("#4070a0"), false, false),
    tok("wa", "Warning", Some("#60a0b0"), true, true),
];

fn token_type(class: &str) -> &'static TokenType {
    TOKEN_TYPES
        .iter()
        .find(|t| t.class == class)
        .unwrap_or(&TOKEN_TYPES[0])
}

/// TextMate scope prefixes and the token class they become, most specific
/// first. A token takes the class of its innermost scope that has one;
/// a token none of whose scopes has one is plain text.
const SCOPE_CLASSES: &[(&str, &str)] = &[
    ("invalid", "er"),
    ("comment", "co"),
    ("constant.character.escape", "sc"),
    ("string.regexp", "ss"),
    ("string", "st"),
    ("constant.numeric.float", "fl"),
    ("constant.numeric.integer.hexadecimal", "bn"),
    ("constant.numeric.integer.octal", "bn"),
    ("constant.numeric.integer.binary", "bn"),
    ("constant.numeric", "dv"),
    ("constant.character", "ch"),
    ("constant", "cn"),
    ("keyword.control.import", "im"),
    ("keyword.control", "cf"),
    ("keyword.operator.word", "kw"),
    ("keyword.operator", "op"),
    ("keyword", "kw"),
    ("storage.type.function", "kw"),
    ("storage.type.class", "kw"),
    ("storage.type", "dt"),
    ("storage", "kw"),
    ("support.type", "dt"),
    ("support.class", "dt"),
    ("support.constant", "cn"),
    ("support.function", "bu"),
    ("entity.name.function", "fu"),
    ("variable.function", "fu"),
    ("variable.language", "kw"),
    ("entity.name.tag", "kw"),
    ("entity.other.attribute-name", "at"),
    ("variable.other.readwrite.shell", "va"),
    ("punctuation.definition.variable.shell", "va"),
    ("meta.preprocessor", "pp"),
];

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_newlines);

static SCOPES: LazyLock<Vec<(Scope, &'static TokenType)>> = LazyLock::new(|| {
    SCOPE_CLASSES
        .iter()
        .filter_map(|(s, c)| Scope::new(s).ok().map(|scope| (scope, token_type(c))))
        .collect()
});

fn classify(stack: &ScopeStack) -> Option<&'static TokenType> {
    stack.as_slice().iter().rev().find_map(|scope| {
        SCOPES
            .iter()
            .find(|(prefix, _)| prefix.is_prefix_of(*scope))
            .map(|(_, t)| *t)
    })
}

/// A run of text and its token type (`None`: plain text).
pub type Token = (Option<&'static TokenType>, String);

/// A highlighted code block.
pub struct Highlighted {
    /// The language's name in the `sourceCode` class of `<code>`.
    pub name: String,
    pub lines: Vec<Vec<Token>>,
}

/// Language names pandoc knows that the syntaxes are found under by
/// another name.
const ALIASES: &[(&str, &str)] = &[
    ("csharp", "cs"),
    ("fsharp", "fs"),
    ("objectivec", "objc"),
    ("shell", "bash"),
    ("zsh", "bash"),
];

fn syntax_for(lang: &str) -> Option<&'static SyntaxReference> {
    let lower = lang.to_lowercase();
    let token = ALIASES
        .iter()
        .find(|(name, _)| *name == lower)
        .map_or(lower.as_str(), |(_, token)| token);
    SYNTAXES.find_syntax_by_token(token)
}

/// `code` highlighted as `lang`, or `None` for a language there is no
/// syntax for.
pub fn highlight(lang: &str, code: &str) -> Option<Highlighted> {
    let syntax = syntax_for(lang)?;
    let name = if syntax.name == "Plain Text" {
        "default".to_owned()
    } else {
        lang.to_lowercase()
    };
    let mut lines = Vec::new();
    if code.is_empty() {
        return Some(Highlighted { name, lines });
    }
    let mut state = ParseState::new(syntax);
    let mut stack = ScopeStack::new();
    // Lines as pandoc's highlighter takes them: a final newline does not
    // start another, empty line.
    for line in code.lines() {
        let text = format!("{line}\n");
        let ops = state.parse_line(&text, &SYNTAXES).ok()?;
        let mut tokens: Vec<Token> = Vec::new();
        // Each operation applies at its offset: the text before it is in
        // the scopes before it.
        let mut from = 0;
        for (at, op) in ops
            .iter()
            .map(|(at, op)| (*at, Some(op)))
            .chain([(text.len(), None)])
        {
            let piece = text[from..at].trim_end_matches('\n');
            if !piece.is_empty() {
                let class = classify(&stack);
                match tokens.last_mut() {
                    Some((c, s)) if *c == class => s.push_str(piece),
                    _ => tokens.push((class, piece.to_owned())),
                }
            }
            if let Some(op) = op {
                stack.apply(op).ok()?;
            }
            from = at;
        }
        lines.push(tokens);
    }
    Some(Highlighted { name, lines })
}

/// The stylesheet a page with highlighted code gets: the rules for the
/// highlighting markup (from skylighting-format-blaze-html, BSD-3-Clause) and a
/// rule per token type.
pub fn css() -> String {
    let mut css = String::from(CSS_PREAMBLE);
    for t in TOKEN_TYPES {
        let mut rule = String::new();
        if let Some(c) = t.color {
            rule.push_str(&format!("color: {c}; "));
        }
        if t.bold {
            rule.push_str("font-weight: bold; ");
        }
        if t.italic {
            rule.push_str("font-style: italic; ");
        }
        css.push_str(&format!(
            "code span.{} {{ {rule}}} /* {} */\n",
            t.class, t.name
        ));
    }
    css
}

const CSS_PREAMBLE: &str = "html { -webkit-text-size-adjust: 100%; }
pre > code.sourceCode { white-space: pre; position: relative; }
pre > code.sourceCode > span { display: inline-block; line-height: 1.25; }
pre > code.sourceCode > span:empty { height: 1.2em; }
.sourceCode { overflow: visible; }
code.sourceCode > span { color: inherit; text-decoration: inherit; }
div.sourceCode { margin: 1em 0; }
pre.sourceCode { margin: 0; }
@media screen {
div.sourceCode { overflow: auto; }
}
@media print {
pre > code.sourceCode { white-space: pre-wrap; }
pre > code.sourceCode > span { text-indent: -5em; padding-left: 5em; }
}
pre.numberSource code
  { counter-reset: source-line 0; }
pre.numberSource code > span
  { position: relative; left: -4em; counter-increment: source-line; }
pre.numberSource code > span > a:first-child::before
  { content: counter(source-line);
    position: relative; left: -1em; text-align: right; vertical-align: baseline;
    border: none; display: inline-block;
    -webkit-touch-callout: none; -webkit-user-select: none;
    -khtml-user-select: none; -moz-user-select: none;
    -ms-user-select: none; user-select: none;
    padding: 0 4px; width: 4em;
    color: #aaaaaa;
  }
pre.numberSource { margin-left: 3em; border-left: 1px solid #aaaaaa;  padding-left: 4px; }
div.sourceCode
  {   }
@media screen {
pre > code.sourceCode > span > a:first-child::before { text-decoration: underline; }
}
";
