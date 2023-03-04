//! 正規表現の式をパースし､中小構文木へ変換
use std::{
    error::Error,
    fmt::{self, Display},
    mem::take,
};

#[derive(Debug)]
pub enum Ast {
    Char(char),
    Plus(Box<Ast>),
    Star(Box<Ast>),
    Question(Box<Ast>),
    Or(Box<Ast>, Box<Ast>),
    Seq(Vec<Ast>),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(usize, char), // 誤ったエスケープシーケンス
    InvalidRightParen(usize),   // 開き括弧なし
    NoPrev(usize),              // +,|,*,?の前に式がない
    NoRightParen,               // 閉じ括弧なし
    Empty,                      // 空のパターン
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidEscape(pos, c) => {
                write!(f, "ParseError: invalid escape: pos = {pos}, char = '{c}'")
            }
            ParseError::InvalidRightParen(pos) => {
                write!(f, "ParseError: invalid right parenthesis: pos = {pos}")
            }
            ParseError::NoPrev(pos) => {
                write!(f, "ParseError: no previous expression: pos = {pos}")
            }
            ParseError::NoRightParen => {
                write!(f, "ParseError: no right parenthesis")
            }
            ParseError::Empty => {
                write!(f, "ParseError: empty expression")
            }
        }
    }
}

impl Error for ParseError {} // エラー用に､Errorトレイトを実装

/// parse_plus_star_question関数で利用するための列挙型
enum Psq {
    Plus,
    Star,
    Question,
}

/// 特殊文字のエスケープ
fn parse_escape(pos: usize, c: char) -> Result<Ast, ParseError> {
    match c {
        '\\' | '(' | ')' | '|' | '+' | '*' | '?' => Ok(Ast::Char(c)),
        _ => Err(ParseError::InvalidEscape(pos, c)),
    }
}

/// +,*,?をASTに変換
///
/// 後置記法で､+,*,?の前にパターンがない場合はエラー
///
/// 例 : *ab, abc|+などはエラー
fn parse_plus_star_question(
    seq: &mut Vec<Ast>,
    ast_type: Psq,
    pos: usize,
) -> Result<(), ParseError> {
    if let Some(prev) = seq.pop() {
        let ast = match ast_type {
            Psq::Plus => Ast::Plus(Box::new(prev)),
            Psq::Star => Ast::Star(Box::new(prev)),
            Psq::Question => Ast::Question(Box::new(prev)),
        };
        seq.push(ast);
        Ok(())
    } else {
        Err(ParseError::NoPrev(pos))
    }
}

/// Orで結合された複数の式をASTに変換
///
/// 例えば､abc|def|ghiは､AST::Or("abc", Ast::Or("def", "fhi"))というASTとなる｡
fn fold_or(mut seq_or: Vec<Ast>) -> Option<Ast> {
    if seq_or.len() > 1 {
        // seq_orの要素が複数ある場合は､Orで式を結合
        let mut ast = seq_or.pop().unwrap();
        seq_or.reverse();
        for s in seq_or {
            ast = Ast::Or(Box::new(s), Box::new(ast));
        }
        Some(ast)
    } else {
        // seq_orの要素が1つのみの場合は､Orではなく､最初の値を返す
        seq_or.pop()
    }
}

/// 正規表現を中小構文木に変換
pub fn parse(expr: &str) -> Result<Ast, ParseError> {
    // 内部状態を表現するための型
    // Char 状態 : 文字列処理中
    // Escape 状態 : エスケープシーケンス処理中
    enum ParseState {
        Char,
        Escape,
    }

    let mut seq = Vec::new(); // 現在のSeqのコンテキスト
    let mut seq_or = Vec::new(); // 現在のOrコンテキスト
    let mut stack = Vec::new(); // コンテキストのスタック
    let mut state = ParseState::Char; // 現在の状態

    for (i, c) in expr.chars().enumerate() {
        match &state {
            ParseState::Char => match c {
                '+' => parse_plus_star_question(&mut seq, Psq::Plus, i)?,
                '*' => parse_plus_star_question(&mut seq, Psq::Star, i)?,
                '?' => parse_plus_star_question(&mut seq, Psq::Question, i)?,
                '(' => {
                    // 現在のコンテキストをスタックに保存し､
                    // 現在のコンテキストをからの状態にする
                    let prev = take(&mut seq);
                    let prev_or = take(&mut seq_or);
                    stack.push((prev, prev_or));
                }
                ')' => {
                    // 現在のコンテキストをスタックからポップ
                    if let Some((mut prev, prev_or)) = stack.pop() {
                        // "()"のように､式が空の場合はpushしない
                        if !seq.is_empty() {
                            seq_or.push(Ast::Seq(seq));
                        }

                        // Orを生成
                        if let Some(ast) = fold_or(seq_or) {
                            prev.push(ast);
                        }

                        // 以前のコンテキストを､現在のコンテキストにする
                        seq = prev;
                        seq_or = prev_or;
                    } else {
                        // "abc)"のように､開き括弧がないのに閉じ括弧がある場合はエラー
                        return Err(ParseError::InvalidRightParen(i));
                    }
                }
                '|' => {
                    if seq.is_empty() {
                        // "||", "(|abc)"などと､式画からの場合はエラー
                        return Err(ParseError::NoPrev(i));
                    } else {
                        let prev = take(&mut seq);
                        seq_or.push(Ast::Seq(prev));
                    }
                }
                '\\' => state = ParseState::Escape,
                _ => seq.push(Ast::Char(c)),
            },
            ParseState::Escape => {
                // エスケープシーケンス処理
                let ast = parse_escape(i, c)?;
                seq.push(ast);
                state = ParseState::Char;
            }
        }
    }

    // 閉じ括弧が足りない場合はエラー
    if !stack.is_empty() {
        return Err(ParseError::NoRightParen);
    }

    // "()"のように､式画からの場合はpushしない
    if !seq.is_empty() {
        seq_or.push(Ast::Seq(seq));
    }

    // Orを生成し､成功した場合はそれを返す
    if let Some(ast) = fold_or(seq_or) {
        Ok(ast)
    } else {
        Err(ParseError::Empty)
    }
}
