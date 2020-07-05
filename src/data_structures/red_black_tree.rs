use std::collections::HashMap;
use std::fmt::Debug;
use std::pin::Pin;
use std::ptr::NonNull;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    Red,
    Black,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ChildType {
    Left,
    Right,
}

impl ChildType {
    pub fn flip(self) -> ChildType {
        match self {
            ChildType::Left => ChildType::Right,
            ChildType::Right => ChildType::Left,
        }
    }
}

/// A non-null pointer to a RedBlackTreeNode.
type NodePointer<'pointer, T> = NonNull<RedBlackTreeNode<'pointer, T>>;

/// Type that optionally owns a RedBlackTreeNode in a static location on the heap.
struct NodeContainer<'node, T: Debug> {
    value: Option<Pin<Box<RedBlackTreeNode<'node, T>>>>
}

impl<'node, T: Debug> NodeContainer<'node, T> {
    pub fn get_mut(&mut self) -> Option<&mut RedBlackTreeNode<'node, T>> {
        match &mut self.value {
            Some(v) => {
                let c = Pin::into_inner(v.as_mut());
                Some(c)
            },
            None => None,
        }
    }

    #[allow(unused)]
    pub fn get(&self) -> Option<&RedBlackTreeNode<'node, T>> {
        match &self.value {
            Some(v) => {
                let c = Pin::into_inner(v.as_ref());
                Some(c)
            },
            None => None,
        }
    }

    pub fn new() -> Self {
        NodeContainer {
            value: None
        }
    }

    pub fn set(&mut self, value: RedBlackTreeNode<'node, T>) {
        self.value.replace(Box::pin(value));
    }

    pub fn as_ptr(&self) -> Option<NodePointer<'node, T>> {
        match &self.value {
            Some(p) => NonNull::new((&**p) as *const _ as *mut RedBlackTreeNode<T>),
            None => None,
        }
    }

    pub fn empty(&self) -> bool {
        self.value.is_none()
    }
}

/// A mutably borrowed reference to a NodeContainer.
type NodeContainerRef<'pointer, 'node, T> = &'pointer mut NodeContainer<'node, T>;

/// A hash map from key to NodePointer, used to directly find the RedBlackTreeNode corresponding to the key.
type NodeCache<'keys, T> = HashMap<*const T, NodePointer<'keys, T>>;

/// A descriptor for a location of a node in the tree, either by reference to a parent or as the root.
#[derive(Clone, Copy, PartialEq, Debug)]
enum TreePosition<'position, T: Debug> {
    Child(NodePointer<'position, T>, ChildType),
    Root(NonNull<NodeContainer<'position, T>>),
}

impl<'position, T: Debug> TreePosition<'position, T> {
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
            _ => panic!("Root does not have a sibling."),
        }
    }

    pub unsafe fn get(&self) -> Option<&mut RedBlackTreeNode<'position, T>> {
        match self {
            TreePosition::Child(ptr, ct) => (*ptr.as_ptr()).child(*ct),
            TreePosition::Root(r) => (*r.as_ptr()).get_mut()
        }
    }

    pub unsafe fn get_container(&self) -> NodeContainerRef<T> {
        match self {
            TreePosition::Child(ptr, ct) => (*ptr.as_ptr()).child_container(*ct),
            TreePosition::Root(r) => &mut *r.as_ptr(),
        }
    }
}

/// A node of the tree. Nodes own a reference to their key and own their (optional) children.
struct RedBlackTreeNode<'node, T: Debug> {
    key: &'node T,
    color: Color,
    position: TreePosition<'node, T>,
    left_child: NodeContainer<'node, T>,
    right_child: NodeContainer<'node, T>,
}

