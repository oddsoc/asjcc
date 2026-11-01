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

use crate::ast::*;
use crate::errors::Error;
use crate::walk::{Pass, Tree};

impl Tree for AstStage<'_> {
    type Id = AstId;

    fn root(&self, index: usize) -> Option<AstId> {
        self.root.get(index).copied()
    }

    fn child(&self, id: AstId, index: usize) -> Option<AstId> {
        match &self.arena[id].kind {
            AstKind::Initialiser { value, .. } => {
                if index == 0 {
                    *value
                } else {
                    None
                }
            }
            AstKind::CompoundInitialiser(initialiser) => {
                initialiser.initialisers.get(index).copied()
            }
            AstKind::ConstInt(_)
            | AstKind::ConstUnsignedInt(_)
            | AstKind::ConstLong(_)
            | AstKind::ConstUnsignedLong(_)
            | AstKind::ConstDouble(_)
            | AstKind::Void
            | AstKind::Int
            | AstKind::Double
            | AstKind::Identifier { .. }
            | AstKind::Break { .. }
            | AstKind::Continue { .. }
            | AstKind::GoTo { .. }
            | AstKind::EmptyStmt
            | AstKind::Pointer { .. }
            | AstKind::Parameter { .. }
            | AstKind::Array { .. } => None,
            AstKind::Function { params, block, .. } => {
                if let Some(&param) = params.get(index) {
                    Some(param)
                } else if index == params.len() {
                    *block
                } else {
                    None
                }
            }
            AstKind::Block { body } => body.get(index).copied(),
            AstKind::Variable { init, .. } => {
                if index == 0 {
                    *init
                } else {
                    None
                }
            }
            AstKind::Return { expr } | AstKind::ExprStmt { expr } => {
                if index == 0 { Some(*expr) } else { None }
            }
            AstKind::If {
                cond,
                then,
                otherwise,
            } => match index {
                0 => Some(*cond),
                1 => Some(*then),
                2 => *otherwise,
                _ => None,
            },
            AstKind::While { cond, body } | AstKind::DoWhile { cond, body } => {
                match index {
                    0 => Some(*cond),
                    1 => Some(*body),
                    _ => None,
                }
            }
            AstKind::For {
                init,
                cond,
                post,
                body,
            } => {
                let mut index = index;
                if let Some(init) = init {
                    if index == 0 {
                        return Some(*init);
                    }
                    index -= 1;
                }
                if let Some(cond) = cond {
                    if index == 0 {
                        return Some(*cond);
                    }
                    index -= 1;
                }
                if let Some(post) = post {
                    if index == 0 {
                        return Some(*post);
                    }
                    index -= 1;
                }
                if index == 0 { Some(*body) } else { None }
            }
            AstKind::Switch { cond, body, .. } => match index {
                0 => Some(*cond),
                1 => Some(*body),
                _ => None,
            },
            AstKind::Label { stmt, .. } | AstKind::Default { stmt } => {
                if index == 0 { Some(*stmt) } else { None }
            }
            AstKind::Case { expr, stmt, .. } => match index {
                0 => Some(*expr),
                1 => Some(*stmt),
                _ => None,
            },
            AstKind::Ternary {
                left,
                middle,
                right,
            } => match index {
                0 => Some(*left),
                1 => Some(*middle),
                2 => Some(*right),
                _ => None,
            },
            AstKind::Complement { expr }
            | AstKind::Negate { expr }
            | AstKind::Not { expr }
            | AstKind::PreIncr { expr }
            | AstKind::PreDecr { expr }
            | AstKind::PostIncr { expr }
            | AstKind::PostDecr { expr }
            | AstKind::AddrOf { expr }
            | AstKind::Deref { expr } => {
                if index == 0 {
                    Some(*expr)
                } else {
                    None
                }
            }
            AstKind::Add { left, right }
            | AstKind::Subtract { left, right }
            | AstKind::Multiply { left, right }
            | AstKind::Divide { left, right }
            | AstKind::Modulo { left, right }
            | AstKind::LeftShift { left, right }
            | AstKind::RightShift { left, right }
            | AstKind::And { left, right }
            | AstKind::Or { left, right }
            | AstKind::Xor { left, right }
            | AstKind::LogicAnd { left, right }
            | AstKind::LogicOr { left, right }
            | AstKind::Equal { left, right }
            | AstKind::NotEq { left, right }
            | AstKind::Less { left, right }
            | AstKind::LessOrEq { left, right }
            | AstKind::Greater { left, right }
            | AstKind::GreaterOrEq { left, right }
            | AstKind::Assign { left, right }
            | AstKind::CompoundAssign { left, right }
            | AstKind::Subscript { left, right } => match index {
                0 => Some(*left),
                1 => Some(*right),
                _ => None,
            },
            AstKind::Call { expr, args } => {
                if index == 0 {
                    Some(*expr)
                } else {
                    args.get(index - 1).copied()
                }
            }
            AstKind::Cast { type_spec, expr } => match index {
                0 => Some(*expr),
                1 => *type_spec,
                _ => None,
            },
        }
    }
}

