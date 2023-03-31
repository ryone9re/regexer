//! 命令列と入力文字列を受け取り､マッチングを行う
use super::Instruction;
use crate::helper::safe_add;
use std::{
    // collections::VecDeque,
    error::Error,
    fmt::{self, Display},
    // slice::SliceIndex,
};

#[derive(Debug)]
pub enum EvalError {
    PCOverFlow,
    SPOverFlow,
    POSOvreFlow,
    InvalidPC,
    // InvalidContext,
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EvalError: {:?}", self)
    }
}

impl Error for EvalError {}

/// 深さ優先探索で再帰的にマッチングを行う関数
fn eval_depth(
    inst: &[Instruction],
    line: &[char],
    mut pc: usize,
    mut sp: usize,
) -> Result<bool, EvalError> {
    let mut pos: usize = 0;
    let mut init_position_state = false;
    loop {
        let next = if let Some(i) = inst.get(pc) {
            i
        } else {
            return Err(EvalError::InvalidPC);
        };

        match next {
            Instruction::Char(c) => {
                if let Some(sp_c) = line.get(sp) {
                    if *c == '\n' {
                        init_position_state = true;
                    }
                    if *c == '.' {
                        safe_add(&mut pc, &1, || EvalError::PCOverFlow)?;
                        safe_add(&mut sp, &1, || EvalError::SPOverFlow)?;
                    } else if c == sp_c {
                        safe_add(&mut pc, &1, || EvalError::PCOverFlow)?;
                        safe_add(&mut sp, &1, || EvalError::SPOverFlow)?;
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }

                if init_position_state {
                    pos = 0;
                    init_position_state = false;
                } else {
                    safe_add(&mut pos, &1, || EvalError::POSOvreFlow)?;
                }
            }
            Instruction::Match => {
                return Ok(true);
            }
            Instruction::Jump(addr) => {
                pc = *addr;
            }
            Instruction::Split(addr1, addr2) => {
                if eval_depth(inst, line, *addr1, sp)? || eval_depth(inst, line, *addr2, sp)? {
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
            Instruction::MatchBegin => {
                if pos == 0 {
                    safe_add(&mut pc, &1, || EvalError::PCOverFlow)?;
                } else {
                    return Ok(false);
                }
            }
            Instruction::MatchEnd => {
                if let Some(c) = line.get(sp) {
                    if *c == '\n' {
                        safe_add(&mut pc, &1, || EvalError::PCOverFlow)?;
                    } else {
                        return Ok(false);
                    }
                } else if pos == line.len() {
                    safe_add(&mut pc, &1, || EvalError::PCOverFlow)?;
                } else {
                    return Ok(false);
                }
            }
        }
    }
}

/// 幅優先探索でマッチングを行う関数
fn eval_width(
    _inst: &[Instruction],
    _line: &[char],
    mut _pc: usize,
    mut _sp: usize,
) -> Result<bool, EvalError> {
    Ok(false)
}

/// 命令列の評価を行う関数
///
/// instが命令列となり､その命令列を用いて入力文字列lineにマッチさせる
/// is_depthがtrueの場合に深さ優先探索を､falseの場合に幅優先探索を行う
///
/// 実行時にエラーが起きた場合はErrを返す
/// マッチ成功時はOk(true)を､失敗時はOk(false)を返す
pub fn eval(inst: &[Instruction], line: &[char], is_depth: bool) -> Result<bool, EvalError> {
    if is_depth {
        eval_depth(inst, line, 0, 0)
    } else {
        eval_width(inst, line, 0, 0)
    }
}
