//! ASTからコード生成を行う
use super::{parser::Ast, Instruction};
use crate::helper::safe_add;
use std::{
    error::Error,
    fmt::{self, Display},
};

/// コード生成エラーを表す型
#[derive(Debug)]
pub enum CodeGenError {
    PCOverFlow,
    FailStar,
    FailOr,
    FailQuestion,
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}

impl Error for CodeGenError {}

#[derive(Debug, Default)]
struct Generator {
    pc: usize,
    insts: Vec<Instruction>,
}

impl Generator {
    /// プログラムカウントをインクリメント
    fn inc_pc(&mut self) -> Result<(), CodeGenError> {
        safe_add(&mut self.pc, &1, || CodeGenError::PCOverFlow)
    }

    /// ASTをパターン分けし､コード生成を行う関数
    fn gen_expr(&mut self, ast: &Ast) -> Result<(), CodeGenError> {
        match ast {
            Ast::Char(c) => self.gen_char(*c)?,
            Ast::Or(e1, e2) => self.gen_or(e1, e2)?,
            Ast::Plus(e) => self.gen_plus(e)?,
            Ast::Star(e) => self.gen_star(e)?,
            Ast::Question(e) => self.gen_question(e)?,
            Ast::Seq(v) => self.gen_seq(v)?,
            Ast::Doller => self.gen_doller()?,
            Ast::Hat => self.gen_hat()?,
        }

        Ok(())
    }

    /// char命令生成器
    fn gen_char(&mut self, c: char) -> Result<(), CodeGenError> {
        let inst = Instruction::Char(c);
        self.insts.push(inst);
        self.inc_pc()?;

        Ok(())
    }

    /// Or演算子のコード生成器
    ///
    /// 以下のようなコードを生成
    ///
    /// ```text
    ///     split L1, L2
    /// L1: e1のコード
    ///     jmp L3
    /// L2: e2のコード
    /// L3:
    /// ```
    fn gen_or(&mut self, e1: &Ast, e2: &Ast) -> Result<(), CodeGenError> {
        // split L1, L2
        let split_addr = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0); // L1 = self.pc L2は仮に0と設定
        self.insts.push(split);

        // L1: e1のコード
        self.gen_expr(e1)?;

        // jmp L3
        let jmp_addr = self.pc;
        self.insts.push(Instruction::Jump(0)); // L3を仮に0と設定

        // L2の値を設定
        self.inc_pc()?;
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
        } else {
            return Err(CodeGenError::FailOr);
        }

        // L2: e2のコード
        self.gen_expr(e2)?;

        // L3の値を設定
        if let Some(Instruction::Jump(l3)) = self.insts.get_mut(jmp_addr) {
            *l3 = self.pc;
        } else {
            return Err(CodeGenError::FailOr);
        }

        Ok(())
    }

    /// plus命令器
    ///
    /// 以下のようなコードを生成
    /// ```text
    /// L1: e1のコード
    ///     split L1, L2
    /// L2:
    /// ```
    fn gen_plus(&mut self, e: &Ast) -> Result<(), CodeGenError> {
        // L1: eのコード生成
        let addr = self.pc;
        self.gen_expr(e)?;

        // split L1, L2
        self.inc_pc()?;
        self.insts.push(Instruction::Split(addr, self.pc));

        // L2は次の命令になる
        Ok(())
    }

    /// star命令生成器
    ///
    /// 以下のようなコードを生成
    /// ```text
    /// L1: split L2, L3
    /// L2: e1のコード
    ///     jmp L1
    /// L3:
    /// ```
    fn gen_star(&mut self, e: &Ast) -> Result<(), CodeGenError> {
        // L1: split L2, L3
        let addr = self.pc;
        self.inc_pc()?;
        self.insts.push(Instruction::Split(self.pc, 0)); // L2はL1直下の行になり､L3は不明のため0と仮定

        // L2: e1のコード
        self.gen_expr(e)?;

        // jmp L1
        self.insts.push(Instruction::Jump(addr));
        self.inc_pc()?;

        // L3:
        if let Some(Instruction::Split(_, l3)) = self.insts.get_mut(addr) {
            *l3 = self.pc;
        } else {
            return Err(CodeGenError::FailStar);
        }

        // L3は次の命令になる
        Ok(())
    }

    /// question命令器
    ///
    /// 以下のようなコードを生成
    /// ```text
    ///     split L1, L2
    /// L1: e1のコード
    /// L2:
    /// ```
    fn gen_question(&mut self, e: &Ast) -> Result<(), CodeGenError> {
        // split L1, L2
        let split_addr = self.pc; // L1はsplit直下の行になる
        self.inc_pc()?;
        let addr = self.pc; // L1はsplit直下の行になる
        self.insts.push(Instruction::Split(addr, 0)); // L2は不明のため0と仮定

        // L1: e1のコード
        self.gen_expr(e)?;

        // L2:
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
        } else {
            return Err(CodeGenError::FailQuestion);
        }

        // L2は次の命令になる
        Ok(())
    }

    /// doller命令器
    /// これは文字列の終端をチェックする
    /// 次の文字が改行か終端ならマッチする
    fn gen_doller(&mut self) -> Result<(), CodeGenError> {
        self.insts.push(Instruction::MatchEnd);

        Ok(())
    }

    /// hat命令器
    /// これは文字列の先頭をチェックする
    /// 文字列の先頭ならマッチする
    fn gen_hat(&mut self) -> Result<(), CodeGenError> {
        self.insts.push(Instruction::MatchBegin);

        Ok(())
    }

    /// 連続するASTのコードを生成
    fn gen_seq(&mut self, exprs: &[Ast]) -> Result<(), CodeGenError> {
        for e in exprs {
            self.gen_expr(e)?;
        }

        Ok(())
    }

    /// コード生成を行う関数の入り口
    fn gen_code(&mut self, ast: &Ast) -> Result<(), CodeGenError> {
        self.gen_expr(ast)?;
        self.inc_pc()?;
        self.insts.push(Instruction::Match);

        Ok(())
    }
}

pub fn gen_code(ast: &Ast) -> Result<Vec<Instruction>, CodeGenError> {
    let mut generator = Generator::default();
    generator.gen_code(ast)?;
    Ok(generator.insts)
}