pub trait AstVisitor {
    fn enter(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        match &stage.arena[id].kind {
            AstKind::Initialiser { .. } => self.visit_initialiser(stage, id),
            AstKind::CompoundInitialiser(_) => {
                self.visit_compound_initialiser(stage, id)
            }
            AstKind::ConstInt(_) => self.visit_const_int(stage, id),
            AstKind::ConstUnsignedInt(_) => {
                self.visit_const_unsigned_int(stage, id)
            }
            AstKind::ConstLong(_) => self.visit_const_long(stage, id),
            AstKind::ConstUnsignedLong(_) => {
                self.visit_const_unsigned_long(stage, id)
            }
            AstKind::ConstDouble(_) => self.visit_const_double(stage, id),
            AstKind::Void => self.visit_void(stage, id),
            AstKind::Int => self.visit_int(stage, id),
            AstKind::Double => self.visit_double(stage, id),
            AstKind::Pointer { .. } => self.visit_pointer(stage, id),
            AstKind::Function { .. } => self.visit_function(stage, id),
            AstKind::Block { .. } => self.visit_block(stage, id),
            AstKind::Variable { .. } => self.visit_variable(stage, id),
            AstKind::Parameter { .. } => self.visit_parameter(stage, id),
            AstKind::Array { .. } => self.visit_array(stage, id),
            AstKind::Identifier { .. } => self.visit_identifier(stage, id),
            AstKind::Return { .. } => self.visit_return(stage, id),
            AstKind::If { .. } => self.visit_if(stage, id),
            AstKind::Break { .. } => self.visit_break(stage, id),
            AstKind::Continue { .. } => self.visit_continue(stage, id),
            AstKind::While { .. } => self.visit_while(stage, id),
            AstKind::DoWhile { .. } => self.visit_do_while(stage, id),
            AstKind::For { .. } => self.visit_for(stage, id),
            AstKind::Switch { .. } => self.visit_switch(stage, id),
            AstKind::ExprStmt { .. } => self.visit_expr_stmt(stage, id),
            AstKind::GoTo { .. } => self.visit_goto(stage, id),
            AstKind::Label { .. } => self.visit_label(stage, id),
            AstKind::Case { .. } => self.visit_case(stage, id),
            AstKind::Default { .. } => self.visit_default(stage, id),
            AstKind::EmptyStmt => self.visit_empty_stmt(stage, id),
            AstKind::Ternary { .. } => self.visit_ternary(stage, id),
            AstKind::Complement { .. } => self.visit_complement(stage, id),
            AstKind::Negate { .. } => self.visit_negate(stage, id),
            AstKind::Not { .. } => self.visit_not(stage, id),
            AstKind::PreIncr { .. } => self.visit_pre_incr(stage, id),
            AstKind::PreDecr { .. } => self.visit_pre_decr(stage, id),
            AstKind::PostIncr { .. } => self.visit_post_incr(stage, id),
            AstKind::PostDecr { .. } => self.visit_post_decr(stage, id),
            AstKind::Add { .. } => self.visit_add(stage, id),
            AstKind::Subtract { .. } => self.visit_subtract(stage, id),
            AstKind::Multiply { .. } => self.visit_multiply(stage, id),
            AstKind::Divide { .. } => self.visit_divide(stage, id),
            AstKind::Modulo { .. } => self.visit_modulo(stage, id),
            AstKind::LeftShift { .. } => self.visit_left_shift(stage, id),
            AstKind::RightShift { .. } => self.visit_right_shift(stage, id),
            AstKind::And { .. } => self.visit_and(stage, id),
            AstKind::Or { .. } => self.visit_or(stage, id),
            AstKind::Xor { .. } => self.visit_xor(stage, id),
            AstKind::LogicAnd { .. } => self.visit_logic_and(stage, id),
            AstKind::LogicOr { .. } => self.visit_logic_or(stage, id),
            AstKind::Equal { .. } => self.visit_equal(stage, id),
            AstKind::NotEq { .. } => self.visit_not_eq(stage, id),
            AstKind::Less { .. } => self.visit_less(stage, id),
            AstKind::LessOrEq { .. } => self.visit_less_or_eq(stage, id),
            AstKind::Greater { .. } => self.visit_greater(stage, id),
            AstKind::GreaterOrEq { .. } => self.visit_greater_or_eq(stage, id),
            AstKind::Assign { .. } => self.visit_assign(stage, id),
            AstKind::CompoundAssign { .. } => {
                self.visit_compound_assign(stage, id)
            }
            AstKind::Call { .. } => self.visit_call(stage, id),
            AstKind::Subscript { .. } => self.visit_subscript(stage, id),
            AstKind::Cast { .. } => self.visit_cast(stage, id),
            AstKind::AddrOf { .. } => self.visit_addr_of(stage, id),
            AstKind::Deref { .. } => self.visit_deref(stage, id),
        }
    }

