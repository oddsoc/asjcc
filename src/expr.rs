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
use crate::symtab::*;
use crate::types::*;
use crate::walk::ast::AstVisitor;

fn as_const_int(arena: &mut AstArena, n: i32, scope: ScopeId) -> AstId {
    arena.alloc(AstKind::ConstInt(n), Some(int_type(true)), scope)
}

fn as_const_unsigned_int(
    arena: &mut AstArena,
    n: u32,
    scope: ScopeId,
) -> AstId {
    arena.alloc(AstKind::ConstUnsignedInt(n), Some(int_type(false)), scope)
}

fn as_const_long(arena: &mut AstArena, n: i64, scope: ScopeId) -> AstId {
    arena.alloc(AstKind::ConstLong(n), Some(long_type(true)), scope)
}

fn as_const_unsigned_long(
    arena: &mut AstArena,
    n: u64,
    scope: ScopeId,
) -> AstId {
    arena.alloc(AstKind::ConstUnsignedLong(n), Some(long_type(false)), scope)
}

fn fold_node(arena: &mut AstArena, id: AstId) -> AstId {
    let scope = arena[id].scope;
    let kind = arena[id].kind.clone();

    match &kind {
        AstKind::LogicAnd { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *lhs != 0 && *rhs != 0 { 1 } else { 0 };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *lhs != 0 && *rhs != 0 { 1 } else { 0 };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = if *lhs != 0 && *rhs != 0 { 1 } else { 0 };
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = if *lhs != 0 && *rhs != 0 { 1 } else { 0 };
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::LogicOr { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *lhs != 0 || *rhs != 0 { 1 } else { 0 };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *lhs != 0 || *rhs != 0 { 1 } else { 0 };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = if *lhs != 0 || *rhs != 0 { 1 } else { 0 };
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = if *lhs != 0 || *rhs != 0 { 1 } else { 0 };
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::Add { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = (*lhs).wrapping_add(*rhs);
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = (*lhs).wrapping_add(*rhs);
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = (*lhs).wrapping_add(*rhs);
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = (*lhs).wrapping_add(*rhs);
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::Subtract { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = (*lhs).wrapping_sub(*rhs);
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = (*lhs).wrapping_sub(*rhs);
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = (*lhs).wrapping_sub(*rhs);
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = (*lhs).wrapping_sub(*rhs);
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::Multiply { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = (*lhs).wrapping_mul(*rhs);
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = (*lhs).wrapping_mul(*rhs);
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = (*lhs).wrapping_mul(*rhs);
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = (*lhs).wrapping_mul(*rhs);
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::Divide { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *rhs == 0 { 0 } else { *lhs / *rhs };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *rhs == 0 { 0 } else { *lhs / *rhs };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = lhs.checked_div(*rhs).unwrap_or(0);
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = lhs.checked_div(*rhs).unwrap_or(0);
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::Modulo { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *rhs == 0 { 0 } else { *lhs % *rhs };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *rhs == 0 { 0 } else { *lhs % *rhs };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = if *rhs == 0 { 0 } else { *lhs % *rhs };
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = if *rhs == 0 { 0 } else { *lhs % *rhs };
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::And { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = *lhs & *rhs;
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = *lhs & *rhs;
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = *lhs & *rhs;
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = *lhs & *rhs;
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::Or { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = *lhs | *rhs;
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = *lhs | *rhs;
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = *lhs | *rhs;
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = *lhs | *rhs;
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::LeftShift { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = (*lhs).wrapping_shl(*rhs as u32);
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = (*lhs).wrapping_shl(*rhs as u32);
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = (*lhs).wrapping_shl(*rhs);
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = (*lhs).wrapping_shl(*rhs as u32);
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::RightShift { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *rhs >= 0 { *lhs >> *rhs } else { 0 };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *rhs >= 0 { *lhs >> *rhs } else { 0 };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = *lhs >> *rhs;
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = *lhs >> (*rhs as u32);
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::Xor { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = *lhs ^ *rhs;
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = *lhs ^ *rhs;
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = *lhs ^ *rhs;
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = *lhs ^ *rhs;
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::Equal { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *lhs == *rhs { 1 } else { 0 };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *lhs == *rhs { 1 } else { 0 };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = if *lhs == *rhs { 1 } else { 0 };
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = if *lhs == *rhs { 1 } else { 0 };
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::NotEq { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *lhs != *rhs { 1 } else { 0 };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *lhs != *rhs { 1 } else { 0 };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = if *lhs != *rhs { 1 } else { 0 };
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = if *lhs != *rhs { 1 } else { 0 };
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::Less { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *lhs < *rhs { 1 } else { 0 };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *lhs < *rhs { 1 } else { 0 };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = if *lhs < *rhs { 1 } else { 0 };
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = if *lhs < *rhs { 1 } else { 0 };
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::LessOrEq { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *lhs <= *rhs { 1 } else { 0 };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *lhs <= *rhs { 1 } else { 0 };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = if *lhs <= *rhs { 1 } else { 0 };
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = if *lhs <= *rhs { 1 } else { 0 };
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::Greater { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *lhs > *rhs { 1 } else { 0 };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *lhs > *rhs { 1 } else { 0 };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = if *lhs > *rhs { 1 } else { 0 };
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = if *lhs > *rhs { 1 } else { 0 };
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }
        AstKind::GreaterOrEq { left, right } => {
            match (&arena[*left].kind, &arena[*right].kind) {
                (AstKind::ConstInt(lhs), AstKind::ConstInt(rhs)) => {
                    let res = if *lhs >= *rhs { 1 } else { 0 };
                    as_const_int(arena, res, scope)
                }
                (AstKind::ConstLong(lhs), AstKind::ConstLong(rhs)) => {
                    let res = if *lhs >= *rhs { 1 } else { 0 };
                    as_const_long(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedInt(lhs),
                    AstKind::ConstUnsignedInt(rhs),
                ) => {
                    let res = if *lhs >= *rhs { 1 } else { 0 };
                    as_const_unsigned_int(arena, res, scope)
                }
                (
                    AstKind::ConstUnsignedLong(lhs),
                    AstKind::ConstUnsignedLong(rhs),
                ) => {
                    let res = if *lhs >= *rhs { 1 } else { 0 };
                    as_const_unsigned_long(arena, res, scope)
                }
                _ => id,
            }
        }

        AstKind::Ternary {
            left,
            middle,
            right,
        } => match &arena[*left].kind {
            AstKind::ConstInt(n) => {
                if *n != 0 {
                    *middle
                } else {
                    *right
                }
            }
            AstKind::ConstLong(n) => {
                if *n != 0 {
                    *middle
                } else {
                    *right
                }
            }
            _ => id,
        },

        AstKind::Not { expr: inner } => match &arena[*inner].kind {
            AstKind::ConstInt(n) => {
                if *n != 0 {
                    as_const_int(arena, 0, scope)
                } else {
                    as_const_int(arena, 1, scope)
                }
            }
            AstKind::ConstLong(n) => {
                if *n != 0 {
                    as_const_int(arena, 0, scope)
                } else {
                    as_const_int(arena, 1, scope)
                }
            }
            AstKind::ConstUnsignedInt(n) => {
                if *n != 0 {
                    as_const_unsigned_int(arena, 0, scope)
                } else {
                    as_const_unsigned_int(arena, 1, scope)
                }
            }
            AstKind::ConstUnsignedLong(n) => {
                if *n != 0 {
                    as_const_unsigned_long(arena, 0, scope)
                } else {
                    as_const_unsigned_long(arena, 1, scope)
                }
            }
            _ => id,
        },
        AstKind::Negate { expr: inner } => match &arena[*inner].kind {
            AstKind::ConstInt(n) => as_const_int(arena, -(*n), scope),
            AstKind::ConstLong(n) => as_const_long(arena, -(*n), scope),
            AstKind::ConstUnsignedInt(n) => {
                as_const_int(arena, -(*n as i32), scope)
            }
            AstKind::ConstUnsignedLong(n) => {
                as_const_long(arena, -(*n as i64), scope)
            }
            _ => id,
        },
        AstKind::Complement { expr: inner } => match &arena[*inner].kind {
            AstKind::ConstInt(n) => as_const_int(arena, !(*n), scope),
            AstKind::ConstLong(n) => as_const_long(arena, !(*n), scope),
            AstKind::ConstUnsignedInt(n) => {
                as_const_int(arena, !(*n as i32), scope)
            }
            AstKind::ConstUnsignedLong(n) => {
                as_const_long(arena, !(*n as i64), scope)
            }
            _ => id,
        },
        AstKind::Cast { expr: inner, .. } => {
            let ty = arena[id].ty.clone();
            let signed = is_signed(&ty);

            match &arena[*inner].kind {
                AstKind::ConstInt(n) => {
                    if ty.kind == TypeKind::Int {
                        as_const_int(arena, *n, scope)
                    } else if ty.kind == TypeKind::Long {
                        as_const_long(arena, *n as i64, scope)
                    } else if !signed && ty.kind == TypeKind::Int {
                        as_const_unsigned_int(arena, *n as u32, scope)
                    } else if !signed && ty.kind == TypeKind::Long {
                        as_const_unsigned_long(arena, *n as u64, scope)
                    } else {
                        id
                    }
                }
                AstKind::ConstLong(n) => {
                    if signed && ty.kind == TypeKind::Int {
                        as_const_int(arena, *n as i32, scope)
                    } else if signed && ty.kind == TypeKind::Long {
                        as_const_long(arena, *n, scope)
                    } else if !signed && ty.kind == TypeKind::Int {
                        as_const_unsigned_int(arena, *n as u32, scope)
                    } else if !signed && ty.kind == TypeKind::Long {
                        as_const_unsigned_long(arena, *n as u64, scope)
                    } else {
                        id
                    }
                }
                AstKind::ConstUnsignedInt(n) => {
                    if signed && ty.kind == TypeKind::Int {
                        as_const_int(arena, *n as i32, scope)
                    } else if signed && ty.kind == TypeKind::Long {
                        as_const_long(arena, *n as i64, scope)
                    } else if !signed && ty.kind == TypeKind::Int {
                        as_const_unsigned_int(arena, *n, scope)
                    } else if !signed && ty.kind == TypeKind::Long {
                        as_const_unsigned_long(arena, *n as u64, scope)
                    } else {
                        id
                    }
                }
                AstKind::ConstUnsignedLong(n) => {
                    if signed && ty.kind == TypeKind::Int {
                        as_const_int(arena, *n as i32, scope)
                    } else if signed && ty.kind == TypeKind::Long {
                        as_const_long(arena, *n as i64, scope)
                    } else if !signed && ty.kind == TypeKind::Int {
                        as_const_unsigned_int(arena, *n as u32, scope)
                    } else if !signed && ty.kind == TypeKind::Long {
                        as_const_unsigned_long(arena, *n, scope)
                    } else {
                        id
                    }
                }
                _ => id,
            }
        }
        _ => {
            unreachable!();
        }
    }
}

pub fn is_callable(arena: &AstArena, symtab: &SymTab, id: AstId) -> bool {
    if let AstKind::Identifier { .. } = &arena[id].kind
        && let Some(sym) = resolve(symtab, arena, &id)
        && let Some(node) = sym_as_node(symtab, sym)
    {
        match &arena[node].kind {
            AstKind::Function { .. } => return true,
            _ => return false,
        }
    }

    false
}

pub fn is_lvalue(arena: &AstArena, id: AstId) -> bool {
    matches!(
        &arena[id].kind,
        AstKind::Identifier { .. }
            | AstKind::Deref { .. }
            | AstKind::Subscript { .. }
    )
}

pub fn is_const_unsigned_int_expr(arena: &AstArena, id: AstId) -> bool {
    match &arena[id].kind {
        AstKind::Initialiser {
            type_spec: _,
            value,
        } => {
            if let Some(subexpr) = value {
                is_const_unsigned_int_expr(arena, *subexpr)
            } else {
                true
            }
        }
        AstKind::Cast { expr: subexpr, .. } => {
            is_const_unsigned_int_expr(arena, *subexpr)
        }
        AstKind::ConstUnsignedInt(_) | AstKind::ConstUnsignedLong(_) => true,
        _ => false,
    }
}

pub fn is_const_int_expr(arena: &AstArena, id: AstId) -> bool {
    match &arena[id].kind {
        AstKind::Initialiser {
            type_spec: _,
            value,
        } => {
            if let Some(subexpr) = value {
                is_const_int_expr(arena, *subexpr)
            } else {
                true
            }
        }
        AstKind::Cast { expr: subexpr, .. } => {
            is_const_int_expr(arena, *subexpr)
        }
        AstKind::ConstInt(_) | AstKind::ConstLong(_) => true,
        _ => false,
    }
}

pub fn is_const_double_expr(arena: &AstArena, id: AstId) -> bool {
    match &arena[id].kind {
        AstKind::Initialiser {
            type_spec: _,
            value,
        } => {
            if let Some(subexpr) = value {
                is_const_double_expr(arena, *subexpr)
            } else {
                true
            }
        }
        AstKind::Cast { expr: subexpr, .. } => {
            is_const_double_expr(arena, *subexpr)
        }
        AstKind::ConstDouble(_) => true,
        _ => false,
    }
}

pub fn is_const_expr(arena: &AstArena, id: AstId) -> bool {
    match &arena[id].kind {
        AstKind::CompoundInitialiser(initialiser) => {
            for elem_initialiser in &initialiser.initialisers {
                if !is_const_expr(arena, *elem_initialiser) {
                    return false;
                }
            }

            true
        }
        _ => {
            is_const_int_expr(arena, id)
                || is_const_unsigned_int_expr(arena, id)
                || is_const_double_expr(arena, id)
        }
    }
}

pub fn is_null_pointer_const_expr(arena: &AstArena, id: AstId) -> bool {
    if is_const_int_expr(arena, id) && const_int_value(arena, id) == 0 {
        true
    } else {
        is_const_unsigned_int_expr(arena, id)
            && const_unsigned_int_value(arena, id) == 0
    }
}

pub fn const_int_value(arena: &AstArena, id: AstId) -> i64 {
    assert!(is_const_int_expr(arena, id));
    match &arena[id].kind {
        AstKind::Initialiser {
            type_spec: _,
            value: Some(subexpr),
        } => const_int_value(arena, *subexpr),
        AstKind::Cast { expr: subexpr, .. } => const_int_value(arena, *subexpr),
        AstKind::ConstInt(value) => *value as i64,
        AstKind::ConstLong(value) => *value,
        _ => {
            unreachable!()
        }
    }
}

pub fn const_unsigned_int_value(arena: &AstArena, id: AstId) -> u64 {
    assert!(is_const_unsigned_int_expr(arena, id));
    match &arena[id].kind {
        AstKind::Initialiser {
            type_spec: _,
            value,
        } => {
            if let Some(subexpr) = value {
                const_unsigned_int_value(arena, *subexpr)
            } else {
                unreachable!()
            }
        }
        AstKind::Cast { expr: subexpr, .. } => {
            const_unsigned_int_value(arena, *subexpr)
        }
        AstKind::ConstUnsignedInt(value) => *value as u64,
        AstKind::ConstUnsignedLong(value) => *value,
        _ => unreachable!(),
    }
}

#[allow(unused)]
pub fn const_double_value(arena: &AstArena, id: AstId) -> f64 {
    assert!(is_const_double_expr(arena, id));
    match &arena[id].kind {
        AstKind::Initialiser {
            type_spec: _,
            value,
        } => {
            if let Some(subexpr) = value {
                const_double_value(arena, *subexpr)
            } else {
                unreachable!()
            }
        }
        AstKind::Cast { expr: subexpr, .. } => {
            const_double_value(arena, *subexpr)
        }
        AstKind::ConstDouble(value) => *value,
        _ => unreachable!(),
    }
}

pub struct Folder;

impl Folder {
    pub fn new() -> Folder {
        Folder
    }

    fn fold(
        &mut self,
        stage: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        let folded = fold_node(&mut stage.arena, id);
        stage.arena[id] = stage.arena[folded].clone();
        Ok(())
    }
}

impl AstVisitor for Folder {
    fn visit_logic_and(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_logic_or(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_add(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_subtract(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_multiply(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_divide(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_modulo(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_left_shift(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_right_shift(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_and(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_or(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_xor(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_equal(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_not_eq(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_less(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_less_or_eq(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_greater(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_greater_or_eq(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_ternary(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_not(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_negate(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_complement(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }

    fn visit_cast(
        &mut self,
        s: &mut AstStage<'_>,
        id: AstId,
    ) -> Result<(), Error> {
        self.fold(s, id)
    }
}
