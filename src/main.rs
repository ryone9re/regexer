mod engine;
mod helper;

use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

use helper::DynError;

/// ファイルをオープンし､行ごとにマッチングを行う
///
/// マッチングはそれぞれの行頭から1文字ずつずらして行い､
/// いずれかにマッチした場合に､その行がマッチしたものとみなす
///
/// 例えば､abcdという文字列があった場合､以下の順にマッチが行われ､
/// このいずれかにマッチした場合､与えられた正規表現にマッチする行と判定する
///
/// - abcd
/// - bcd
/// - cd
/// - d
fn match_file(expr: &str, file_path: &str) -> Result<(), DynError> {
    let f = File::open(file_path)?;
    let reader = BufReader::new(f);

    engine::print(expr)?;
    println!();

    for line in reader.lines() {
        let line = line?;
        for (i, _) in line.char_indices() {
            if engine::do_matching(expr, &line[i..], true)? {
                println!("{line}");
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), DynError> {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        eprintln!("usage: {} regex file", args[0]);
        return Err("invalid arguments".into());
    } else {
        match_file(&args[1], &args[2])?;
    }

    Ok(())
}

// 単体テスト
#[cfg(test)]
mod tests {
    use crate::{
        engine::do_matching,
        helper::{safe_add, SafeAdd},
    };

    #[test]
    fn test_safe_add() {
        let n: usize = 10;
        assert_eq!(Some(30), n.safe_add(&20));

        let n: usize = !0; // 2^64 - 1 (64 bits CPU)
        assert_eq!(None, n.safe_add(&1));

        let mut n: usize = 10;
        assert!(safe_add(&mut n, &20, || ()).is_ok());

        let mut n: usize = !0;
        assert!(safe_add(&mut n, &20, || ()).is_err());
    }

    #[test]
    fn test_matching() {
        // パースエラー
        assert!(do_matching("+b", "bbb", true).is_err());
        assert!(do_matching("*b", "bbb", true).is_err());
        assert!(do_matching("|b", "bbb", true).is_err());
        assert!(do_matching("?b", "bbb", true).is_err());

        // パース成功､マッチ成功
        assert!(do_matching("abc|def", "def", true).unwrap());
        assert!(do_matching("(abc)*", "abcabc", true).unwrap());
        assert!(do_matching("(ab|cd)+", "abcdcd", true).unwrap());
        assert!(do_matching("abc?", "ab", true).unwrap());

        // パース成功､マッチ失敗
        assert!(!do_matching("abc|def", "efa", true).unwrap());
        assert!(!do_matching("(ab|cd)+", "", true).unwrap());
        assert!(!do_matching("abc?", "acb", true).unwrap());
    }

    #[test]
    fn test_dot_pattern() {
        // パース成功､マッチ成功
        assert!(do_matching(".", "a", true).unwrap());
        assert!(do_matching(".", "b", true).unwrap());

        // パース成功､マッチ失敗
        assert!(!do_matching(".", "", true).unwrap());
    }

    #[test]
    fn test_dot_star_pattern() {
        // パース成功､マッチ成功
        assert!(do_matching(".*", "a", true).unwrap());
        assert!(do_matching(".*", "b", true).unwrap());
        assert!(do_matching(".*", "ab", true).unwrap());
        assert!(do_matching(".*", "bcde", true).unwrap());
    }

    #[test]
    fn test_match_begin() {
        assert!(do_matching("^foo", "foo", true).unwrap());
        assert!(!do_matching("^foo", "barfoo", true).unwrap());
    }

    #[test]
    fn test_match_end() {
        assert!(do_matching("foo$", "foo", true).unwrap());
        assert!(!do_matching("foo$", "foobar", true).unwrap());
        assert!(do_matching("foo$", "foo\n", true).unwrap());
    }

    #[test]
    fn test_match_begin_end() {
        assert!(do_matching("^foo$", "foo", true).unwrap());
        assert!(!do_matching("^foo$", "foobar", true).unwrap());
        assert!(!do_matching("^foo$", "barfoo", true).unwrap());
        assert!(!do_matching("^foo$", "barfoobar", true).unwrap());
    }
}
