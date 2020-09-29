
// iterative (not recursive) post-order traversal with lookup table for
//   nodes is based on:
//   https://sachanganesh.com/programming/graph-tree-traversals-in-rust/
pub mod tree {
    use std::collections::HashSet;

    pub type NodeIndex = usize;
    
    pub enum ChildSide {
        Left,
        Right
    }

    pub struct TreeNode<T> {
        pub value: T,
        left: Option<NodeIndex>,
        right: Option<NodeIndex>,
        parent: Option<NodeIndex>
    }

    impl<T> TreeNode<T> {
        pub fn new(value: T) -> Self {
            TreeNode {
                value,
                left: None,
                right: None,
                parent: None
            }
        }

        pub fn has_left(&self) -> bool {
            self.left.is_some()
        }

        pub fn has_right(&self) -> bool {
            self.right.is_some()
        }

        pub fn get_left(&self) -> Option<NodeIndex> {
            self.left
        }

        pub fn get_right(&self) -> Option<NodeIndex> {
            self.right
        }
    }

    pub struct Tree<T> {
        index: Vec<Option<TreeNode<T>>>,
        root: Option<NodeIndex>
    }

    impl<T> Tree<T> {
        pub fn new() -> Self {
            Tree {
                index: Vec::new(),
                root: None
            }
        }

        pub fn set_root(&mut self, root: Option<NodeIndex>) {
            self.root = root;
        }

        pub fn has_root(&self) -> bool {
            self.root.is_some() && self.node_at(self.root.unwrap()).is_some()
        }

        pub fn get_root(&self) -> Option<NodeIndex> {
            self.root
        }

        pub fn matches_root(&self, index_loc: NodeIndex) -> bool {
            match self.root {
                Some(i) => { i == index_loc },
                None => false
            }
        }

        pub fn add_node(&mut self, node: TreeNode<T>) -> NodeIndex {
            let index_loc = self.index.len();
            self.index.push(Some(node));
            return index_loc
        }

        pub fn add_node_with_children(&mut self, node: TreeNode<T>, left_child: Option<NodeIndex>, right_child: Option<NodeIndex>) -> NodeIndex {
            let index_loc = self.add_node(node);
            if left_child.is_some() {
                if self.set_node_child(index_loc, left_child, ChildSide::Left).is_err() {
                    // eat this error?
                }
            }
            if right_child.is_some() {
                if self.set_node_child(index_loc, right_child, ChildSide::Right).is_err() {
                    // eat this error?
                }
            }
            return index_loc
        }
    
        pub fn remove_node_at(&mut self, index_loc: NodeIndex) -> Option<TreeNode<T>> {
            if let Some(node) = self.index.get_mut(index_loc) {
                node.take()
            } else {
                None
            }
        }

        pub fn has_node_at(&self, index_loc: NodeIndex) -> bool {
            self.index.get(index_loc).is_some()
        }
    
        pub fn node_at(&self, index_loc: NodeIndex) -> Option<&TreeNode<T>> {
            return if let Some(node) = self.index.get(index_loc) {
                node.as_ref()
            } else {
                None
            }
        }
    
        pub fn node_at_mut(&mut self, index_loc: NodeIndex) -> Option<&mut TreeNode<T>> {
            return if let Some(node) = self.index.get_mut(index_loc) {
                node.as_mut()
            } else {
                None
            }
        }

        pub fn set_node_child(&mut self, node_loc: NodeIndex,
                child_loc: Option<NodeIndex>, child_side: ChildSide,) -> Result<(),()> {
            
            // if the node's child is already set, unset that node's parent
            //   (may not be needed, since there's no way to traverse to the
            //   soon-to-be-removed child)
            let prev_child_loc = match self.node_at(node_loc) {
                Some(parent) => {
                    match child_side {
                        ChildSide::Left => parent.left,
                        ChildSide::Right => parent.right
                    }
                },
                None => { return Err(()); }
            };
            match prev_child_loc {
                Some(prev_child) => {
                    match self.node_at_mut(prev_child) {
                        Some(child) => {
                            child.parent = None;
                        },
                        None => ()
                    }
                },
                None => ()
            }

            // set the new child
            match self.node_at_mut(node_loc) {
                Some(parent) => {
                    match child_side {
                        ChildSide::Left => { parent.left = child_loc; }
                        ChildSide::Right => { parent.right = child_loc; }
                    }
                    
                },
                None => { return Err(()); }
            }

            // set the new child node's parent
            match child_loc {
                Some(child_loc) => {
                    match self.node_at_mut(child_loc) {
                        Some(child) => {
                            child.parent = Some(node_loc);
                        },
                        None => { return Err(()); }
                    }
                },
                None => ()
            }
            return Ok(());
        }

        pub fn get_node_parent(&self, node_loc: NodeIndex) -> Option<NodeIndex> {
            match self.node_at(node_loc) {
                Some(child) => { return child.parent; },
                None => { return None; }
            };
        }

