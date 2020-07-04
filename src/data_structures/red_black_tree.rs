use std::collections::HashMap;
use std::pin::Pin;
use std::ptr::NonNull;

#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    Red,
    Black,
}

#[allow(unused)]
#[derive(Clone, Copy)]
enum ChildType {
    Left,
    Right,
}

impl ChildType {
    #[allow(unused)]
    pub fn flip(self) -> ChildType {
        match self {
            ChildType::Left => ChildType::Right,
            ChildType::Right => ChildType::Left,
        }
    }
}

/// Type that optionally owns a RedBlackTreeNode in a static location on the heap.
type NodeContainer<'node, T> = Option<Pin<Box<RedBlackTreeNode<'node, T>>>>;

/// A non-null pointer to a RedBlackTreeNode.
type NodePointer<'pointer, T> = NonNull<RedBlackTreeNode<'pointer, T>>;

/// A mutably borrowed reference to a NodeContainer.
#[allow(unused)]
type NodeContainerRef<'pointer, 'node, T> = &'pointer mut NodeContainer<'node, T>;

/// A hash map from key to NodePointer, used to directly find the RedBlackTreeNode corresponding to the key.
#[allow(unused)]
type NodeCache<'keys, T> = HashMap<*const T, NodePointer<'keys, T>>;

/// A descriptor for a location of a node in the tree, either by reference to a parent or as the root.
#[allow(unused)]
#[derive(Clone, Copy)]
enum TreePosition<'position, T> {
    Child(NodePointer<'position, T>, ChildType),
    Root(NonNull<NodeContainer<'position, T>>),
}

#[allow(unused)]
impl<'position, T> TreePosition<'position, T> {
    /// Note: this is unsafe because the resulting borrow aliases the pointer passed in.
    pub unsafe fn parent(&self) -> Option<&mut RedBlackTreeNode<'position, T>> {
        match self {
            TreePosition::Child(ptr, _) => Some(&mut *ptr.as_ptr()),
            TreePosition::Root(_) => None,
        }
    }

    pub fn sibling(&self) -> TreePosition<'position, T> {
        match self {
            TreePosition::Child(ptr, ct) => TreePosition::Child(*ptr, ct.flip()),
            _ => unimplemented!(),
        }
    }

    pub unsafe fn get(&self) -> Option<&mut RedBlackTreeNode<'position, T>> {
        match self {
            TreePosition::Child(ptr, ct) => (*ptr.as_ptr()).child(*ct),
            TreePosition::Root(r) => {
                let p = (*(*r).as_ptr()).as_deref_mut().unwrap();
                Some(p)
            }
        }
    }
}

/// A node of the tree. Nodes own a reference to their key and own their (optional) children.
#[derive(Clone)]
struct RedBlackTreeNode<'node, T> {
    key: &'node T,
    color: Color,
    position: TreePosition<'node, T>,
    left_child: NodeContainer<'node, T>,
    right_child: NodeContainer<'node, T>,
}

impl<'node, T> RedBlackTreeNode<'node, T> {
    pub fn child(&mut self, child_type: ChildType) -> Option<&mut RedBlackTreeNode<'node, T>> {
        let ch = match child_type {
            ChildType::Left => &mut self.left_child,
            ChildType::Right => &mut self.right_child,
        };

        match ch {
            Some(c) => Some(c.as_mut().get_mut()),
            None => None,
        }
    }

    #[allow(unused)]
    unsafe fn repair_tree(&mut self) {
        let parent = self.position.parent();

        if let Some(RedBlackTreeNode {
            color: Color::Black,
            ..
        }) = parent
        {
            // Insert case 2: parent is black, do nothing.
        } else if let Some(p) = parent {
            if let Some(grandparent) = p.position.parent() {
                let c = p.position.sibling();
                let uncle = c.get();
                let uncle_color = NodeCursor::node_color(&uncle);

                if uncle_color == Color::Red {
                    // Insert case 3: parent and uncle are black.
                    p.color = Color::Black;
                    uncle.unwrap().color = Color::Black;
                    grandparent.color = Color::Red;
                    grandparent.repair_tree();
                } else {
                    // Insert case 4.
                    unimplemented!()
                }
            } else {
                // Insert case 4.
                unimplemented!()
            }
        } else {
            // Insert case 1: node is root; color black.
            self.color = Color::Black;
        }
    }
}

#[allow(unused)]
struct NodeCursor<'cursor, 'tree, T> {
    node: &'cursor mut RedBlackTreeNode<'tree, T>,
    nodes: &'cursor mut HashMap<*const T, NodePointer<'tree, T>>,
}

#[allow(unused)]
impl<'cursor, 'tree, T> NodeCursor<'cursor, 'tree, T> {
    fn node_color(node: &Option<&mut RedBlackTreeNode<T>>) -> Color {
        match node {
            Some(v) => v.color,
            None => Color::Black,
        }
    }

    pub fn left_child(self) -> TreeCursor<'cursor, 'tree, T> {
        if self.node.left_child.is_none() {
            let p: *mut _ = self.node;
            let position = TreePosition::Child(NonNull::new(p).unwrap(), ChildType::Left);
            TreeCursor::leaf_from_position(&mut self.node.left_child, position, self.nodes)
        } else {
            let v = &mut **self.node.left_child.as_mut().unwrap();
            TreeCursor::from_node(v, self.nodes)
        }
    }

    pub fn right_child(self) -> TreeCursor<'cursor, 'tree, T> {
        if self.node.right_child.is_none() {
            let p: *mut _ = self.node;
            let position = TreePosition::Child(NonNull::new(p).unwrap(), ChildType::Right);
            TreeCursor::leaf_from_position(&mut self.node.right_child, position, self.nodes)
        } else {
            let v = &mut **self.node.right_child.as_mut().unwrap();
            TreeCursor::from_node(v, self.nodes)
        }
    }

