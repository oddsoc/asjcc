//  SPDX-License-Identifier: MIT
/*
 *  Copyright (c) 2025 Andrew Scott-Jones
 *
 *  Permission is hereby granted, free of charge, to any person obtaining a
 *  copy of this software and associated documentation files (the "Software"),
 *  to deal in the Software without restriction, including without limitation
 *  the rights to use, copy, modify, merge, publish, distribute, sublicense,
 *  and/or sell copies of the Software, and to permit persons to whom the
 *  Software is furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be included in
 *  all copies or substantial portions of the Software.
 *
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 *  OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 *  FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 *  DEALINGS IN THE SOFTWARE.
 */

use crate::errors::Error;
use crate::mir::x86_64::sysv::mir::*;
use crate::walk::{Pass, Tree};

impl Tree for MirStage {
    type Id = MirId;

    fn root(&self, index: usize) -> Option<MirId> {
        self.mir.top_level.get(index).copied()
    }

    fn child(&self, id: MirId, index: usize) -> Option<MirId> {
        match &self.mir[id] {
            Mir::Op(op) => match op {
                Op::Ret
                | Op::PushBytes(_)
                | Op::PopBytes(_)
                | Op::Jmp(_)
                | Op::Jnz(_)
                | Op::Jcc { .. }
                | Op::Cqo(_) => None,
                Op::Lea(a, b, ..)
                | Op::Mov(a, b, ..)
                | Op::MovAbs(a, b, ..)
                | Op::Movsx(a, b, ..)
                | Op::Cvttsd2si(a, b, ..)
                | Op::Cvtsi2sd(a, b, ..)
                | Op::Imul(a, b, ..)
                | Op::Add(a, b, ..)
                | Op::Sub(a, b, ..)
                | Op::Shl(a, b, ..)
                | Op::Shr(a, b, ..)
                | Op::Sar(a, b, ..)
                | Op::And(a, b, ..)
                | Op::Or(a, b, ..)
                | Op::Xor(a, b, ..)
                | Op::Cmp(a, b, ..)
                | Op::Test(a, b) => match index {
                    0 => Some(*a),
                    1 => Some(*b),
                    _ => None,
                },
                Op::Push(a, ..) => {
                    if index == 0 {
                        Some(*a)
                    } else {
                        None
                    }
                }
                Op::Movsd(a, b)
                | Op::Xorpd(a, b)
                | Op::Ucomisd(a, b)
                | Op::Addsd(a, b)
                | Op::Subsd(a, b)
                | Op::Mulsd(a, b)
                | Op::Divsd(a, b) => match index {
                    0 => Some(*a),
                    1 => Some(*b),
                    _ => None,
                },
                Op::Not(a, ..)
                | Op::Neg(a, ..)
                | Op::Mul(a, ..)
                | Op::Idiv(a, ..)
                | Op::Div(a, ..)
                | Op::Call(a) => {
                    if index == 0 {
                        Some(*a)
                    } else {
                        None
                    }
                }
                Op::Setcc { dst, .. } => {
                    if index == 0 {
                        Some(*dst)
                    } else {
                        None
                    }
                }
            },
            Mir::Operand(op, _) => match op {
                Operand::Imm(_) | Operand::Reg(_) | Operand::Sym(_) => None,
                Operand::Mem(base, _) => {
                    if index == 0 {
                        Some(*base)
                    } else {
                        None
                    }
                }
                Operand::Idx {
                    base, index: idx, ..
                } => match index {
                    0 => Some(*base),
                    1 => Some(*idx),
                    _ => None,
                },
            },
            Mir::Object(obj) => match obj {
                Object::Function { mir, .. } => mir.get(index).copied(),
                Object::Data { inits, .. } => inits.get(index).copied(),
                Object::InitInteger { .. } | Object::InitDouble(_) => None,
            },
            Mir::Meta(meta) => match meta {
                Meta::Label(_) => None,
            },
        }
    }
}

