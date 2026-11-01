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

use std::collections::HashMap;

use crate::air::AirGenerator;
use crate::ast::*;
use crate::symtab::*;
use crate::tokenising::Token;
use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TacId(pub usize);

#[derive(Debug, Clone)]
pub enum Op {
    Call {
        ty: TypeRef,
        func: TacId,
        args: Vec<TacId>,
        dst: TacId,
    },
    DoubleToInt {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
    DoubleToUlong {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
    IntToDouble {
        src: TacId,
        dst: TacId,
    },
    SignExt {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
    ZeroExt {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
    Truncate {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
    Inv {
        src: TacId,
        dst: TacId,
    },
    Neg {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
    Not {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
    Mul {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    Div {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    Mod {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    Add {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    AddPtr {
        lhs: TacId,
        rhs: TacId,
        scale: usize,
        dst: TacId,
    },
    Sub {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    LeftShift {
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    RightShift {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    And {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    Or {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    Xor {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    Equal {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    NotEq {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    Less {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    LessOrEq {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    Greater {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    GreaterOrEq {
        ty: TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    },
    Copy {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
    CopyToOffset {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
        off: usize,
    },
    Jump(TacId),
    JumpOnZero {
        expr: TacId,
        label: TacId,
    },
    JumpOnNotZero {
        expr: TacId,
        label: TacId,
    },
    Return(TypeRef, TacId),
    GetAddr {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
    Load {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
    Store {
        ty: TypeRef,
        src: TacId,
        dst: TacId,
    },
}

#[derive(Debug, Clone)]
pub enum Operand {
    Integer { ty: TypeRef, value: u64 },
    Double(f64),
    Pseudo(TypeRef, String),
    PlainOperand(TacId),
    DereferencedPtr(TacId),
    Sym(TypeRef, String),
}

#[derive(Debug, Clone)]
pub enum Object {
    StaticInit(TypeRef, TacId),
    StaticInitList(Vec<TacId>, usize),
    Function {
        name: String,
        global: bool,
        params: Vec<TacId>,
        tac: Vec<TacId>,
    },
    Data(TypeRef, String, bool, bool, TacId),
}

#[derive(Debug, Clone)]
pub enum Meta {
    Label(usize),
}

#[derive(Debug, Clone)]
pub enum Tac {
    Op(Op),
    Operand(Operand),
    Object(Object),
    Meta(Meta),
}

#[derive(Debug)]
pub struct TacArena {
    pub arena: Vec<Tac>,
    pub top_level: Vec<TacId>,
}

impl std::ops::Index<TacId> for TacArena {
    type Output = Tac;

    fn index(&self, id: TacId) -> &Tac {
        &self.arena[id.0]
    }
}

#[derive(Debug)]
pub struct AirStage {
    pub tac: TacArena,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Fp64Bits(u64);

impl From<f64> for Fp64Bits {
    fn from(f: f64) -> Self {
        Self(f.to_bits())
    }
}

impl From<Fp64Bits> for f64 {
    fn from(bits: Fp64Bits) -> Self {
        f64::from_bits(bits.0)
    }
}

pub struct TacGenerator {
    label_idx: usize,
    tmp_idx: usize,
    label_map: HashMap<String, usize>,
    fp_consts: HashMap<Fp64Bits, usize>,
    pub arena: Vec<Tac>,
    tac_code: Vec<TacId>,
    symtab: Option<SymTab>,
}

impl AirGenerator for TacGenerator {
    type Air = AirStage;

    fn new() -> Self {
        Self {
            label_idx: 0,
            tmp_idx: 0,
            label_map: HashMap::new(),
            fp_consts: HashMap::new(),
            arena: Vec::new(),
            tac_code: Vec::new(),
            symtab: None,
        }
    }

    fn lower(&mut self, stage: AstStage) -> Self::Air {
        self.symtab = Some(stage.symtab);
        let mut scope: Option<ScopeId> = None;
        self.tac_code.clear();
        for &node_id in &stage.root {
            scope = Some(stage.arena[node_id].scope);
            self.stmt_or_decl(&stage.arena, node_id);
        }
        if let Some(s) = scope {
            let symtab = self.symtab.take().unwrap();
            let top_scope = symtab.upto_top(s);
            let tac_vec = self.symbols(&stage.arena, &symtab, top_scope);
            self.tac_code.extend(tac_vec);
        }
        AirStage {
            tac: TacArena {
                arena: std::mem::take(&mut self.arena),
                top_level: std::mem::take(&mut self.tac_code),
            },
        }
    }
}

impl TacGenerator {
    fn alloc(&mut self, tac: Tac) -> TacId {
        let id = TacId(self.arena.len());
        self.arena.push(tac);
        id
    }

    fn scopes(&self) -> &SymTab {
        self.symtab.as_ref().unwrap()
    }

    fn incr_label_idx(&mut self) -> usize {
        let idx = self.label_idx;
        self.label_idx += 1;
        idx
    }

    fn const_double(&mut self, value: f64) -> TacId {
        let key: Fp64Bits = value.into();
        let idx: usize;

        if !self.fp_consts.contains_key(&key) {
            idx = self.incr_label_idx();
            self.fp_consts.insert(key, idx);
        } else {
            idx = *self.fp_consts.get(&key).unwrap();
        }

        let name = format!(".L{idx}");
        self.alloc(Tac::Operand(Operand::Sym(double_type(), name)))
    }

    fn named_label(&mut self, name: &str) -> TacId {
        if !self.label_map.contains_key(name) {
            let idx = self.incr_label_idx();
            self.label_map.insert(name.to_string(), idx);
        }
        self.alloc(Tac::Meta(Meta::Label(*self.label_map.get(name).unwrap())))
    }

    fn label(&mut self) -> TacId {
        let idx = self.incr_label_idx();
        self.alloc(Tac::Meta(Meta::Label(idx)))
    }

    fn continue_label(&mut self, scope: ScopeId) -> TacId {
        let loop_scope = self.scopes().upto(scope, ScopeKind::Loop).unwrap();
        let label = format!(".continue.{}", loop_scope.0);
        self.named_label(&label)
    }

    fn break_label(&mut self, scope: ScopeId) -> TacId {
        let loop_or_switch = self
            .scopes()
            .upto_any(scope, &[ScopeKind::Loop, ScopeKind::Switch])
            .unwrap();
        let label = format!(".break.{}", loop_or_switch.0);
        self.named_label(&label)
    }

    fn case_label(&mut self, scope: ScopeId, case_idx: usize) -> TacId {
        let switch_scope =
            self.scopes().upto(scope, ScopeKind::Switch).unwrap();
        let label = format!(".case.{}.{}", case_idx, switch_scope.0);
        self.named_label(&label)
    }

    fn tmp_var(&mut self, ty: TypeRef) -> TacId {
        let name = format!(".tmp.{}", self.tmp_idx);
        self.tmp_idx += 1;
        self.alloc(Tac::Operand(Operand::Pseudo(ty, name)))
    }

    fn pseudo_name(symtab: &SymTab, sym: SymId) -> String {
        symtab[sym].name.clone()
    }

    fn binop(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> (TacId, TacId, TacId) {
        let lhs_val = self.expr_and_convert(arena, lhs);
        let rhs_val = self.expr_and_convert(arena, rhs);
        let dst = self.tmp_var(node.ty.clone());

        (lhs_val, rhs_val, dst)
    }

    fn unop(
        &mut self,
        arena: &AstArena,
        ty: &TypeRef,
        lhs: AstId,
    ) -> (TacId, TacId) {
        let src = self.expr_and_convert(arena, lhs);
        let dst = self.tmp_var(ty.clone());

        (src, dst)
    }

    fn pre_incr_or_decr(
        &mut self,
        arena: &AstArena,
        ty: &TypeRef,
        lhs: AstId,
        incr: bool,
    ) -> TacId {
        let loc = self.expr(arena, lhs);
        let val = self.convert(loc, ty);

        let delta = if is_double_type(ty) {
            self.const_double(1.0)
        } else if is_pointer_type(ty) {
            let base_ty = as_base_type(ty);
            self.alloc(Tac::Operand(Operand::Integer {
                ty: long_type(false),
                value: size_of(&base_ty) as u64,
            }))
        } else {
            self.alloc(Tac::Operand(Operand::Integer {
                ty: ty.clone(),
                value: 1,
            }))
        };

        let new_val = self.tmp_var(ty.clone());
        if incr {
            let op = self.alloc(Tac::Op(Op::Add {
                ty: ty.clone(),
                lhs: val,
                rhs: delta,
                dst: new_val,
            }));
            self.emit(op);
        } else {
            let op = self.alloc(Tac::Op(Op::Sub {
                ty: ty.clone(),
                lhs: val,
                rhs: delta,
                dst: new_val,
            }));
            self.emit(op);
        }

        let (is_plain, inner) = match &self.arena[loc.0] {
            Tac::Operand(Operand::PlainOperand(obj)) => (true, *obj),
            Tac::Operand(Operand::DereferencedPtr(ptr)) => (false, *ptr),
            _ => unreachable!(),
        };

        if is_plain {
            let op = self.alloc(Tac::Op(Op::Copy {
                ty: ty.clone(),
                src: new_val,
                dst: inner,
            }));
            self.emit(op);
        } else {
            let op = self.alloc(Tac::Op(Op::Store {
                ty: ty.clone(),
                src: new_val,
                dst: inner,
            }));
            self.emit(op);
        }

        new_val
    }

    fn post_incr_or_decr(
        &mut self,
        arena: &AstArena,
        ty: &TypeRef,
        lhs: AstId,
        incr: bool,
    ) -> TacId {
        let loc = self.expr(arena, lhs);
        let val = self.expr_and_convert(arena, lhs);

        let old_val = self.tmp_var(ty.clone());
        let copy = self.alloc(Tac::Op(Op::Copy {
            ty: ty.clone(),
            src: val,
            dst: old_val,
        }));
        self.emit(copy);

        let delta = if is_double_type(ty) {
            self.const_double(1.0)
        } else if is_pointer_type(ty) {
            let base_ty = as_base_type(ty);
            self.alloc(Tac::Operand(Operand::Integer {
                ty: long_type(false),
                value: size_of(&base_ty) as u64,
            }))
        } else {
            self.alloc(Tac::Operand(Operand::Integer {
                ty: ty.clone(),
                value: 1,
            }))
        };

        let new_val = self.tmp_var(ty.clone());

        if incr {
            let op = self.alloc(Tac::Op(Op::Add {
                ty: ty.clone(),
                lhs: delta,
                rhs: val,
                dst: new_val,
            }));
            self.emit(op);
        } else {
            let op = self.alloc(Tac::Op(Op::Sub {
                ty: ty.clone(),
                lhs: val,
                rhs: delta,
                dst: new_val,
            }));
            self.emit(op);
        }

        let (is_plain, inner) = match &self.arena[loc.0] {
            Tac::Operand(Operand::PlainOperand(obj)) => (true, *obj),
            Tac::Operand(Operand::DereferencedPtr(ptr)) => (false, *ptr),
            _ => unreachable!(),
        };

        if is_plain {
            let op = self.alloc(Tac::Op(Op::Copy {
                ty: ty.clone(),
                src: new_val,
                dst: inner,
            }));
            self.emit(op);
        } else {
            let op = self.alloc(Tac::Op(Op::Store {
                ty: ty.clone(),
                src: new_val,
                dst: inner,
            }));
            self.emit(op);
        }

        old_val
    }

    fn logical_and(
        &mut self,
        arena: &AstArena,
        ty: &TypeRef,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let false_label = self.label();
        let end_label = self.label();
        let lhs_val = self.expr_and_convert(arena, lhs);
        let jump_lhs = self.alloc(Tac::Op(Op::JumpOnZero {
            expr: lhs_val,
            label: false_label,
        }));
        self.emit(jump_lhs);
        let rhs_val = self.expr_and_convert(arena, rhs);
        let jump_rhs = self.alloc(Tac::Op(Op::JumpOnZero {
            expr: rhs_val,
            label: false_label,
        }));
        self.emit(jump_rhs);
        let result = self.tmp_var(ty.clone());
        let one = self.alloc(Tac::Operand(Operand::Integer {
            ty: ty.clone(),
            value: 1,
        }));
        let copy_one = self.alloc(Tac::Op(Op::Copy {
            ty: ty.clone(),
            src: one,
            dst: result,
        }));
        self.emit(copy_one);
        let jump_end = self.alloc(Tac::Op(Op::Jump(end_label)));
        self.emit(jump_end);
        self.emit(false_label);
        let zero = self.alloc(Tac::Operand(Operand::Integer {
            ty: ty.clone(),
            value: 0,
        }));
        let copy_zero = self.alloc(Tac::Op(Op::Copy {
            ty: ty.clone(),
            src: zero,
            dst: result,
        }));
        self.emit(copy_zero);
        self.emit(end_label);
        result
    }

    fn logical_or(
        &mut self,
        arena: &AstArena,
        ty: &TypeRef,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let true_label = self.label();
        let end_label = self.label();
        let lhs_val = self.expr_and_convert(arena, lhs);
        let jump_lhs = self.alloc(Tac::Op(Op::JumpOnNotZero {
            expr: lhs_val,
            label: true_label,
        }));
        self.emit(jump_lhs);
        let rhs_val = self.expr_and_convert(arena, rhs);
        let jump_rhs = self.alloc(Tac::Op(Op::JumpOnNotZero {
            expr: rhs_val,
            label: true_label,
        }));
        self.emit(jump_rhs);
        let result = self.tmp_var(ty.clone());
        let zero = self.alloc(Tac::Operand(Operand::Integer {
            ty: ty.clone(),
            value: 0,
        }));
        let copy_zero = self.alloc(Tac::Op(Op::Copy {
            ty: ty.clone(),
            src: zero,
            dst: result,
        }));
        self.emit(copy_zero);
        let jump_end = self.alloc(Tac::Op(Op::Jump(end_label)));
        self.emit(jump_end);
        self.emit(true_label);
        let one = self.alloc(Tac::Operand(Operand::Integer {
            ty: ty.clone(),
            value: 1,
        }));
        let copy_one = self.alloc(Tac::Op(Op::Copy {
            ty: ty.clone(),
            src: one,
            dst: result,
        }));
        self.emit(copy_one);
        self.emit(end_label);
        result
    }

    fn assign(
        &mut self,
        arena: &AstArena,
        ty: &TypeRef,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let loc = self.expr(arena, lhs);
        let rhs_val = self.expr_and_convert(arena, rhs);

        let (is_plain, inner) = match &self.arena[loc.0] {
            Tac::Operand(Operand::PlainOperand(obj)) => (true, *obj),
            Tac::Operand(Operand::DereferencedPtr(ptr)) => (false, *ptr),
            _ => unreachable!(),
        };

        if is_plain {
            let copy = self.alloc(Tac::Op(Op::Copy {
                ty: ty.clone(),
                src: rhs_val,
                dst: inner,
            }));
            self.emit(copy);

            loc
        } else {
            let store = self.alloc(Tac::Op(Op::Store {
                ty: ty.clone(),
                src: rhs_val,
                dst: inner,
            }));
            self.emit(store);

            self.alloc(Tac::Operand(Operand::PlainOperand(rhs_val)))
        }
    }

    fn compound_assign(&mut self, arena: &AstArena, rhs: AstId) -> TacId {
        let result_ty = arena[rhs].ty.clone();

        let binop_id = match &arena[rhs].kind {
            AstKind::Cast { expr, .. } => *expr,
            _ => rhs,
        };

        let inner = &arena[binop_id];
        let op_ty = inner.ty.clone();

        let (inner_lhs, inner_rhs) = match &inner.kind {
            AstKind::Add { left, right } => (*left, *right),
            AstKind::Subtract { left, right } => (*left, *right),
            AstKind::Multiply { left, right } => (*left, *right),
            AstKind::Divide { left, right } => (*left, *right),
            AstKind::Modulo { left, right } => (*left, *right),
            AstKind::LeftShift { left, right } => (*left, *right),
            AstKind::RightShift { left, right } => (*left, *right),
            AstKind::And { left, right } => (*left, *right),
            AstKind::Or { left, right } => (*left, *right),
            AstKind::Xor { left, right } => (*left, *right),
            _ => unreachable!(),
        };

        let loc_val_ty = type_of(arena, inner_lhs);

        let (loc, val) =
            if let AstKind::Cast { expr, .. } = &arena[inner_lhs].kind {
                let inner_ty = type_of(arena, *expr);
                let loc = self.expr(arena, *expr);
                let val = self.convert(loc, &inner_ty);
                let val = self.cast(&inner_ty, &loc_val_ty, &val);
                (loc, val)
            } else {
                let loc = self.expr(arena, inner_lhs);
                let val = self.convert(loc, &loc_val_ty);
                (loc, val)
            };

        let rhs_val = self.expr_and_convert(arena, inner_rhs);

        let result = self.tmp_var(op_ty.clone());

        match &inner.kind {
            AstKind::Add { .. } => {
                self.emit_add(&op_ty, val, rhs_val, result, &loc_val_ty);
            }
            AstKind::Subtract { .. } => {
                self.emit_sub(&op_ty, val, rhs_val, result, &loc_val_ty);
            }
            AstKind::Multiply { .. } => {
                self.emit_mul(&op_ty, val, rhs_val, result);
            }
            AstKind::Divide { .. } => {
                self.emit_div(&op_ty, val, rhs_val, result);
            }
            AstKind::Modulo { .. } => {
                self.emit_mod(&op_ty, val, rhs_val, result);
            }
            AstKind::LeftShift { .. } => {
                self.emit_shl(val, rhs_val, result);
            }
            AstKind::RightShift { .. } => {
                self.emit_shr(&op_ty, val, rhs_val, result);
            }
            AstKind::And { .. } => {
                self.emit_and(&op_ty, val, rhs_val, result);
            }
            AstKind::Or { .. } => {
                self.emit_or(&op_ty, val, rhs_val, result);
            }
            AstKind::Xor { .. } => {
                self.emit_xor(&op_ty, val, rhs_val, result);
            }
            _ => unreachable!(),
        }

        let result = if binop_id != rhs {
            self.cast(&op_ty, &result_ty, &result)
        } else {
            result
        };

        let (is_plain, inner) = match &self.arena[loc.0] {
            Tac::Operand(Operand::PlainOperand(obj)) => (true, *obj),
            Tac::Operand(Operand::DereferencedPtr(ptr)) => (false, *ptr),
            _ => unreachable!(),
        };

        if is_plain {
            let copy = self.alloc(Tac::Op(Op::Copy {
                ty: result_ty.clone(),
                src: result,
                dst: inner,
            }));
            self.emit(copy);

            loc
        } else {
            let store = self.alloc(Tac::Op(Op::Store {
                ty: result_ty.clone(),
                src: result,
                dst: inner,
            }));
            self.emit(store);

            self.alloc(Tac::Operand(Operand::PlainOperand(result)))
        }
    }

    fn ternary(
        &mut self,
        arena: &AstArena,
        ty: &TypeRef,
        lhs: AstId,
        middle: AstId,
        rhs: AstId,
    ) -> TacId {
        let else_label = self.label();
        let cond_val = self.expr_and_convert(arena, lhs);
        let jump_else = self.alloc(Tac::Op(Op::JumpOnZero {
            expr: cond_val,
            label: else_label,
        }));
        self.emit(jump_else);
        let then_val = self.expr_and_convert(arena, middle);
        let result = self.tmp_var(ty.clone());
        let copy_then = self.alloc(Tac::Op(Op::Copy {
            ty: ty.clone(),
            src: then_val,
            dst: result,
        }));
        self.emit(copy_then);
        let end_label = self.label();
        let jump_end = self.alloc(Tac::Op(Op::Jump(end_label)));
        self.emit(jump_end);
        self.emit(else_label);
        let else_val = self.expr_and_convert(arena, rhs);
        let copy_else = self.alloc(Tac::Op(Op::Copy {
            ty: ty.clone(),
            src: else_val,
            dst: result,
        }));
        self.emit(copy_else);
        self.emit(end_label);
        result
    }

    fn emit_add(
        &mut self,
        ty: &TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
        lhs_ty: &TypeRef,
    ) -> TacId {
        if is_pointer_type(lhs_ty) {
            let base_ty = as_base_type(lhs_ty);
            let elem_size = size_of(&base_ty);
            let op = self.alloc(Tac::Op(Op::AddPtr {
                lhs,
                rhs,
                scale: elem_size,
                dst,
            }));
            self.emit(op);
        } else {
            let op = self.alloc(Tac::Op(Op::Add {
                ty: ty.clone(),
                lhs,
                rhs,
                dst,
            }));
            self.emit(op);
        }
        dst
    }

    fn emit_sub(
        &mut self,
        ty: &TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
        lhs_ty: &TypeRef,
    ) -> TacId {
        if is_pointer_type(lhs_ty) {
            let base_ty = as_base_type(lhs_ty);
            let elem_size = size_of(&base_ty);
            let byte_offset = if elem_size == 1 {
                rhs
            } else {
                let scale = self.alloc(Tac::Operand(Operand::Integer {
                    ty: long_type(false),
                    value: elem_size as u64,
                }));
                let scaled = self.tmp_var(long_type(false));
                let op = self.alloc(Tac::Op(Op::Mul {
                    ty: long_type(false),
                    lhs: rhs,
                    rhs: scale,
                    dst: scaled,
                }));
                self.emit(op);
                scaled
            };
            let neg = self.tmp_var(long_type(false));
            let op = self.alloc(Tac::Op(Op::Neg {
                ty: long_type(false),
                src: byte_offset,
                dst: neg,
            }));
            self.emit(op);
            let op = self.alloc(Tac::Op(Op::AddPtr {
                lhs,
                rhs: neg,
                scale: 1,
                dst,
            }));
            self.emit(op);
        } else {
            let op = self.alloc(Tac::Op(Op::Sub {
                ty: ty.clone(),
                lhs,
                rhs,
                dst,
            }));
            self.emit(op);
        }
        dst
    }

    fn emit_mul(
        &mut self,
        ty: &TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    ) -> TacId {
        let op = self.alloc(Tac::Op(Op::Mul {
            ty: ty.clone(),
            lhs,
            rhs,
            dst,
        }));
        self.emit(op);
        dst
    }

    fn emit_div(
        &mut self,
        ty: &TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    ) -> TacId {
        let op = self.alloc(Tac::Op(Op::Div {
            ty: ty.clone(),
            lhs,
            rhs,
            dst,
        }));
        self.emit(op);
        dst
    }

    fn emit_mod(
        &mut self,
        ty: &TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    ) -> TacId {
        let op = self.alloc(Tac::Op(Op::Mod {
            ty: ty.clone(),
            lhs,
            rhs,
            dst,
        }));
        self.emit(op);
        dst
    }

    fn emit_shl(&mut self, lhs: TacId, rhs: TacId, dst: TacId) -> TacId {
        let op = self.alloc(Tac::Op(Op::LeftShift { lhs, rhs, dst }));
        self.emit(op);
        dst
    }

    fn emit_shr(
        &mut self,
        ty: &TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    ) -> TacId {
        let op = self.alloc(Tac::Op(Op::RightShift {
            ty: ty.clone(),
            lhs,
            rhs,
            dst,
        }));
        self.emit(op);
        dst
    }

    fn emit_and(
        &mut self,
        ty: &TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    ) -> TacId {
        let op = self.alloc(Tac::Op(Op::And {
            ty: ty.clone(),
            lhs,
            rhs,
            dst,
        }));
        self.emit(op);
        dst
    }

    fn emit_or(
        &mut self,
        ty: &TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    ) -> TacId {
        let op = self.alloc(Tac::Op(Op::Or {
            ty: ty.clone(),
            lhs,
            rhs,
            dst,
        }));
        self.emit(op);
        dst
    }

    fn emit_xor(
        &mut self,
        ty: &TypeRef,
        lhs: TacId,
        rhs: TacId,
        dst: TacId,
    ) -> TacId {
        let op = self.alloc(Tac::Op(Op::Xor {
            ty: ty.clone(),
            lhs,
            rhs,
            dst,
        }));
        self.emit(op);
        dst
    }

    fn multiply(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);
        self.emit_mul(&node.ty, lhs_val, rhs_val, dst)
    }

    fn divide(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);
        self.emit_div(&node.ty, lhs_val, rhs_val, dst)
    }

    fn modulo(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);
        self.emit_mod(&node.ty, lhs_val, rhs_val, dst)
    }

    fn add(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);
        let lhs_ty = type_of(arena, lhs);
        self.emit_add(&node.ty, lhs_val, rhs_val, dst, &lhs_ty)
    }

    fn subtract_ptrs(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);

        let lhs_ty = type_of(arena, lhs);
        let base_ty = as_base_type(&lhs_ty);

        let elem_size = self.alloc(Tac::Operand(Operand::Integer {
            ty: long_type(true),
            value: size_of(&base_ty) as u64,
        }));

        let long_ty = long_type(true);
        let diff = self.tmp_var(long_ty.clone());

        let sub = self.alloc(Tac::Op(Op::Sub {
            ty: long_ty.clone(),
            lhs: lhs_val,
            rhs: rhs_val,
            dst: diff,
        }));
        self.emit(sub);

        let div = self.alloc(Tac::Op(Op::Div {
            ty: long_ty.clone(),
            lhs: diff,
            rhs: elem_size,
            dst,
        }));
        self.emit(div);

        dst
    }

    fn subtract(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let lhs_ty = type_of(arena, lhs);
        let rhs_ty = type_of(arena, rhs);

        if is_pointer_type(&lhs_ty) && is_pointer_type(&rhs_ty) {
            return self.subtract_ptrs(arena, node, lhs, rhs);
        }

        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);
        self.emit_sub(&node.ty, lhs_val, rhs_val, dst, &lhs_ty)
    }

    fn shift_left(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);
        self.emit_shl(lhs_val, rhs_val, dst)
    }

    fn shift_right(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);
        self.emit_shr(&node.ty, lhs_val, rhs_val, dst)
    }

    fn and(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);
        self.emit_and(&node.ty, lhs_val, rhs_val, dst)
    }

    fn or(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);
        self.emit_or(&node.ty, lhs_val, rhs_val, dst)
    }

    fn xor(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);
        self.emit_xor(&node.ty, lhs_val, rhs_val, dst)
    }

    fn equal(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let cmp_ty = type_of(arena, lhs);
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);

        let cmp = self.alloc(Tac::Op(Op::Equal {
            ty: cmp_ty,
            lhs: lhs_val,
            rhs: rhs_val,
            dst,
        }));
        self.emit(cmp);

        dst
    }

    fn not_eq(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let cmp_ty = type_of(arena, lhs);
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);

        let cmp = self.alloc(Tac::Op(Op::NotEq {
            ty: cmp_ty,
            lhs: lhs_val,
            rhs: rhs_val,
            dst,
        }));
        self.emit(cmp);

        dst
    }

    fn less(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let cmp_ty = type_of(arena, lhs);
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);

        let cmp = self.alloc(Tac::Op(Op::Less {
            ty: cmp_ty,
            lhs: lhs_val,
            rhs: rhs_val,
            dst,
        }));
        self.emit(cmp);

        dst
    }

    fn less_or_eq(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);

        let cmp_ty = type_of(arena, lhs);
        let cmp = self.alloc(Tac::Op(Op::LessOrEq {
            ty: cmp_ty,
            lhs: lhs_val,
            rhs: rhs_val,
            dst,
        }));
        self.emit(cmp);

        dst
    }

    fn greater(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let cmp_ty = type_of(arena, lhs);
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);

        let cmp = self.alloc(Tac::Op(Op::Greater {
            ty: cmp_ty,
            lhs: lhs_val,
            rhs: rhs_val,
            dst,
        }));
        self.emit(cmp);

        dst
    }

    fn greater_or_eq(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        rhs: AstId,
    ) -> TacId {
        let (lhs_val, rhs_val, dst) = self.binop(arena, node, lhs, rhs);

        let cmp_ty = type_of(arena, lhs);
        let cmp = self.alloc(Tac::Op(Op::GreaterOrEq {
            ty: cmp_ty,
            lhs: lhs_val,
            rhs: rhs_val,
            dst,
        }));
        self.emit(cmp);

        dst
    }

    fn identifier(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        expr_id: AstId,
        name: &Token,
    ) -> TacId {
        let symtab = self.scopes();
        let sym = resolve(symtab, arena, &expr_id).unwrap();
        let name_str = arena.token_str(name);
        let sym_node = sym_as_node(symtab, sym).unwrap();
        let sym_type = arena[sym_node].ty.clone();

        match &arena[sym_node].kind {
            AstKind::Function { name, .. } => {
                self.alloc(Tac::Operand(Operand::Sym(
                    sym_type,
                    arena.token_str(name.as_ref().unwrap()).to_string(),
                )))
            }
            _ => {
                if has_static_storage_duration(symtab, sym) {
                    if has_linkage(symtab, sym) {
                        self.alloc(Tac::Operand(Operand::Sym(
                            sym_type,
                            name_str.to_string(),
                        )))
                    } else {
                        let at_scope = symtab[sym].at_scope.0;
                        let mangled_name = format!("{}.{}", name_str, at_scope);
                        self.alloc(Tac::Operand(Operand::Sym(
                            sym_type,
                            mangled_name,
                        )))
                    }
                } else {
                    self.alloc(Tac::Operand(Operand::Pseudo(
                        node.ty.clone(),
                        Self::pseudo_name(symtab, sym),
                    )))
                }
            }
        }
    }

    fn call(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
        args: &[AstId],
    ) -> TacId {
        let mut arg_vals: Vec<TacId> = vec![];

        for &arg in args {
            arg_vals.push(self.expr_and_convert(arena, arg));
        }

        let result = self.tmp_var(node.ty.clone());
        let func = self.expr_and_convert(arena, lhs);

        let call = self.alloc(Tac::Op(Op::Call {
            ty: node.ty.clone(),
            func,
            args: arg_vals,
            dst: result,
        }));
        self.emit(call);

        result
    }

    fn cast_double_to_ulong(&mut self, ty: &TypeRef, lhs: TacId, dst: TacId) {
        let out_of_range = self.label();
        let end = self.label();
        let upper_bound = self.const_double((i64::MAX as u64 + 1) as f64);

        let too_large = self.tmp_var(ty.clone());
        let compare = self.alloc(Tac::Op(Op::GreaterOrEq {
            ty: ty.clone(),
            lhs,
            rhs: upper_bound,
            dst: too_large,
        }));
        self.emit(compare);

        let jump_out = self.alloc(Tac::Op(Op::JumpOnNotZero {
            expr: too_large,
            label: out_of_range,
        }));
        self.emit(jump_out);

        let convert = self.alloc(Tac::Op(Op::DoubleToUlong {
            ty: ty.clone(),
            src: lhs,
            dst,
        }));
        self.emit(convert);
        let jump_end = self.alloc(Tac::Op(Op::Jump(end)));
        self.emit(jump_end);

        self.emit(out_of_range);
        let reduced = self.tmp_var(ty.clone());
        let sub = self.alloc(Tac::Op(Op::Sub {
            ty: ty.clone(),
            lhs,
            rhs: upper_bound,
            dst: reduced,
        }));
        self.emit(sub);
        let converted = self.tmp_var(ty.clone());
        let convert_reduced = self.alloc(Tac::Op(Op::DoubleToUlong {
            ty: ty.clone(),
            src: reduced,
            dst: converted,
        }));
        self.emit(convert_reduced);
        let biased = self.tmp_var(ty.clone());
        let bias = self.alloc(Tac::Operand(Operand::Integer {
            ty: ty.clone(),
            value: 9223372036854775808,
        }));
        let add = self.alloc(Tac::Op(Op::Add {
            ty: ty.clone(),
            lhs: converted,
            rhs: bias,
            dst: biased,
        }));
        self.emit(add);
        let copy = self.alloc(Tac::Op(Op::Copy {
            ty: ty.clone(),
            src: biased,
            dst,
        }));
        self.emit(copy);
        self.emit(end);
    }

    fn cast_ulong_to_double(&mut self, ty: &TypeRef, lhs: TacId, dst: TacId) {
        let out_of_range = self.label();
        let end = self.label();

        let zero = self.alloc(Tac::Operand(Operand::Integer {
            ty: long_type(false),
            value: 0,
        }));
        let is_neg = self.tmp_var(int_type(true));
        let less = self.alloc(Tac::Op(Op::Less {
            ty: long_type(true),
            lhs,
            rhs: zero,
            dst: is_neg,
        }));
        self.emit(less);

        let jump_out = self.alloc(Tac::Op(Op::JumpOnNotZero {
            expr: is_neg,
            label: out_of_range,
        }));
        self.emit(jump_out);

        let convert = self.alloc(Tac::Op(Op::IntToDouble { src: lhs, dst }));
        self.emit(convert);
        let jump_end = self.alloc(Tac::Op(Op::Jump(end)));
        self.emit(jump_end);

        self.emit(out_of_range);

        let saved = self.tmp_var(long_type(false));

        let save = self.alloc(Tac::Op(Op::Copy {
            ty: long_type(false),
            src: lhs,
            dst: saved,
        }));
        self.emit(save);

        let shifted_src = self.tmp_var(long_type(false));
        let copy_shift = self.alloc(Tac::Op(Op::Copy {
            ty: long_type(false),
            src: saved,
            dst: shifted_src,
        }));
        self.emit(copy_shift);
        let high = self.tmp_var(ty.clone());
        let shift_amount = self.alloc(Tac::Operand(Operand::Integer {
            ty: long_type(false),
            value: 1,
        }));
        let shift = self.alloc(Tac::Op(Op::RightShift {
            ty: long_type(false),
            lhs: shifted_src,
            rhs: shift_amount,
            dst: high,
        }));
        self.emit(shift);

        let low = self.tmp_var(long_type(false));
        let low_mask = self.alloc(Tac::Operand(Operand::Integer {
            ty: long_type(false),
            value: 1,
        }));
        let and = self.alloc(Tac::Op(Op::And {
            ty: long_type(false),
            lhs: saved,
            rhs: low_mask,
            dst: low,
        }));
        self.emit(and);

        let fixed = self.tmp_var(long_type(false));
        let or = self.alloc(Tac::Op(Op::Or {
            ty: long_type(false),
            lhs: high,
            rhs: low,
            dst: fixed,
        }));
        self.emit(or);

        let as_double = self.tmp_var(ty.clone());
        let to_double = self.alloc(Tac::Op(Op::IntToDouble {
            src: fixed,
            dst: as_double,
        }));
        self.emit(to_double);

        let doubled = self.tmp_var(ty.clone());
        let double_op = self.alloc(Tac::Op(Op::Add {
            ty: ty.clone(),
            lhs: as_double,
            rhs: as_double,
            dst: doubled,
        }));
        self.emit(double_op);

        let copy = self.alloc(Tac::Op(Op::Copy {
            ty: ty.clone(),
            src: doubled,
            dst,
        }));
        self.emit(copy);

        self.emit(end);
    }

    fn cast_double_to_uint(&mut self, ty: &TypeRef, lhs: TacId, dst: TacId) {
        let truncated = self.tmp_var(long_type(true));
        let convert = self.alloc(Tac::Op(Op::DoubleToInt {
            ty: long_type(true),
            src: lhs,
            dst: truncated,
        }));
        self.emit(convert);
        let truncate = self.alloc(Tac::Op(Op::Truncate {
            ty: ty.clone(),
            src: truncated,
            dst,
        }));
        self.emit(truncate);
    }

    fn cast(
        &mut self,
        from_ty: &TypeRef,
        to_ty: &TypeRef,
        lhs: &TacId,
    ) -> TacId {
        let src = *lhs;

        if is_match(from_ty, to_ty) {
            return src;
        }

        let name = format!(".tmp.{}", self.tmp_idx);
        self.tmp_idx += 1;
        let dst =
            self.alloc(Tac::Operand(Operand::Pseudo(to_ty.clone(), name)));

        let src_is_double = is_double_type(from_ty);
        let dst_is_double = is_double_type(to_ty);
        let src_is_int = is_int_type(from_ty);
        let dst_is_int = is_int_type(to_ty);

        if src_is_double && dst_is_int {
            if is_signed(to_ty) {
                let op = self.alloc(Tac::Op(Op::DoubleToInt {
                    ty: to_ty.clone(),
                    src,
                    dst,
                }));
                self.emit(op);
            } else {
                if size_of(to_ty) == 8 {
                    self.cast_double_to_ulong(to_ty, src, dst);
                } else {
                    self.cast_double_to_uint(to_ty, src, dst);
                }
            }
        } else if src_is_int && dst_is_double {
            if is_signed(from_ty) {
                let op = self.alloc(Tac::Op(Op::IntToDouble { src, dst }));
                self.emit(op);
            } else {
                if size_of(from_ty) == 8 {
                    self.cast_ulong_to_double(to_ty, src, dst);
                } else {
                    let ext = self.tmp_var(from_ty.clone());
                    let ext_op = self.alloc(Tac::Op(Op::ZeroExt {
                        ty: from_ty.clone(),
                        src,
                        dst: ext,
                    }));
                    self.emit(ext_op);
                    let to_double =
                        self.alloc(Tac::Op(Op::IntToDouble { src: ext, dst }));
                    self.emit(to_double);
                }
            }
        } else if size_of(to_ty) == size_of(from_ty) {
            let op = self.alloc(Tac::Op(Op::Copy {
                ty: to_ty.clone(),
                src,
                dst,
            }));
            self.emit(op);
        } else if size_of(to_ty) < size_of(from_ty) {
            let op = self.alloc(Tac::Op(Op::Truncate {
                ty: to_ty.clone(),
                src,
                dst,
            }));
            self.emit(op);
        } else if is_signed(from_ty) {
            let op = self.alloc(Tac::Op(Op::SignExt {
                ty: to_ty.clone(),
                src,
                dst,
            }));
            self.emit(op);
        } else {
            let op = self.alloc(Tac::Op(Op::ZeroExt {
                ty: to_ty.clone(),
                src,
                dst,
            }));
            self.emit(op);
        }

        dst
    }

    fn complement(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        lhs: AstId,
    ) -> TacId {
        let (src, dst) = self.unop(arena, &node.ty, lhs);

        let op = self.alloc(Tac::Op(Op::Inv { src, dst }));
        self.emit(op);

        dst
    }

    fn negate(&mut self, arena: &AstArena, node: &Ast, lhs: AstId) -> TacId {
        if is_double_type(&node.ty) {
            let (src, dst) = self.unop(arena, &node.ty, lhs);
            let minus_zero = self.const_double(-0.0f64);
            let op = self.alloc(Tac::Op(Op::Xor {
                ty: node.ty.clone(),
                lhs: src,
                rhs: minus_zero,
                dst,
            }));
            self.emit(op);

            dst
        } else {
            let (src, dst) = self.unop(arena, &node.ty, lhs);

            let op = self.alloc(Tac::Op(Op::Neg {
                ty: node.ty.clone(),
                src,
                dst,
            }));
            self.emit(op);

            dst
        }
    }

    fn not(&mut self, arena: &AstArena, node: &Ast, lhs: AstId) -> TacId {
        let (src, dst) = self.unop(arena, &node.ty, lhs);

        let op = self.alloc(Tac::Op(Op::Not {
            ty: node.ty.clone(),
            src,
            dst,
        }));
        self.emit(op);

        dst
    }

    fn deref(&mut self, arena: &AstArena, lhs: AstId) -> TacId {
        let ptr = self.expr_and_convert(arena, lhs);
        self.alloc(Tac::Operand(Operand::DereferencedPtr(ptr)))
    }

    fn addr_of(
        &mut self,
        arena: &AstArena,
        ptr_ty: &TypeRef,
        lhs: AstId,
    ) -> TacId {
        let loc = self.expr(arena, lhs);
        let (is_plain, inner) = match &self.arena[loc.0] {
            Tac::Operand(Operand::PlainOperand(obj)) => (true, *obj),
            Tac::Operand(Operand::DereferencedPtr(ptr)) => (false, *ptr),
            _ => unreachable!(),
        };

        if is_plain {
            let addr = self.tmp_var(ptr_ty.clone());
            let op = self.alloc(Tac::Op(Op::GetAddr {
                ty: ptr_ty.clone(),
                src: inner,
                dst: addr,
            }));
            self.emit(op);
            self.alloc(Tac::Operand(Operand::PlainOperand(addr)))
        } else {
            self.alloc(Tac::Operand(Operand::PlainOperand(inner)))
        }
    }

    fn convert(&mut self, operand: TacId, ty: &TypeRef) -> TacId {
        let plain_val = match &self.arena[operand.0] {
            Tac::Operand(Operand::PlainOperand(value)) => Some(*value),
            _ => None,
        };
        if let Some(value) = plain_val {
            return value;
        }

        if let Tac::Operand(Operand::DereferencedPtr(ptr)) =
            &self.arena[operand.0]
        {
            let ptr = *ptr;
            let dst = self.tmp_var(ty.clone());
            let load = self.alloc(Tac::Op(Op::Load {
                ty: ty.clone(),
                src: ptr,
                dst,
            }));
            self.emit(load);

            return dst;
        }

        unreachable!()
    }

    fn subscript(&mut self, arena: &AstArena, lhs: AstId, rhs: AstId) -> TacId {
        let ptr_ty = type_of(arena, lhs);
        let base_ty = as_base_type(&ptr_ty);
        let elem_size = size_of(&base_ty);

        let ptr = self.expr_and_convert(arena, lhs);
        let index = self.expr_and_convert(arena, rhs);

        let addr = self.tmp_var(ptr_ty);
        let op = self.alloc(Tac::Op(Op::AddPtr {
            lhs: ptr,
            rhs: index,
            scale: elem_size,
            dst: addr,
        }));
        self.emit(op);

        self.alloc(Tac::Operand(Operand::DereferencedPtr(addr)))
    }

    fn expr_and_convert(&mut self, arena: &AstArena, lhs: AstId) -> TacId {
        let operand = self.expr(arena, lhs);
        let ty = type_of(arena, lhs);

        self.convert(operand, &ty)
    }

    fn expr(&mut self, arena: &AstArena, expr_id: AstId) -> TacId {
        let node = &arena[expr_id];
        let ty = type_of(arena, expr_id);
        let result: TacId;
        match &node.kind {
            AstKind::ConstInt(val) => {
                result = self.alloc(Tac::Operand(Operand::Integer {
                    ty: type_of(arena, expr_id),
                    value: *val as u64,
                }));
            }
            AstKind::ConstLong(val) => {
                result = self.alloc(Tac::Operand(Operand::Integer {
                    ty: type_of(arena, expr_id),
                    value: *val as u64,
                }));
            }
            AstKind::ConstUnsignedInt(val) => {
                result = self.alloc(Tac::Operand(Operand::Integer {
                    ty: type_of(arena, expr_id),
                    value: *val as u64,
                }));
            }
            AstKind::ConstUnsignedLong(val) => {
                result = self.alloc(Tac::Operand(Operand::Integer {
                    ty: type_of(arena, expr_id),
                    value: *val,
                }));
            }
            AstKind::ConstDouble(val) => {
                result = self.const_double(*val);
            }
            AstKind::Complement { expr: lhs } => {
                result = self.complement(arena, node, *lhs);
            }
            AstKind::Negate { expr: lhs } => {
                result = self.negate(arena, node, *lhs);
            }
            AstKind::Not { expr: lhs } => {
                result = self.not(arena, node, *lhs);
            }
            AstKind::PostIncr { expr: lhs } => {
                result = self.post_incr_or_decr(arena, &node.ty, *lhs, true);
            }
            AstKind::PostDecr { expr: lhs } => {
                result = self.post_incr_or_decr(arena, &node.ty, *lhs, false);
            }
            AstKind::PreIncr { expr: lhs } => {
                result = self.pre_incr_or_decr(arena, &node.ty, *lhs, true);
            }
            AstKind::PreDecr { expr: lhs } => {
                result = self.pre_incr_or_decr(arena, &node.ty, *lhs, false);
            }
            AstKind::Multiply {
                left: lhs,
                right: rhs,
            } => {
                result = self.multiply(arena, node, *lhs, *rhs);
            }
            AstKind::Divide {
                left: lhs,
                right: rhs,
            } => {
                result = self.divide(arena, node, *lhs, *rhs);
            }
            AstKind::Modulo {
                left: lhs,
                right: rhs,
            } => {
                result = self.modulo(arena, node, *lhs, *rhs);
            }
            AstKind::Add {
                left: lhs,
                right: rhs,
            } => {
                result = self.add(arena, node, *lhs, *rhs);
            }
            AstKind::Subtract {
                left: lhs,
                right: rhs,
            } => {
                result = self.subtract(arena, node, *lhs, *rhs);
            }
            AstKind::LeftShift {
                left: lhs,
                right: rhs,
            } => {
                result = self.shift_left(arena, node, *lhs, *rhs);
            }
            AstKind::RightShift {
                left: lhs,
                right: rhs,
            } => {
                result = self.shift_right(arena, node, *lhs, *rhs);
            }
            AstKind::And {
                left: lhs,
                right: rhs,
            } => {
                result = self.and(arena, node, *lhs, *rhs);
            }
            AstKind::Or {
                left: lhs,
                right: rhs,
            } => {
                result = self.or(arena, node, *lhs, *rhs);
            }
            AstKind::Xor {
                left: lhs,
                right: rhs,
            } => {
                result = self.xor(arena, node, *lhs, *rhs);
            }
            AstKind::Equal {
                left: lhs,
                right: rhs,
            } => {
                result = self.equal(arena, node, *lhs, *rhs);
            }
            AstKind::NotEq {
                left: lhs,
                right: rhs,
            } => {
                result = self.not_eq(arena, node, *lhs, *rhs);
            }
            AstKind::Less {
                left: lhs,
                right: rhs,
            } => {
                result = self.less(arena, node, *lhs, *rhs);
            }
            AstKind::LessOrEq {
                left: lhs,
                right: rhs,
            } => {
                result = self.less_or_eq(arena, node, *lhs, *rhs);
            }
            AstKind::Greater {
                left: lhs,
                right: rhs,
            } => {
                result = self.greater(arena, node, *lhs, *rhs);
            }
            AstKind::GreaterOrEq {
                left: lhs,
                right: rhs,
            } => {
                result = self.greater_or_eq(arena, node, *lhs, *rhs);
            }
            AstKind::LogicAnd {
                left: lhs,
                right: rhs,
            } => {
                result = self.logical_and(arena, &node.ty, *lhs, *rhs);
            }
            AstKind::LogicOr {
                left: lhs,
                right: rhs,
            } => {
                result = self.logical_or(arena, &node.ty, *lhs, *rhs);
            }
            AstKind::Assign {
                left: lhs,
                right: rhs,
            } => {
                return self.assign(arena, &node.ty, *lhs, *rhs);
            }
            AstKind::CompoundAssign {
                left: _,
                right: rhs,
            } => {
                return self.compound_assign(arena, *rhs);
            }
            AstKind::Ternary {
                left: lhs,
                middle,
                right: rhs,
            } => {
                result = self.ternary(arena, &node.ty, *lhs, *middle, *rhs);
            }
            AstKind::Identifier { name, .. } => {
                result = self.identifier(arena, node, expr_id, name);
            }
            AstKind::Call { expr: lhs, args } => {
                result = self.call(arena, node, *lhs, args);
            }

            AstKind::Cast {
                type_spec: _,
                expr: lhs,
            } => {
                let val = self.expr_and_convert(arena, *lhs);
                let from_ty = type_of(arena, *lhs);
                result = self.cast(&from_ty, &node.ty, &val);
            }

            AstKind::Initialiser {
                type_spec: _,
                value: Some(lhs),
            } => {
                result = self.expr_and_convert(arena, *lhs);
            }

            AstKind::Deref { expr: lhs } => {
                return self.deref(arena, *lhs);
            }

            AstKind::AddrOf { expr: lhs } => {
                return self.addr_of(arena, &ty, *lhs);
            }

            AstKind::Subscript {
                left: lhs,
                right: rhs,
            } => {
                return self.subscript(arena, *lhs, *rhs);
            }

            _ => {
                unreachable!()
            }
        }

        self.alloc(Tac::Operand(Operand::PlainOperand(result)))
    }

    fn function(
        &mut self,
        arena: &AstArena,
        name: &Token,
        sym: Option<SymId>,
        params: &[AstId],
        body: &Option<AstId>,
    ) {
        let saved_label_map = std::mem::take(&mut self.label_map);
        let body_start = self.tac_code.len();

        if body.is_some() {
            let mut tac_params: Vec<TacId> = vec![];

            for &param in params {
                if let AstKind::Parameter {
                    name: _,
                    sym: _,
                    type_spec: _,
                } = &arena[param].kind
                {
                    let symtab = self.scopes();
                    let sym = resolve(symtab, arena, &param).unwrap();
                    let ty = arena[param].ty.clone();
                    let param = self.alloc(Tac::Operand(Operand::Pseudo(
                        ty,
                        Self::pseudo_name(symtab, sym),
                    )));
                    tac_params.push(param);
                }
            }

            if let AstKind::Block { body: b } = &arena[body.unwrap()].kind {
                for &stmt in b {
                    self.stmt_or_decl(arena, stmt);
                }
            }

            let zero = self.alloc(Tac::Operand(Operand::Integer {
                ty: int_type(true),
                value: 0,
            }));
            let ret = self.alloc(Tac::Op(Op::Return(int_type(true), zero)));
            self.emit(ret);

            let body_tac: Vec<TacId> =
                self.tac_code.drain(body_start..).collect();

            self.label_map = saved_label_map;

            let global = if let Some(s) = sym {
                let symtab = self.scopes();
                !matches!(symtab[s].storage_class, Some(StorageClass::Static),)
            } else {
                unreachable!()
            };

            let func = self.alloc(Tac::Object(Object::Function {
                name: arena.token_str(name).to_string(),
                global,
                params: tac_params,
                tac: body_tac,
            }));

            self.emit(func);
        }
    }

    fn block(&mut self, arena: &AstArena, body: &[AstId]) {
        for &stmt in body {
            self.stmt_or_decl(arena, stmt);
        }
    }

    fn if_stmt(
        &mut self,
        arena: &AstArena,
        cond: AstId,
        then: AstId,
        otherwise: &Option<AstId>,
    ) {
        let else_label = if otherwise.is_some() {
            Some(self.label())
        } else {
            None
        };
        let end_label = self.label();
        let cond_val = self.expr_and_convert(arena, cond);
        let jump_else = self.alloc(Tac::Op(Op::JumpOnZero {
            expr: cond_val,
            label: if let Some(l) = else_label {
                l
            } else {
                end_label
            },
        }));
        self.emit(jump_else);
        self.stmt_or_decl(arena, then);
        if otherwise.is_some() {
            let jump_end = self.alloc(Tac::Op(Op::Jump(end_label)));
            self.emit(jump_end);
            self.emit(else_label.unwrap());
            self.stmt_or_decl(arena, otherwise.unwrap());
        }
        self.emit(end_label);
    }

    fn switch_stmt(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        cond: AstId,
        body: AstId,
        cases: &[AstId],
    ) {
        let cond_val = self.expr_and_convert(arena, cond);
        let switch_val = self.tmp_var(arena[cond].ty.clone());
        let copy_cond = self.alloc(Tac::Op(Op::Copy {
            ty: node.ty.clone(),
            src: cond_val,
            dst: switch_val,
        }));
        self.emit(copy_cond);
        for &c in cases {
            if let AstKind::Case {
                expr,
                stmt: _,
                idx: _,
            } = &arena[c].kind
            {
                let case_val = self.expr_and_convert(arena, *expr);
                let matches = self.tmp_var(arena[*expr].ty.clone());
                let cmp = self.alloc(Tac::Op(Op::Equal {
                    ty: node.ty.clone(),
                    lhs: switch_val,
                    rhs: case_val,
                    dst: matches,
                }));
                self.emit(cmp);
                let case_label = self.case_label(arena[c].scope, c.0);
                let jump_case = self.alloc(Tac::Op(Op::JumpOnNotZero {
                    expr: matches,
                    label: case_label,
                }));
                self.emit(jump_case);
            }
        }
        let mut has_default = false;
        for &c in cases {
            if let AstKind::Default { .. } = &arena[c].kind {
                let case_label = self.case_label(arena[c].scope, c.0);
                let jump_default = self.alloc(Tac::Op(Op::Jump(case_label)));
                self.emit(jump_default);
                has_default = true;
            }
        }
        let break_label = self.break_label(arena[body].scope);
        if !has_default {
            let jump_break = self.alloc(Tac::Op(Op::Jump(break_label)));
            self.emit(jump_break);
        }
        if !cases.is_empty() {
            self.stmt_or_decl(arena, body);
        }
        self.emit(break_label);
    }

    fn do_while_stmt(&mut self, arena: &AstArena, cond: AstId, body: AstId) {
        let start_label = self.label();
        self.emit(start_label);
        self.stmt_or_decl(arena, body);
        let continue_label = self.continue_label(arena[body].scope);
        self.emit(continue_label);
        let cond_val = self.expr_and_convert(arena, cond);
        let jump_back = self.alloc(Tac::Op(Op::JumpOnNotZero {
            expr: cond_val,
            label: start_label,
        }));
        self.emit(jump_back);
        let break_label = self.break_label(arena[body].scope);
        self.emit(break_label);
    }

    fn while_stmt(&mut self, arena: &AstArena, cond: AstId, body: AstId) {
        let continue_label = self.continue_label(arena[body].scope);
        self.emit(continue_label);
        let cond_val = self.expr_and_convert(arena, cond);
        let break_label = self.break_label(arena[body].scope);
        let jump_break = self.alloc(Tac::Op(Op::JumpOnZero {
            expr: cond_val,
            label: break_label,
        }));
        self.emit(jump_break);
        self.stmt_or_decl(arena, body);
        let jump_back = self.alloc(Tac::Op(Op::Jump(continue_label)));
        self.emit(jump_back);
        self.emit(break_label);
    }

    fn for_stmt(
        &mut self,
        arena: &AstArena,
        init: &Option<AstId>,
        cond: &Option<AstId>,
        post: &Option<AstId>,
        body: AstId,
    ) {
        if let Some(init_id) = init {
            self.stmt_or_decl(arena, *init_id);
        }
        let start_label = self.label();
        self.emit(start_label);
        let break_label = self.break_label(arena[body].scope);
        if let Some(cond_id) = cond {
            let cond_val = self.expr_and_convert(arena, *cond_id);
            let jump_break = self.alloc(Tac::Op(Op::JumpOnZero {
                expr: cond_val,
                label: break_label,
            }));
            self.emit(jump_break);
        }
        self.stmt_or_decl(arena, body);
        let continue_label = self.continue_label(arena[body].scope);
        self.emit(continue_label);
        if let Some(post_id) = post {
            self.stmt_or_decl(arena, *post_id);
        }
        let jump_back = self.alloc(Tac::Op(Op::Jump(start_label)));
        self.emit(jump_back);
        self.emit(break_label);
    }

    fn return_stmt(&mut self, arena: &AstArena, ty: &TypeRef, lhs: AstId) {
        let ret_val = self.expr_and_convert(arena, lhs);
        let ret = self.alloc(Tac::Op(Op::Return(ty.clone(), ret_val)));
        self.emit(ret);
    }

    fn goto_stmt(&mut self, arena: &AstArena, lhs: &Token) {
        let label = self.named_label(arena.token_str(lhs));
        let jump = self.alloc(Tac::Op(Op::Jump(label)));
        self.emit(jump);
    }

    fn labelled_stmt(&mut self, arena: &AstArena, name: &Token, stmt: AstId) {
        let label = self.named_label(arena.token_str(name));
        self.emit(label);
        self.stmt_or_decl(arena, stmt);
    }

    fn case_stmt(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        case_id: AstId,
        stmt: AstId,
    ) {
        let label = self.case_label(node.scope, case_id.0);
        self.emit(label);
        self.stmt_or_decl(arena, stmt);
    }

    fn default_stmt(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        case_id: AstId,
        stmt: AstId,
    ) {
        let label = self.case_label(node.scope, case_id.0);
        self.emit(label);
        self.stmt_or_decl(arena, stmt);
    }

    fn break_stmt(&mut self, node: &Ast) {
        let label = self.break_label(node.scope);
        let jump = self.alloc(Tac::Op(Op::Jump(label)));
        self.emit(jump);
    }

    fn continue_stmt(&mut self, node: &Ast) {
        let label = self.continue_label(node.scope);
        let jump = self.alloc(Tac::Op(Op::Jump(label)));
        self.emit(jump);
    }

    fn compound_elem(
        &mut self,
        arena: &AstArena,
        init_ref: AstId,
        dst: &TacId,
        offset: &mut usize,
    ) {
        let is_compound =
            matches!(arena[init_ref].kind, AstKind::CompoundInitialiser(_));

        if is_compound {
            let elem_total_size = size_of(&arena[init_ref].ty);
            let start_offset = *offset;

            let initialisers = if let AstKind::CompoundInitialiser(ci) =
                &arena[init_ref].kind
            {
                &ci.initialisers
            } else {
                unreachable!()
            };

            for &initialiser in initialisers {
                self.compound_elem(arena, initialiser, dst, offset);
            }

            let scalar_ty = innermost_base_type(&arena[init_ref].ty);
            let scalar_size = size_of(&scalar_ty);
            while *offset < start_offset + elem_total_size {
                let zero_src = if is_double_type(&scalar_ty) {
                    self.const_double(0.0)
                } else {
                    self.alloc(Tac::Operand(Operand::Integer {
                        ty: scalar_ty.clone(),
                        value: 0,
                    }))
                };
                let op = self.alloc(Tac::Op(Op::CopyToOffset {
                    ty: scalar_ty.clone(),
                    src: zero_src,
                    dst: *dst,
                    off: *offset,
                }));
                self.emit(op);
                *offset += scalar_size;
            }
        } else {
            let elem_ty = arena[init_ref].ty.clone();
            let elem_size = size_of(&elem_ty);
            let elem_val = self.expr_and_convert(arena, init_ref);
            let copy = self.alloc(Tac::Op(Op::CopyToOffset {
                ty: elem_ty,
                src: elem_val,
                dst: *dst,
                off: *offset,
            }));
            self.emit(copy);
            *offset += elem_size;
        }
    }

    fn variable(
        &mut self,
        arena: &AstArena,
        node: &Ast,
        ast: AstId,
        sym: Option<SymId>,
        init: &Option<AstId>,
    ) {
        if let Some(s) = sym {
            let symtab = self.scopes();
            if !has_static_storage_duration(symtab, s)
                && symtab.has_parent(node.scope)
                && init.is_some()
            {
                let init_ref = init.unwrap();
                let is_compound = matches!(
                    arena[init_ref].kind,
                    AstKind::CompoundInitialiser(_)
                );

                if is_compound {
                    let symtab = self.scopes();
                    let sym_resolved = resolve(symtab, arena, &ast).unwrap();
                    let record = self.alloc(Tac::Operand(Operand::Pseudo(
                        node.ty.clone(),
                        Self::pseudo_name(symtab, sym_resolved),
                    )));
                    let initialisers = if let AstKind::CompoundInitialiser(ci) =
                        &arena[init_ref].kind
                    {
                        &ci.initialisers
                    } else {
                        unreachable!()
                    };
                    let mut offset = 0usize;
                    for &initialiser in initialisers {
                        self.compound_elem(
                            arena,
                            initialiser,
                            &record,
                            &mut offset,
                        );
                    }

                    let total_size = size_of(&node.ty);
                    if offset < total_size {
                        let elem_ty = innermost_base_type(&node.ty);
                        let elem_size = size_of(&elem_ty);
                        while offset < total_size {
                            let zero_src = if is_double_type(&elem_ty) {
                                self.const_double(0.0)
                            } else {
                                self.alloc(Tac::Operand(Operand::Integer {
                                    ty: elem_ty.clone(),
                                    value: 0,
                                }))
                            };
                            let zero_op =
                                self.alloc(Tac::Op(Op::CopyToOffset {
                                    ty: elem_ty.clone(),
                                    src: zero_src,
                                    dst: record,
                                    off: offset,
                                }));
                            self.emit(zero_op);
                            offset += elem_size;
                        }
                    }
                } else {
                    let name = {
                        let symtab = self.scopes();
                        let sym = resolve(symtab, arena, &ast).unwrap();
                        Self::pseudo_name(symtab, sym)
                    };
                    let init_val = self.expr_and_convert(arena, init_ref);
                    let var = self.alloc(Tac::Operand(Operand::Pseudo(
                        node.ty.clone(),
                        name,
                    )));
                    let copy = self.alloc(Tac::Op(Op::Copy {
                        ty: node.ty.clone(),
                        src: init_val,
                        dst: var,
                    }));
                    self.emit(copy);
                }
            }
        }
    }

    fn stmt_or_decl(&mut self, arena: &AstArena, ast: AstId) {
        let node = &arena[ast];

        match &node.kind {
            AstKind::Function {
                name,
                sym,
                params,
                block,
                type_spec: _,
            } => {
                if let Some(ident) = name {
                    self.function(arena, ident, *sym, params, block)
                }
            }
            AstKind::Block { body } => self.block(arena, body),
            AstKind::If {
                cond,
                then,
                otherwise,
            } => self.if_stmt(arena, *cond, *then, otherwise),
            AstKind::Switch { cond, body, cases } => {
                self.switch_stmt(arena, node, *cond, *body, cases)
            }
            AstKind::DoWhile { cond, body } => {
                self.do_while_stmt(arena, *cond, *body)
            }
            AstKind::While { cond, body } => {
                self.while_stmt(arena, *cond, *body)
            }
            AstKind::For {
                init,
                cond,
                post,
                body,
            } => self.for_stmt(arena, init, cond, post, *body),
            AstKind::Return { expr, .. } => {
                self.return_stmt(arena, &node.ty, *expr)
            }
            AstKind::GoTo { label } => self.goto_stmt(arena, label),
            AstKind::Label { name, stmt } => {
                self.labelled_stmt(arena, name, *stmt)
            }
            AstKind::Case { stmt, .. } => {
                self.case_stmt(arena, node, ast, *stmt)
            }
            AstKind::Default { stmt, .. } => {
                self.default_stmt(arena, node, ast, *stmt)
            }
            AstKind::Break { .. } => self.break_stmt(node),
            AstKind::Continue { .. } => self.continue_stmt(node),
            AstKind::ExprStmt { expr } => {
                _ = self.expr_and_convert(arena, *expr);
            }
            AstKind::Variable {
                name: _,
                sym,
                type_spec: _,
                init,
            } => self.variable(arena, node, ast, *sym, init),
            AstKind::Parameter {
                name: _,
                sym,
                type_spec: _,
            } => self.variable(arena, node, ast, *sym, &None),
            AstKind::EmptyStmt => {}
            _ => {
                unreachable!();
            }
        }
    }

    fn emit(&mut self, ir: TacId) {
        self.tac_code.push(ir);
    }

    fn symbol(
        &mut self,
        arena: &AstArena,
        symtab: &SymTab,
        sym: SymId,
        name: &str,
    ) -> Option<TacId> {
        let s = &symtab[sym];
        let kind = s.kind;
        let storage_class = s.storage_class;

        if let SymKind::Variable = &kind {
            if s.definition == Some(Definition::Concrete) {
                if let Some(node) = sym_as_node(symtab, sym) {
                    let ty = arena[node].ty.clone();
                    if let AstKind::Variable { init, .. } = &arena[node].kind {
                        let init_value =
                            self.static_init(arena, &ty, init.unwrap());
                        let global = !matches!(
                            storage_class,
                            Some(StorageClass::Static)
                        );

                        return Some(self.alloc(Tac::Object(Object::Data(
                            ty.clone(),
                            name.to_string(),
                            global,
                            false,
                            init_value,
                        ))));
                    }
                }
            } else if let Some(node) = sym_as_node(symtab, sym) {
                let ty = arena[node].ty.clone();

                if let AstKind::Variable { .. } = &arena[node].kind {
                    let global =
                        !matches!(storage_class, Some(StorageClass::Static));

                    let zero = self.alloc(Tac::Operand(Operand::Integer {
                        ty: ty.clone(),
                        value: 0,
                    }));
                    let scalar_init = self.alloc(Tac::Object(
                        Object::StaticInit(ty.clone(), zero),
                    ));

                    let init_list: Vec<TacId> = vec![scalar_init];
                    let scalar_ty = innermost_base_type(&ty);

                    let list = self.alloc(Tac::Object(Object::StaticInitList(
                        init_list,
                        size_of(&scalar_ty),
                    )));
                    let data = self.alloc(Tac::Object(Object::Data(
                        ty,
                        name.to_string(),
                        global,
                        false,
                        list,
                    )));

                    return Some(data);
                }
            }
        }

        None
    }

    fn emit_fp_consts(&mut self) -> Vec<TacId> {
        let fp_consts: Vec<_> = self
            .fp_consts
            .iter()
            .map(|(bits, idx)| (bits.0, *idx))
            .collect();
        fp_consts
            .iter()
            .map(|(bits, idx)| {
                let name = format!(".L{}", idx);
                let ty = double_type();
                let value = self.alloc(Tac::Operand(Operand::Double(
                    f64::from_bits(*bits),
                )));
                let scalar_init = self
                    .alloc(Tac::Object(Object::StaticInit(ty.clone(), value)));
                let list = self.alloc(Tac::Object(Object::StaticInitList(
                    vec![scalar_init],
                    size_of(&ty),
                )));
                self.alloc(Tac::Object(Object::Data(
                    ty, name, false, true, list,
                )))
            })
            .collect()
    }

    fn symbols(
        &mut self,
        arena: &AstArena,
        symtab: &SymTab,
        scope: ScopeId,
    ) -> Vec<TacId> {
        assert!(!symtab.has_parent(scope));
        let mut code = vec![];

        let defs = symtab.defs();
        for &def in &defs {
            if has_static_storage_duration(symtab, def) {
                let name = symtab[def].name.clone();
                if let Some(tac) = self.symbol(arena, symtab, def, &name) {
                    code.push(tac);
                }
            }
        }

        code.extend(self.emit_fp_consts());

        code
    }

    fn static_init(
        &mut self,
        arena: &AstArena,
        ty: &TypeRef,
        init: AstId,
    ) -> TacId {
        let mut init_list: Vec<TacId> = Vec::new();
        let value: u64;

        match &arena[init].kind {
            AstKind::CompoundInitialiser(_) => {
                let initialisers = if let AstKind::CompoundInitialiser(ci) =
                    &arena[init].kind
                {
                    &ci.initialisers
                } else {
                    unreachable!()
                };

                let mut list: Vec<TacId> = Vec::new();

                let elem_ty = as_base_type(ty);
                let elem_size = size_of(&elem_ty);

                let scalar_ty = innermost_base_type(&elem_ty);
                let scalar_size = size_of(&scalar_ty);

                for &initialiser in initialisers {
                    let before = list.len();
                    let elem_init =
                        self.static_init(arena, &elem_ty, initialiser);
                    if let Tac::Object(Object::StaticInitList(
                        items,
                        _alignment,
                    )) = &self.arena[elem_init.0]
                    {
                        list.extend(items.iter().copied());
                    } else {
                        list.push(elem_init);
                    }

                    let emitted: usize = list[before..]
                        .iter()
                        .map(|item| {
                            if let Tac::Object(Object::StaticInit(t, _)) =
                                &self.arena[item.0]
                            {
                                size_of(t)
                            } else {
                                0
                            }
                        })
                        .sum();
                    let gap = elem_size.saturating_sub(emitted);
                    for _ in 0..(gap / scalar_size) {
                        let zero = self.alloc(Tac::Operand(Operand::Integer {
                            ty: scalar_ty.clone(),
                            value: 0,
                        }));
                        list.push(self.alloc(Tac::Object(Object::StaticInit(
                            scalar_ty.clone(),
                            zero,
                        ))));
                    }
                }

                return self.alloc(Tac::Object(Object::StaticInitList(
                    list,
                    scalar_size,
                )));
            }
            AstKind::Initialiser {
                type_spec: _,
                value: Some(expr),
            } => {
                return self.static_init(arena, ty, *expr);
            }
            AstKind::Cast { expr, .. } => {
                return self.static_init(arena, ty, *expr);
            }
            AstKind::ConstInt(v) => {
                if is_double_type(ty) {
                    value = ((*v as u64) as f64).to_bits();
                } else {
                    value = *v as u64;
                }
            }
            AstKind::ConstLong(v) => {
                if is_double_type(ty) {
                    value = ((*v as u64) as f64).to_bits();
                } else {
                    value = *v as u64;
                }
            }
            AstKind::ConstUnsignedInt(v) => {
                if is_double_type(ty) {
                    value = ((*v as u64) as f64).to_bits();
                } else {
                    value = *v as u64;
                }
            }
            AstKind::ConstUnsignedLong(v) => {
                if is_double_type(ty) {
                    value = (*v as f64).to_bits();
                } else {
                    value = *v;
                }
            }
            AstKind::ConstDouble(v) => {
                if is_double_type(ty) {
                    value = v.to_bits();
                } else if is_signed(ty) {
                    if size_of(ty) == 8 {
                        value = *v as i64 as u64;
                    } else {
                        value = *v as i32 as u64;
                    }
                } else if size_of(ty) == 8 {
                    value = *v as u64;
                } else {
                    value = *v as u32 as u64;
                }
            }
            _ => {
                unreachable!()
            }
        }

        let value_operand = if is_double_type(ty) {
            self.alloc(Tac::Operand(Operand::Double(f64::from_bits(value))))
        } else {
            self.alloc(Tac::Operand(Operand::Integer {
                ty: ty.clone(),
                value,
            }))
        };
        let scalar_init = self
            .alloc(Tac::Object(Object::StaticInit(ty.clone(), value_operand)));

        init_list.push(scalar_init);

        self.alloc(Tac::Object(Object::StaticInitList(init_list, size_of(ty))))
    }
}
