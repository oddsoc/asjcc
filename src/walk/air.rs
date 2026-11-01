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

use crate::air::tac::*;
use crate::errors::Error;
use crate::walk::{Pass, Tree};

impl Tree for AirStage {
    type Id = TacId;

    fn root(&self, index: usize) -> Option<TacId> {
        self.tac.top_level.get(index).copied()
    }

    fn child(&self, id: TacId, index: usize) -> Option<TacId> {
        match &self.tac[id] {
            Tac::Op(op) => match op {
                Op::Call {
                    func, args, dst, ..
                } => {
                    if index == 0 {
                        Some(*func)
                    } else if index == args.len() + 1 {
                        Some(*dst)
                    } else {
                        args.get(index - 1).copied()
                    }
                }
                Op::DoubleToInt { src, dst, .. }
                | Op::DoubleToUlong { src, dst, .. }
                | Op::IntToDouble { src, dst }
                | Op::SignExt { src, dst, .. }
                | Op::ZeroExt { src, dst, .. }
                | Op::Truncate { src, dst, .. }
                | Op::Inv { src, dst }
                | Op::Neg { src, dst, .. }
                | Op::Not { src, dst, .. }
                | Op::Copy { src, dst, .. }
                | Op::CopyToOffset { src, dst, .. }
                | Op::GetAddr { src, dst, .. }
                | Op::Load { src, dst, .. }
                | Op::Store { src, dst, .. } => match index {
                    0 => Some(*src),
                    1 => Some(*dst),
                    _ => None,
                },
                Op::Mul { lhs, rhs, dst, .. }
                | Op::Div { lhs, rhs, dst, .. }
                | Op::Mod { lhs, rhs, dst, .. }
                | Op::Add { lhs, rhs, dst, .. }
                | Op::AddPtr { lhs, rhs, dst, .. }
                | Op::Sub { lhs, rhs, dst, .. }
                | Op::LeftShift { lhs, rhs, dst, .. }
                | Op::RightShift { lhs, rhs, dst, .. }
                | Op::And { lhs, rhs, dst, .. }
                | Op::Or { lhs, rhs, dst, .. }
                | Op::Xor { lhs, rhs, dst, .. }
                | Op::Equal { lhs, rhs, dst, .. }
                | Op::NotEq { lhs, rhs, dst, .. }
                | Op::Less { lhs, rhs, dst, .. }
                | Op::LessOrEq { lhs, rhs, dst, .. }
                | Op::Greater { lhs, rhs, dst, .. }
                | Op::GreaterOrEq { lhs, rhs, dst, .. } => match index {
                    0 => Some(*lhs),
                    1 => Some(*rhs),
                    2 => Some(*dst),
                    _ => None,
                },
                Op::Jump(_) => None,
                Op::JumpOnZero { expr, .. }
                | Op::JumpOnNotZero { expr, .. } => {
                    if index == 0 {
                        Some(*expr)
                    } else {
                        None
                    }
                }
                Op::Return(_, expr) => {
                    if index == 0 {
                        Some(*expr)
                    } else {
                        None
                    }
                }
            },
            Tac::Operand(op) => match op {
                Operand::Integer { .. }
                | Operand::Double(_)
                | Operand::Pseudo(..)
                | Operand::Sym(..) => None,
                Operand::PlainOperand(id) | Operand::DereferencedPtr(id) => {
                    if index == 0 {
                        Some(*id)
                    } else {
                        None
                    }
                }
            },
            Tac::Object(obj) => match obj {
                Object::StaticInit(_, id) => {
                    if index == 0 {
                        Some(*id)
                    } else {
                        None
                    }
                }
                Object::StaticInitList(inits, _) => inits.get(index).copied(),
                Object::Data(_, _, _, _, id) => {
                    if index == 0 {
                        Some(*id)
                    } else {
                        None
                    }
                }
                Object::Function { params, tac, .. } => {
                    if let Some(&param) = params.get(index) {
                        Some(param)
                    } else {
                        tac.get(index - params.len()).copied()
                    }
                }
            },
            Tac::Meta(meta) => match meta {
                Meta::Label(_) => None,
            },
        }
    }
}