impl<'node, T: Debug> RedBlackTreeNode<'node, T> {
    pub fn child(&mut self, child_type: ChildType) -> Option<&mut RedBlackTreeNode<'node, T>> {
        let ch = self.child_container(child_type);
        ch.get_mut()
    }

    pub fn child_container<'a>(
        &'a mut self,
        child_type: ChildType,
    ) -> NodeContainerRef<'a, 'node, T> {
        let ch = match child_type {
            ChildType::Left => &mut self.left_child,
            ChildType::Right => &mut self.right_child,
        };

        ch
    }

    /*
    fn set_child(
        &mut self,
        child: Option<Pin<Box<RedBlackTreeNode<'node, T>>>>,
        child_type: ChildType,
    ) {
        let ch = self.child_container(child_type);
        if let Some(mut chh) = child {
            let cc = NonNull::new(&mut *chh as *mut _).unwrap();
            chh.position = TreePosition::Child(cc, child_type);

            ch.set(chh);
        }
    }

    unsafe fn rotate_right(&mut self) {
        let container = self.position.get_container();
        let old_root = &mut *container.0.take().unwrap();
        let new_root = &mut *old_root.left_child.0.take().unwrap();
        let pivot_child = new_root.right_child.0.take();

        old_root.set_child(pivot_child, ChildType::Left);
    }
    */

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
                let sibling = p.position.sibling();
                let uncle = sibling.get();
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

struct NodeCursor<'cursor, 'tree, T: Debug> {
    node: &'cursor mut RedBlackTreeNode<'tree, T>,
    nodes: &'cursor mut HashMap<*const T, NodePointer<'tree, T>>,
}

#[allow(unused)]
impl<'cursor, 'tree, T: Debug> NodeCursor<'cursor, 'tree, T> {
    fn node_color(node: &Option<&mut RedBlackTreeNode<T>>) -> Color {
        match node {
            Some(v) => v.color,
            None => Color::Black,
        }
    }

    pub fn left_child(self) -> TreeCursor<'cursor, 'tree, T> {
        if self.node.left_child.empty() {
            let position =
                TreePosition::Child(NonNull::new(self.node as *mut _).unwrap(), ChildType::Left);
            TreeCursor::leaf_from_position(&mut self.node.left_child, position, self.nodes)
        } else {
            let v = self.node.left_child.get_mut().unwrap();
            TreeCursor::from_node(v, self.nodes)
        }
    }

