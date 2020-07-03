use std::collections::HashMap;
use std::pin::Pin;
use std::ptr::NonNull;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    Red,
    Black,
}

#[derive(Clone, Copy)]
enum ChildType {
    Left,
    Right,
}

impl ChildType {
    pub fn flip(self) -> ChildType {
        match self {
            ChildType::Left => ChildType::Right,
            ChildType::Right => ChildType::Left
        }
    }
}

/// Type that optionally owns a RedBlackTreeNode in a static location on the heap.
type NodeContainer<'node, T> = Option<Pin<Box<RedBlackTreeNode<'node, T>>>>;

/// A non-null pointer to a RedBlackTreeNode.
type NodePointer<'pointer, T> = NonNull<RedBlackTreeNode<'pointer, T>>;

/// A mutably borrowed reference to a NodeContainer.
type NodeContainerRef<'pointer, 'node, T> = &'pointer mut NodeContainer<'node, T>;

/// A hash map from key to NodePointer, used to directly find the RedBlackTreeNode corresponding to the key.
type NodeCache<'keys, T> = HashMap<*const T, NodePointer<'keys, T>>;

/// A descriptor for a location of a node in the tree, either by reference to a parent or as the root.
#[derive(Clone, Copy)]
enum TreePosition<'position, T> {
    Child(NodePointer<'position, T>, ChildType),
    Root()
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

struct NodeCursor<'cursor, 'tree, T> {
    node: &'cursor mut RedBlackTreeNode<'tree, T>,
    nodes: &'cursor mut HashMap<*const T, NodePointer<'tree, T>>,
}

impl<'cursor, 'tree, T> NodeCursor<'cursor, 'tree, T> {
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
            TreePosition::Root() => None,
            TreePosition::Child(parent, _) => Some(NodeCursor {
                node: unsafe { &mut *parent.as_ptr() },
                nodes: self.nodes
            })
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

impl<'cursor, 'tree, T> LeafCursor<'cursor, 'tree, T> {
    pub fn insert(mut self, key: &'tree T) -> NodeCursor<'cursor, 'tree, T> {
        let color = match self.position {
            TreePosition::Child(_, _) => Color::Red,
            TreePosition::Root() => Color::Black,
        };
        let node = RedBlackTreeNode {
            key,
            color,
            position: self.position,
            left_child: None,
            right_child: None,
        };

        let mut bx = Box::pin(node);
        let bxp = NonNull::from(&*bx.as_mut());

        *self.container = Some(bx);
        self.nodes.insert(key, bxp);

        NodeCursor {
            node: &mut *self.container.as_mut().unwrap(),
            nodes: self.nodes,
        }
    }
}

enum TreeCursor<'cursor, 'tree, T: 'cursor> {
    Node(NodeCursor<'cursor, 'tree, T>),
    Leaf(LeafCursor<'cursor, 'tree, T>),
}

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
        TreeCursor::Leaf(LeafCursor { container, position, nodes })
    }
}

struct RedBlackTree<'tree, T> {
    nodes: HashMap<*const T, NodePointer<'tree, T>>,
    root: NodeContainer<'tree, T>,
}

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
            TreeCursor::leaf_from_position(&mut self.root, TreePosition::Root(), &mut self.nodes)
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
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let leaf = tree.root().expect_leaf();

        let mut root = leaf.insert(&4);
        
        let result = root.left_child().expect_leaf().insert(&3);
        assert_eq!(&3, result.node.key);

        root = tree.root().expect_node();
        let five = root.right_child().expect_leaf().insert(&5);

        assert_eq!(&4, five.parent().unwrap().value())
    }
}
