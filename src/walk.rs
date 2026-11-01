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

pub mod air;
pub mod ast;
pub mod mir;

use crate::errors::Error;

pub trait Tree {
    type Id: Copy;

    fn root(&self, index: usize) -> Option<Self::Id>;

    fn child(&self, id: Self::Id, index: usize) -> Option<Self::Id>;
}

pub trait Pass<T: Tree> {
    fn enter(&mut self, _tree: &mut T, _id: T::Id) -> Result<(), Error> {
        Ok(())
    }

    fn exit(&mut self, _tree: &mut T, _id: T::Id) -> Result<(), Error> {
        Ok(())
    }

    fn walk(&mut self, tree: &mut T, id: T::Id) -> Result<(), Error> {
        self.enter(tree, id)?;

        let mut index = 0;
        while let Some(child) = tree.child(id, index) {
            self.walk(tree, child)?;
            index += 1;
        }

        self.exit(tree, id)
    }

    fn walk_roots(&mut self, tree: &mut T) -> Result<(), Error> {
        let mut index = 0;
        while let Some(root) = tree.root(index) {
            self.walk(tree, root)?;
            index += 1;
        }

        Ok(())
    }
}
