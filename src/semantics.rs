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

use std::collections::HashSet;

use crate::ast::*;
use crate::errors::{
    Error,
    ErrorClass::{Parsing, Semantic},
    ParsingError, SemanticError,
};
use crate::expr::*;
use crate::symtab::*;
use crate::walk::ast::AstVisitor;

pub struct Analyser;

impl Analyser {
    pub fn new() -> Analyser {
        Analyser {}
    }
}

impl AstVisitor for Analyser {
    fn enter(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        let kind = stage.arena[id].kind.clone();

        match &kind {
            AstKind::Function { params, block, .. } => {
                for &param in params {
                    if let AstKind::Parameter { .. } = &stage.arena[param].kind
                    {
                    } else if block.is_some() {
                        return Err(stage.arena.node_error(id, Semantic(
                            SemanticError::UnnamedParameterInFunctionDefinition,
                        )));
                    }
                }

                Ok(())
            }

            AstKind::Identifier { name, .. } => {
                if resolve(&stage.symtab, &stage.arena, &id).is_some() {
                    Ok(())
                } else {
                    Err(stage.arena.node_error(
                        id,
                        Parsing(ParsingError::UndeclaredIdentifier(
                            stage.arena.token_str(name).to_string(),
                        )),
                    ))
                }
            }

            AstKind::Assign { left, .. }
            | AstKind::CompoundAssign { left, .. } => {
                if !is_lvalue(&stage.arena, *left) {
                    Err(stage
                        .arena
                        .node_error(id, Semantic(SemanticError::NotAnLvalue)))
                } else {
                    Ok(())
                }
            }

            AstKind::AddrOf { expr } => {
                if !is_lvalue(&stage.arena, *expr) {
                    Err(stage
                        .arena
                        .node_error(id, Semantic(SemanticError::NotAnLvalue)))
                } else {
                    Ok(())
                }
            }

            AstKind::Case { .. } => {
                let scope = stage.arena[id].scope;
                if stage.symtab.upto(scope, ScopeKind::Switch).is_some() {
                    Ok(())
                } else {
                    Err(stage.arena.node_error(
                        id,
                        Semantic(SemanticError::CaseOutsideOfSwitch),
                    ))
                }
            }

            AstKind::Default { .. } => {
                let scope = stage.arena[id].scope;
                if stage.symtab.upto(scope, ScopeKind::Switch).is_some() {
                    Ok(())
                } else {
                    Err(stage.arena.node_error(
                        id,
                        Semantic(SemanticError::DefaultOutsideOfSwitch),
                    ))
                }
            }

            _ => Ok(()),
        }
    }

    fn visit_break(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        let scope = stage.arena[id].scope;
        if stage
            .symtab
            .upto_any(scope, &[ScopeKind::Loop, ScopeKind::Switch])
            .is_some()
        {
            Ok(())
        } else {
            Err(stage.arena.node_error(
                id,
                Semantic(SemanticError::BreakNotInALoopOrSwitch),
            ))
        }
    }

    fn visit_continue(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        let scope = stage.arena[id].scope;
        if stage.symtab.upto(scope, ScopeKind::Loop).is_some() {
            Ok(())
        } else {
            Err(stage
                .arena
                .node_error(id, Semantic(SemanticError::ContinueNotInALoop)))
        }
    }

    fn visit_goto(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        if let AstKind::GoTo { label } = &stage.arena[id].kind {
            let scope = stage.arena[id].scope;
            let name = stage.arena.token_str(label).to_string();
            if stage.symtab.get_label(scope, &name).is_some() {
                Ok(())
            } else {
                Err(stage.arena.node_error(
                    id,
                    Semantic(SemanticError::LabelNotFound(name)),
                ))
            }
        } else {
            Ok(())
        }
    }

    fn visit_variable(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        if let AstKind::Variable { init, .. } = &stage.arena[id].kind
            && let Some(init) = init
            && let Some(sym) = resolve(&stage.symtab, &stage.arena, &id)
            && has_static_storage_duration(&stage.symtab, sym)
            && !is_const_expr(&stage.arena, *init)
        {
            return Err(stage
                .arena
                .node_error(id, Semantic(SemanticError::NotAConstExpression)));
        }

        Ok(())
    }

    fn visit_case(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        if let AstKind::Case { expr, .. } = &stage.arena[id].kind {
            let expr = *expr;
            if !is_const_int_expr(&stage.arena, expr)
                && !is_const_unsigned_int_expr(&stage.arena, expr)
            {
                return Err(stage.arena.node_error(
                    id,
                    Semantic(SemanticError::NotAConstExpression),
                ));
            }
        }

        Ok(())
    }

    fn visit_switch(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        let cases = if let AstKind::Switch { cases, .. } = &stage.arena[id].kind
        {
            cases.clone()
        } else {
            unreachable!()
        };

        let mut case_values: HashSet<u64> = HashSet::new();
        let mut has_default = false;

        for &case in &cases {
            let case_kind = stage.arena[case].kind.clone();
            match &case_kind {
                AstKind::Case { expr, .. } => {
                    if is_const_unsigned_int_expr(&stage.arena, *expr) {
                        if !case_values.insert(const_unsigned_int_value(
                            &stage.arena,
                            *expr,
                        )) {
                            return Err(stage.arena.node_error(
                                id,
                                Semantic(
                                    SemanticError::DuplicateCaseExpression,
                                ),
                            ));
                        }
                    } else if is_const_int_expr(&stage.arena, *expr) {
                        if !case_values
                            .insert(const_int_value(&stage.arena, *expr) as u64)
                        {
                            return Err(stage.arena.node_error(
                                id,
                                Semantic(
                                    SemanticError::DuplicateCaseExpression,
                                ),
                            ));
                        }
                    } else {
                        return Err(stage.arena.node_error(
                            id,
                            Semantic(SemanticError::NotAConstExpression),
                        ));
                    }
                }
                AstKind::Default { .. } => {
                    if has_default {
                        return Err(stage.arena.node_error(
                            id,
                            Semantic(SemanticError::DuplicateDefaultCase),
                        ));
                    }
                    has_default = true;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    fn visit_call(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        if let AstKind::Call { expr, .. } = &stage.arena[id].kind {
            let callee = *expr;
            if !is_callable(&stage.arena, &stage.symtab, callee) {
                return Err(stage
                    .arena
                    .node_error(id, Semantic(SemanticError::NotAFunction)));
            }
        }

        Ok(())
    }

    fn visit_pre_incr(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.check_lvalue(stage, id)
    }

    fn visit_pre_decr(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.check_lvalue(stage, id)
    }

    fn visit_post_incr(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.check_lvalue(stage, id)
    }

    fn visit_post_decr(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.check_lvalue(stage, id)
    }
}

impl Analyser {
    fn check_lvalue(
        &self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        let inner = match &stage.arena[id].kind {
            AstKind::PreIncr { expr }
            | AstKind::PreDecr { expr }
            | AstKind::PostIncr { expr }
            | AstKind::PostDecr { expr } => *expr,
            _ => unreachable!(),
        };

        if !is_lvalue(&stage.arena, inner) {
            Err(stage
                .arena
                .node_error(id, Semantic(SemanticError::NotAnLvalue)))
        } else {
            Ok(())
        }
    }
}

impl Default for Analyser {
    fn default() -> Self {
        Self::new()
    }
}