    pub fn parent(self) -> Option<NodeCursor<'cursor, 'tree, T>> {
        match self.node.position {
            TreePosition::Root(_) => None,
            TreePosition::Child(parent, _) => Some(NodeCursor {
                node: unsafe { &mut *parent.as_ptr() },
                nodes: self.nodes,
            }),
        }
    }

    pub fn value(&self) -> &T {
        self.node.key
    }
}

struct LeafCursor<'cursor, 'tree, T> {
    container: NodeContainerRef<'cursor, 'tree, T>,
    position: TreePosition<'tree, T>,
    nodes: &'cursor mut HashMap<*const T, NodePointer<'tree, T>>,
}

#[allow(unused)]
impl<'cursor, 'tree, T> LeafCursor<'cursor, 'tree, T> {
    pub fn insert(self, key: &'tree T) -> NodeCursor<'cursor, 'tree, T> {
        let node = RedBlackTreeNode {
            key,
            color: Color::Red,
            position: self.position,
            left_child: None,
            right_child: None,
        };

        let mut bx = Box::pin(node);
        let bxp = NonNull::from(&*bx.as_mut());

        *self.container = Some(bx);
        self.nodes.insert(key, bxp);

        let cur = NodeCursor {
            node: &mut *self.container.as_mut().unwrap(),
            nodes: self.nodes,
        };

        unsafe { cur.node.repair_tree() };

        cur
    }
}

enum TreeCursor<'cursor, 'tree, T: 'cursor> {
    Node(NodeCursor<'cursor, 'tree, T>),
    Leaf(LeafCursor<'cursor, 'tree, T>),
}

#[allow(unused)]
impl<'cursor, 'tree, T> TreeCursor<'cursor, 'tree, T> {
    pub fn expect_node(self) -> NodeCursor<'cursor, 'tree, T> {
        if let TreeCursor::Node(n) = self {
            n
        } else {
            panic!("Expected node, got leaf.")
        }
    }

    pub fn expect_leaf(self) -> LeafCursor<'cursor, 'tree, T> {
        if let TreeCursor::Leaf(n) = self {
            n
        } else {
            panic!("Expected leaf, got node.")
        }
    }

    pub fn from_node(
        node: &'cursor mut RedBlackTreeNode<'tree, T>,
        nodes: &'cursor mut NodeCache<'tree, T>,
    ) -> TreeCursor<'cursor, 'tree, T> {
        TreeCursor::Node(NodeCursor { node, nodes })
    }

    pub fn leaf_from_position(
        container: NodeContainerRef<'cursor, 'tree, T>,
        position: TreePosition<'tree, T>,
        nodes: &'cursor mut NodeCache<'tree, T>,
    ) -> TreeCursor<'cursor, 'tree, T> {
        TreeCursor::Leaf(LeafCursor {
            container,
            position,
            nodes,
        })
    }
}

struct RedBlackTree<'tree, T> {
    nodes: HashMap<*const T, NodePointer<'tree, T>>,
    root: NodeContainer<'tree, T>,
}

#[allow(unused)]
impl<'tree, T> RedBlackTree<'tree, T> {
    pub fn new() -> RedBlackTree<'tree, T> {
        RedBlackTree {
            nodes: HashMap::new(),
            root: None,
        }
    }

    pub fn get<'cursor>(&'cursor mut self, key: *const T) -> Option<NodeCursor<'cursor, 'tree, T>> {
        let node = unsafe { &mut *self.nodes.get(&key)?.as_ptr() };
        Some(NodeCursor {
            node,
            nodes: &mut self.nodes,
        })
    }

    pub fn root<'cursor>(&'cursor mut self) -> TreeCursor<'cursor, 'tree, T> {
        if self.root.is_none() {
            let mm = NonNull::from(&mut self.root);
            TreeCursor::leaf_from_position(&mut self.root, TreePosition::Root(mm), &mut self.nodes)
        } else {
            let v = &mut **self.root.as_mut().unwrap();
            TreeCursor::from_node(v, &mut self.nodes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_insert() {
        // Insert case 1 on wikipedia.
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let leaf = tree.root().expect_leaf();

        leaf.insert(&4);
        assert_eq!(&4, tree.root().expect_node().value());

        let node = tree.get(&4).unwrap();

        assert_eq!(&4, node.value());
        assert_eq!(Color::Black, node.node.color);
    }

    #[test]
    fn test_insert_children() {
        // Insert case 2 on wikipedia.
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let leaf = tree.root().expect_leaf();

        let mut root = leaf.insert(&4);

        let result = root.left_child().expect_leaf().insert(&3);
        assert_eq!(&3, result.node.key);
        assert_eq!(Color::Red, result.node.color);

        root = tree.root().expect_node();
        let five = root.right_child().expect_leaf().insert(&5);
        assert_eq!(Color::Red, five.node.color);

        assert_eq!(&4, five.parent().unwrap().value());
    }

    #[test]
    fn test_insert_case_three() {
        // Insert case 3 on wikipedia: insert under red parent with red uncle.
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let root = tree.root().expect_leaf().insert(&5);
        root.right_child().expect_leaf().insert(&6);
        let left = tree
            .root()
            .expect_node()
            .left_child()
            .expect_leaf()
            .insert(&4);
        let mut node = left.left_child().expect_leaf().insert(&3);

        assert_eq!(Color::Red, node.node.color);
        // Parent
        node = node.parent().unwrap();
        assert_eq!(Color::Black, node.node.color);
        // Grandparent
        node = node.parent().unwrap();
        assert_eq!(Color::Black, node.node.color);
        // Uncle
        node = node.right_child().expect_node();
        assert_eq!(Color::Black, node.node.color);
    }
}