        // TODO: don't we need to replace the "from node"'s child with a None?
        //   otherwise the moved child will now have two parents
        pub fn set_node_child_from_node_child(&mut self,
                set_to_loc: NodeIndex, set_to_side: ChildSide,
                set_from_loc: NodeIndex, set_from_side: ChildSide) -> Result<(),()> {
            let new_child = match self.node_at(set_from_loc) {
                Some(f) => {
                    match set_from_side {
                        ChildSide::Left => f.left,
                        ChildSide::Right => f.right
                    }
                },
                None => { return Err(()); }
            };

            let to_node = self.node_at_mut(set_to_loc);
            match to_node {
                Some(t) => {
                    match set_to_side {
                        ChildSide::Left => {
                            t.left = new_child;
                        },
                        ChildSide::Right => {
                            t.right = new_child;
                        }
                    };
                    return Ok(());
                } ,
                None => Err(())
            }
        }

        pub fn replace_root_with_node(&mut self,
                new_root_loc: NodeIndex, move_old_root_to_side: ChildSide) -> Result<(),()> {
            let old_root_idx = match self.get_root() {
                Some(r) => r,
                None => { return Err(()); }
            };
            let new_root_node = match self.node_at_mut(new_root_loc) {
                Some(n) => n,
                None => { return Err(()); }
            };
            match move_old_root_to_side {
                ChildSide::Left => {
                    new_root_node.left = Some(old_root_idx);
                },
                ChildSide::Right => {
                    new_root_node.right = Some(old_root_idx);
                }
            }
            self.set_root(Some(new_root_loc));
            return Ok(());
        }

        pub fn insert_node_below_parent(&mut self,
                parent_node_loc: NodeIndex, old_child_node_side: ChildSide,
                new_child_node_loc: NodeIndex, new_child_node_side: ChildSide) -> Result<(),()> {
            
            if !self.has_node_at(parent_node_loc) || !self.has_node_at(new_child_node_loc) {
                return Err(());
            }

            let old_child_loc = match self.node_at(parent_node_loc) {
                Some(parent_node) => {
                    match old_child_node_side {
                        ChildSide::Left => parent_node.left,
                        ChildSide::Right => parent_node.right
                    }   
                },
                None => { return Err(()); }
            };

            if self.set_node_child(parent_node_loc, Some(new_child_node_loc), old_child_node_side).is_err() {
                return Err(());
            }
            if self.set_node_child(new_child_node_loc, old_child_loc, new_child_node_side).is_err() {
                return Err(());
            }
            return Ok(());
        }

        pub fn insert_node_above_node(&mut self,
                old_child_loc: NodeIndex, new_child_loc: NodeIndex, new_child_side: ChildSide) -> Result<(),()> {
            let parent = self.get_node_parent(old_child_loc);
            if parent.is_none() {
                //return Err("cursor is at a value node which is not the root, but it has no parent");
                return Err(());
            }
            let parent_loc = parent.unwrap();
            let parent = self.node_at(parent_loc);
            if parent.is_none() {
                //return Err("cursor is at a value node that has no parent node at indicated location");
                return Err(());
            }
            let parent = parent.unwrap();
            let parent_left = parent.get_left();
            let parent_right = parent.get_right();

            let old_child_side = if parent_left.is_some() && parent_left.unwrap() == old_child_loc {
                ChildSide::Left
            } else if parent_right.is_some() && parent_right.unwrap() == old_child_loc {
                ChildSide::Right
            } else {
                //return Err("cursor is at a value node that is somehow neither its parent's left nor right child");
                return Err(());
            };

            let result = self.insert_node_below_parent(
                parent_loc, old_child_side,
                new_child_loc, new_child_side);
            if result.is_err() {
                //return Err("unable to insert new operator node in place of an existing value node");
                return Err(());
            }
            return Ok(());
        }

    }

    // TODO: the generic type T shouldn't have to be here, but it appears to be
    //   required in the new() function (generic "_" type didn't compile)
    pub struct PostOrderIter<'a, T> {
        tree: &'a Tree<T>,
        stack: Vec<NodeIndex>,
        visited: HashSet<NodeIndex>
    }

    impl<'a, T> PostOrderIter<'a, T> {

        // this is kind of dumb: a type "T" is required here by the compiler,
        //   even though it's not used.  in Java i would have used '?'
        pub fn new(tree: &'a Tree<T>) -> Self {
            if let Some(i) = tree.root {
                PostOrderIter {
                    tree: tree,
                    stack: vec![i],
                    visited: HashSet::new()
                }
            } else {
                PostOrderIter {
                    tree: tree,
                    stack: vec![],
                    visited: HashSet::new()
                }
            }
        }

        pub fn next(&mut self) -> Option<NodeIndex> {
            while let Some(node_index) = self.stack.pop() {
                if let Some(node) = self.tree.node_at(node_index) {
                    self.stack.push(node_index);
                    let mut pushed_right = false;
                    let mut pushed_left = false;

                    if let Some(right) = node.right {
                        if !self.visited.contains(&right) {
                            self.stack.push(right);
                            pushed_right = true;
                        }
                    }

                    if let Some(left) = node.left {
                        if !self.visited.contains(&left) {
                            self.stack.push(left);
                            pushed_left = true;
                        }
                    }

                    if !pushed_left && !pushed_right {
                        let this_index = self.stack.pop();
                        self.visited.insert(this_index.expect("node must have an index value"));
                        return this_index;  
                    }
                }
            }

            return None
        }
    }
}