    fn visit_initialiser(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_compound_initialiser(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_const_int(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_const_unsigned_int(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_const_long(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_const_unsigned_long(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_const_double(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_void(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_int(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_double(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_pointer(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_function(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_block(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_variable(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_parameter(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_array(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_identifier(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_return(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_if(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_break(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_continue(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_while(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_do_while(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_for(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_switch(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_expr_stmt(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_goto(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_label(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_case(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_default(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_empty_stmt(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_ternary(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_complement(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_negate(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_not(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_pre_incr(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_pre_decr(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_post_incr(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_post_decr(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_add(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_subtract(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_multiply(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_divide(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_modulo(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_left_shift(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_right_shift(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_and(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_or(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_xor(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_logic_and(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_logic_or(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_equal(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_not_eq(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_less(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_less_or_eq(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_greater(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_greater_or_eq(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_assign(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_compound_assign(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_call(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_subscript(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_cast(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_addr_of(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn visit_deref(
        &mut self,
        _stage: &mut AstStage<'_>,
        _id: AstId,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl<'src, V: AstVisitor> Pass<AstStage<'src>> for V {
    fn enter(
        &mut self,
        stage: &mut AstStage<'src>,
        id: AstId,
    ) -> Result<(), Error> {
        AstVisitor::enter(self, stage, id)
    }

    fn exit(
        &mut self,
        stage: &mut AstStage<'src>,
        id: AstId,
    ) -> Result<(), Error> {
        AstVisitor::visit(self, stage, id)
    }
}

impl<A: AstVisitor, B: AstVisitor> AstVisitor for (A, B) {
    fn enter(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.0.enter(stage, id)?;
        self.1.enter(stage, id)
    }

    fn visit(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.0.visit(stage, id)?;
        self.1.visit(stage, id)
    }
}