    pub fn right_child(self) -> TreeCursor<'cursor, 'tree, T> {
        if self.node.right_child.empty() {
            let position =
                TreePosition::Child(NonNull::new(self.node as *mut _).unwrap(), ChildType::Right);
            TreeCursor::leaf_from_position(&mut self.node.right_child, position, self.nodes)
        } else {
            let v = self.node.right_child.get_mut().unwrap();
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

struct LeafCursor<'cursor, 'tree, T: Debug> {
    container: NodeContainerRef<'cursor, 'tree, T>,
    position: TreePosition<'tree, T>,
    nodes: &'cursor mut HashMap<*const T, NodePointer<'tree, T>>,
}

#[allow(unused)]
impl<'cursor, 'tree, T: Debug> LeafCursor<'cursor, 'tree, T> {
    pub fn insert(self, key: &'tree T) -> NodeCursor<'cursor, 'tree, T> {
        let node = RedBlackTreeNode {
            key,
            color: Color::Red,
            position: self.position,
            left_child: NodeContainer::new(),
            right_child: NodeContainer::new(),
        };

        self.container.set(node);
        self.nodes.insert(key, self.container.as_ptr().unwrap());

        let cur = NodeCursor {
            node: self.container.get_mut().unwrap(),
            nodes: self.nodes,
        };

        unsafe { cur.node.repair_tree() };

        cur
    }
}

enum TreeCursor<'cursor, 'tree, T: 'cursor + Debug> {
    Node(NodeCursor<'cursor, 'tree, T>),
    Leaf(LeafCursor<'cursor, 'tree, T>),
}

#[allow(unused)]
impl<'cursor, 'tree, T: Debug> TreeCursor<'cursor, 'tree, T> {
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

struct RedBlackTree<'tree, T: Debug> {
    nodes: NodeCache<'tree, T>,
    root: NodeContainer<'tree, T>,
}

#[allow(unused)]
impl<'tree, T: Debug> RedBlackTree<'tree, T> {
    pub fn new() -> RedBlackTree<'tree, T> {
        RedBlackTree {
            nodes: HashMap::new(),
            root: NodeContainer::new(),
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
        if self.root.empty() {
            let mm = NonNull::from(&mut self.root);
            TreeCursor::leaf_from_position(&mut self.root, TreePosition::Root(mm), &mut self.nodes)
        } else {
            let v = self.root.get_mut().unwrap();
            TreeCursor::from_node(v, &mut self.nodes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Color::Black, Color::Red, *};

    struct NodeExpectation {
        key: usize,
        left_child: Option<Box<NodeExpectation>>,
        right_child: Option<Box<NodeExpectation>>,
        color: Color,
    }

    fn nd(
        key: usize,
        color: Color,
        left_child: Option<Box<NodeExpectation>>,
        right_child: Option<Box<NodeExpectation>>,
    ) -> Option<Box<NodeExpectation>> {
        Some(Box::new(NodeExpectation {
            key,
            left_child,
            right_child,
            color,
        }))
    }

    fn expect_node(
        nodes: &mut NodeCache<usize>,
        position: TreePosition<usize>,
        actual: &NodeContainer<usize>,
        expected: &Option<Box<NodeExpectation>>,
    ) {
        if let Some(expected_node) = expected {
            let actual_ptr = actual.get().expect(&format!(
                "Expected {:?} node with key: {:?}",
                expected_node.color, expected_node.key
            ));
            let actual_node = actual_ptr;

            assert_eq!(expected_node.color, actual_node.color);
            assert_eq!(expected_node.key, *actual_node.key);
            println!("{:?} =? {:?}", actual_node.position, position);
            assert!(actual_node.position == position);

            // Ensure that the value in the tree matches this node.
            {
                let actual_node_ptr = nodes
                    .remove(&(actual_node.key as *const usize))
                    .unwrap()
                    .as_ptr() as *const _;
                let expected_node_ptr = actual_ptr as *const RedBlackTreeNode<_>;
                assert_eq!(true, actual_node_ptr == expected_node_ptr);
            }

            // Recurse left child.
            expect_node(
                nodes,
                TreePosition::Child(
                    NonNull::new(actual_node as *const _ as *mut _).unwrap(),
                    ChildType::Left,
                ),
                &actual_node.left_child,
                &expected_node.left_child,
            );
            // Recurse right child.
            expect_node(
                nodes,
                TreePosition::Child(
                    NonNull::new(actual_node as *const _ as *mut _).unwrap(),
                    ChildType::Right,
                ),
                &actual_node.right_child,
                &expected_node.right_child,
            );
        } else {
            assert!(actual.empty());
        }
    }

    fn expect_tree(actual: &RedBlackTree<usize>, expected: &Option<Box<NodeExpectation>>) {
        let mut nodes = actual.nodes.clone();
        expect_node(
            &mut nodes,
            TreePosition::Root(NonNull::new(&actual.root as *const _ as *mut _).unwrap()),
            &actual.root,
            expected,
        );
        assert_eq!(0, nodes.len());
    }

    #[test]
    fn test_root_insert() {
        // Insert case 1 on wikipedia.
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let leaf = tree.root().expect_leaf();

        leaf.insert(&4);

        expect_tree(&tree, &nd(4, Black, None, None));
    }

    #[test]
    fn test_insert_children() {
        // Insert case 2 on wikipedia.
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let leaf = tree.root().expect_leaf();
        let mut root = leaf.insert(&4);
        let result = root.left_child().expect_leaf().insert(&3);
        assert_eq!(&3, result.node.key);
        assert_eq!(Red, result.node.color);

        root = tree.root().expect_node();
        let five = root.right_child().expect_leaf().insert(&5);
        assert_eq!(Red, five.node.color);
        assert_eq!(&4, five.parent().unwrap().value());

        expect_tree(
            &tree,
            &nd(4, Black, nd(3, Red, None, None), nd(5, Red, None, None)),
        );
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
        left.left_child().expect_leaf().insert(&3);

        expect_tree(
            &tree,
            &nd(
                5,
                Black,
                nd(4, Black, nd(3, Red, None, None), None),
                nd(6, Black, None, None),
            ),
        );
    }

    #[test]
    fn test_insert_case_four() {}
}