pub trait AirVisitor {
    fn enter(
        &mut self,
        _stage: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit(&mut self, stage: &mut AirStage, id: TacId) -> Result<(), Error> {
        match &stage.tac[id] {
            Tac::Op(Op::Call { .. }) => self.visit_call(stage, id),
            Tac::Op(Op::DoubleToInt { .. }) => {
                self.visit_double_to_int(stage, id)
            }
            Tac::Op(Op::DoubleToUlong { .. }) => {
                self.visit_double_to_ulong(stage, id)
            }
            Tac::Op(Op::IntToDouble { .. }) => {
                self.visit_int_to_double(stage, id)
            }
            Tac::Op(Op::SignExt { .. }) => self.visit_sign_ext(stage, id),
            Tac::Op(Op::ZeroExt { .. }) => self.visit_zero_ext(stage, id),
            Tac::Op(Op::Truncate { .. }) => self.visit_truncate(stage, id),
            Tac::Op(Op::Inv { .. }) => self.visit_inv(stage, id),
            Tac::Op(Op::Neg { .. }) => self.visit_neg(stage, id),
            Tac::Op(Op::Not { .. }) => self.visit_not(stage, id),
            Tac::Op(Op::Mul { .. }) => self.visit_mul(stage, id),
            Tac::Op(Op::Div { .. }) => self.visit_div(stage, id),
            Tac::Op(Op::Mod { .. }) => self.visit_mod(stage, id),
            Tac::Op(Op::Add { .. }) => self.visit_add(stage, id),
            Tac::Op(Op::AddPtr { .. }) => self.visit_add_ptr(stage, id),
            Tac::Op(Op::Sub { .. }) => self.visit_sub(stage, id),
            Tac::Op(Op::LeftShift { .. }) => self.visit_left_shift(stage, id),
            Tac::Op(Op::RightShift { .. }) => self.visit_right_shift(stage, id),
            Tac::Op(Op::And { .. }) => self.visit_and(stage, id),
            Tac::Op(Op::Or { .. }) => self.visit_or(stage, id),
            Tac::Op(Op::Xor { .. }) => self.visit_xor(stage, id),
            Tac::Op(Op::Equal { .. }) => self.visit_equal(stage, id),
            Tac::Op(Op::NotEq { .. }) => self.visit_not_eq(stage, id),
            Tac::Op(Op::Less { .. }) => self.visit_less(stage, id),
            Tac::Op(Op::LessOrEq { .. }) => self.visit_less_or_eq(stage, id),
            Tac::Op(Op::Greater { .. }) => self.visit_greater(stage, id),
            Tac::Op(Op::GreaterOrEq { .. }) => {
                self.visit_greater_or_eq(stage, id)
            }
            Tac::Op(Op::Copy { .. }) => self.visit_copy(stage, id),
            Tac::Op(Op::CopyToOffset { .. }) => {
                self.visit_copy_to_offset(stage, id)
            }
            Tac::Op(Op::Jump(_)) => self.visit_jump(stage, id),
            Tac::Op(Op::JumpOnZero { .. }) => {
                self.visit_jump_on_zero(stage, id)
            }
            Tac::Op(Op::JumpOnNotZero { .. }) => {
                self.visit_jump_on_not_zero(stage, id)
            }
            Tac::Op(Op::Return(..)) => self.visit_return(stage, id),
            Tac::Op(Op::GetAddr { .. }) => self.visit_get_addr(stage, id),
            Tac::Op(Op::Load { .. }) => self.visit_load(stage, id),
            Tac::Op(Op::Store { .. }) => self.visit_store(stage, id),
            Tac::Operand(Operand::Integer { .. }) => {
                self.visit_integer(stage, id)
            }
            Tac::Operand(Operand::Double(_)) => self.visit_double(stage, id),
            Tac::Operand(Operand::Pseudo(..)) => self.visit_pseudo(stage, id),
            Tac::Operand(Operand::PlainOperand(_)) => {
                self.visit_plain_operand(stage, id)
            }
            Tac::Operand(Operand::DereferencedPtr(_)) => {
                self.visit_dereferenced_ptr(stage, id)
            }
            Tac::Operand(Operand::Sym(..)) => self.visit_sym(stage, id),
            Tac::Object(Object::StaticInit(..)) => {
                self.visit_static_init(stage, id)
            }
            Tac::Object(Object::StaticInitList(..)) => {
                self.visit_static_init_list(stage, id)
            }
            Tac::Object(Object::Function { .. }) => {
                self.visit_function(stage, id)
            }
            Tac::Object(Object::Data(..)) => self.visit_data(stage, id),
            Tac::Meta(Meta::Label(_)) => self.visit_label(stage, id),
        }
    }

    fn visit_call(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_double_to_int(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_double_to_ulong(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_int_to_double(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_sign_ext(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_zero_ext(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_truncate(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_inv(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_neg(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_not(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_mul(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_div(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_mod(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_add(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_add_ptr(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_sub(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_left_shift(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_right_shift(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_and(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_or(&mut self, _s: &mut AirStage, _id: TacId) -> Result<(), Error> {
        Ok(())
    }

    fn visit_xor(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_equal(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_not_eq(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_less(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_less_or_eq(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_greater(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_greater_or_eq(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_copy(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_copy_to_offset(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_jump(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_jump_on_zero(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_jump_on_not_zero(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_label(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_return(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_get_addr(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_load(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_store(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_integer(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_double(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_pseudo(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_plain_operand(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_dereferenced_ptr(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_sym(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_static_init(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_static_init_list(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_function(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_data(
        &mut self,
        _s: &mut AirStage,
        _id: TacId,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl<V: AirVisitor> Pass<AirStage> for V {
    fn enter(&mut self, stage: &mut AirStage, id: TacId) -> Result<(), Error> {
        AirVisitor::enter(self, stage, id)
    }

    fn exit(&mut self, stage: &mut AirStage, id: TacId) -> Result<(), Error> {
        AirVisitor::visit(self, stage, id)
    }
}

impl<A: AirVisitor, B: AirVisitor> AirVisitor for (A, B) {
    fn enter(&mut self, stage: &mut AirStage, id: TacId) -> Result<(), Error> {
        self.0.enter(stage, id)?;
        self.1.enter(stage, id)
    }

    fn visit(&mut self, stage: &mut AirStage, id: TacId) -> Result<(), Error> {
        self.0.visit(stage, id)?;
        self.1.visit(stage, id)
    }
}