pub trait MirVisitor {
    fn enter(
        &mut self,
        _stage: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit(&mut self, stage: &mut MirStage, id: MirId) -> Result<(), Error> {
        match &stage.mir[id] {
            Mir::Op(Op::Ret) => self.visit_ret(stage, id),
            Mir::Op(Op::Lea(..)) => self.visit_lea(stage, id),
            Mir::Op(Op::Mov(..)) => self.visit_mov(stage, id),
            Mir::Op(Op::Movsd(..)) => self.visit_movsd(stage, id),
            Mir::Op(Op::MovAbs(..)) => self.visit_mov_abs(stage, id),
            Mir::Op(Op::Movsx(..)) => self.visit_movsx(stage, id),
            Mir::Op(Op::Cvttsd2si(..)) => self.visit_cvttsd2si(stage, id),
            Mir::Op(Op::Cvtsi2sd(..)) => self.visit_cvtsi2sd(stage, id),
            Mir::Op(Op::Not(..)) => self.visit_not(stage, id),
            Mir::Op(Op::Neg(..)) => self.visit_neg(stage, id),
            Mir::Op(Op::Imul(..)) => self.visit_imul(stage, id),
            Mir::Op(Op::Mul(..)) => self.visit_mul(stage, id),
            Mir::Op(Op::Idiv(..)) => self.visit_idiv(stage, id),
            Mir::Op(Op::Div(..)) => self.visit_div(stage, id),
            Mir::Op(Op::Add(..)) => self.visit_add(stage, id),
            Mir::Op(Op::Sub(..)) => self.visit_sub(stage, id),
            Mir::Op(Op::Shl(..)) => self.visit_shl(stage, id),
            Mir::Op(Op::Shr(..)) => self.visit_shr(stage, id),
            Mir::Op(Op::Sar(..)) => self.visit_sar(stage, id),
            Mir::Op(Op::And(..)) => self.visit_and(stage, id),
            Mir::Op(Op::Or(..)) => self.visit_or(stage, id),
            Mir::Op(Op::Xor(..)) => self.visit_xor(stage, id),
            Mir::Op(Op::Xorpd(..)) => self.visit_xorpd(stage, id),
            Mir::Op(Op::Cmp(..)) => self.visit_cmp(stage, id),
            Mir::Op(Op::Ucomisd(..)) => self.visit_ucomisd(stage, id),
            Mir::Op(Op::Push(..)) => self.visit_push(stage, id),
            Mir::Op(Op::Call(..)) => self.visit_call(stage, id),
            Mir::Op(Op::PushBytes(_)) => self.visit_push_bytes(stage, id),
            Mir::Op(Op::PopBytes(_)) => self.visit_pop_bytes(stage, id),
            Mir::Op(Op::Jmp(..)) => self.visit_jmp(stage, id),
            Mir::Op(Op::Jnz(..)) => self.visit_jnz(stage, id),
            Mir::Op(Op::Jcc { .. }) => self.visit_jcc(stage, id),
            Mir::Op(Op::Setcc { .. }) => self.visit_setcc(stage, id),
            Mir::Op(Op::Test(..)) => self.visit_test(stage, id),
            Mir::Op(Op::Cqo(_)) => self.visit_cqo(stage, id),
            Mir::Op(Op::Addsd(..)) => self.visit_addsd(stage, id),
            Mir::Op(Op::Subsd(..)) => self.visit_subsd(stage, id),
            Mir::Op(Op::Mulsd(..)) => self.visit_mulsd(stage, id),
            Mir::Op(Op::Divsd(..)) => self.visit_divsd(stage, id),
            Mir::Operand(Operand::Imm(_), _) => self.visit_imm(stage, id),
            Mir::Operand(Operand::Mem(..), _) => self.visit_mem(stage, id),
            Mir::Operand(Operand::Idx { .. }, _) => self.visit_idx(stage, id),
            Mir::Operand(Operand::Reg(_), _) => self.visit_reg(stage, id),
            Mir::Operand(Operand::Sym(_), _) => self.visit_sym(stage, id),
            Mir::Object(Object::Function { .. }) => {
                self.visit_function(stage, id)
            }
            Mir::Object(Object::Data { .. }) => self.visit_data(stage, id),
            Mir::Object(Object::InitInteger { .. }) => {
                self.visit_init_integer(stage, id)
            }
            Mir::Object(Object::InitDouble(_)) => {
                self.visit_init_double(stage, id)
            }
            Mir::Meta(Meta::Label(_)) => self.visit_label(stage, id),
        }
    }

    fn visit_ret(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_lea(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_mov(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_movsd(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_mov_abs(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_movsx(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_cvttsd2si(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_cvtsi2sd(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_not(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_neg(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_imul(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_mul(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_idiv(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_div(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_add(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_sub(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_shl(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_shr(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_sar(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_and(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_or(&mut self, _s: &mut MirStage, _id: MirId) -> Result<(), Error> {
        Ok(())
    }

    fn visit_xor(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_xorpd(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_cmp(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_ucomisd(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_push(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_call(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_label(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_push_bytes(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_pop_bytes(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_jmp(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_jnz(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_jcc(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_setcc(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_test(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_cqo(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_addsd(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_subsd(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_mulsd(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_divsd(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_imm(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_mem(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_idx(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_reg(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_sym(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_function(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_data(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_init_integer(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_init_double(
        &mut self,
        _s: &mut MirStage,
        _id: MirId,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl<V: MirVisitor> Pass<MirStage> for V {
    fn enter(&mut self, stage: &mut MirStage, id: MirId) -> Result<(), Error> {
        MirVisitor::enter(self, stage, id)
    }

    fn exit(&mut self, stage: &mut MirStage, id: MirId) -> Result<(), Error> {
        MirVisitor::visit(self, stage, id)
    }
}

impl<A: MirVisitor, B: MirVisitor> MirVisitor for (A, B) {
    fn enter(&mut self, stage: &mut MirStage, id: MirId) -> Result<(), Error> {
        self.0.enter(stage, id)?;
        self.1.enter(stage, id)
    }

    fn visit(&mut self, stage: &mut MirStage, id: MirId) -> Result<(), Error> {
        self.0.visit(stage, id)?;
        self.1.visit(stage, id)
    }
}
