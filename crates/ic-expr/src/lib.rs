// Copyright 2024 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Op {
    Not,
    And,
    Or,
    Eq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    NotEq,
    BitNot,
    BitAnd,
    BitOr,
    BitXor,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug)]
pub enum Expr<T> {
    Lit(T),
    Unary(Box<Unary<T>>),
    Binary(Box<Binary<T>>),
    Ternary(Box<Ternary<T>>),
}

#[derive(Debug)]
pub struct Unary<T> {
    pub op: Op,
    pub expr: Expr<T>,
}

#[derive(Debug)]
pub struct Binary<T> {
    pub lhs: Expr<T>,
    pub op: Op,
    pub rhs: Expr<T>,
}

#[derive(Debug)]
pub struct Ternary<T> {
    pub cond: Expr<T>,
    pub then: Expr<T>,
    pub els: Expr<T>,
}

pub trait Handler<T, R> {
    fn eval_lit(lit: &T) -> R;
}

// I guess we could pass a trait or callback that deals with Lit(T)?
// The issue here is that we need a notion of types.
pub fn eval<T, H, R>(_expr: &Expr<T>, _handler: H) -> R
where
    H: Handler<T, R>,
{
    todo!()
}
