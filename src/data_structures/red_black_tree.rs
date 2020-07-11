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
    /// Return the opposite ChildType.
    fn flip(self) -> ChildType {
        match self {
            ChildType::Left => ChildType::Right,
            ChildType::Right => ChildType::Left,
        }
    }
}

/// A non-null pointer to a RedBlackTreeNode.
type NodePointer<'pointer, T> = NonNull<RedBlackTreeNode<'pointer, T>>;

/// Container that optionally owns a RedBlackTreeNode in a static location on the heap.
struct NodeContainer<'node, T: Debug> {
    value: Option<Pin<Box<RedBlackTreeNode<'node, T>>>>,
}

impl<'node, T: Debug> Debug for NodeContainer<'node, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.get() {
            Some(v) => v.fmt(f),
            None => write!(f, "()"),
        }
    }
}

impl<'node, T: Debug> NodeContainer<'node, T> {
    /// Empties this container and returns its (optional) value.
    fn take(&mut self) -> Option<Pin<Box<RedBlackTreeNode<'node, T>>>> {
        self.value.take()
    }

    /// Returns an optional mutable reference to the RedBlackTree in this container.
    fn get_mut(&mut self) -> Option<&mut RedBlackTreeNode<'node, T>> {
        match &mut self.value {
            Some(v) => {
                let c = Pin::into_inner(v.as_mut());
                Some(c)
            }
            None => None,
        }
    }

    /// Returns an optional reference to the RedBlackTree in this container.
    pub fn get(&self) -> Option<&RedBlackTreeNode<'node, T>> {
        match &self.value {
            Some(v) => {
                let c = Pin::into_inner(v.as_ref());
                Some(c)
            }
            None => None,
        }
    }

    /// Create a new empty container.
    fn new() -> Self {
        NodeContainer { value: None }
    }

    /// Returns an optional NonNull pointer to the content of this container.
    fn get_ptr(&self) -> Option<NodePointer<'node, T>> {
        match &self.value {
            Some(p) => NonNull::new((&**p) as *const RedBlackTreeNode<T> as *mut _),
            None => None,
        }
    }

    /// Returns true if this container is empty.
    fn empty(&self) -> bool {
        self.value.is_none()
    }
}

/// A mutably borrowed reference to a NodeContainer.
type NodeContainerRef<'pointer, 'node, T> = &'pointer mut NodeContainer<'node, T>;

/// A hash map from key to NodePointer, used to directly find the RedBlackTreeNode corresponding to the key.
type NodeCache<'keys, T> = HashMap<*const T, NodePointer<'keys, T>>;

/// A descriptor for a location of a node in the tree, either by reference to a parent or as the root.
#[derive(PartialEq, Debug)]
enum TreePosition<'position, T: Debug> {
    /// Position of the root node with reference to its NodeContainer.
    Root(NonNull<NodeContainer<'position, T>>),
    /// Position of a non-root node by reference to its parent.
    Child(NodePointer<'position, T>, ChildType),
}

impl<'position, T: Debug> Clone for TreePosition<'position, T> {
    fn clone(&self) -> Self {
        match self {
            TreePosition::Child(ptr, ct) => TreePosition::Child(*ptr, *ct),
            TreePosition::Root(r) => TreePosition::Root(*r),
        }
    }
}

impl<'position, T: Debug> TreePosition<'position, T> {
    fn is_root(&self) -> bool {
        match &self {
            TreePosition::Root(_) => true,
            _ => false,
        }
    }

    /// Returns the parent to this node, if it exists.
    /// This is unsafe because the resulting borrow aliases the pointer passed in, which is not consumed.
    unsafe fn parent(&self) -> Option<&mut RedBlackTreeNode<'position, T>> {
        match self {
            TreePosition::Child(ptr, _) => Some(&mut *ptr.as_ptr()),
            TreePosition::Root(_) => None,
        }
    }

    /// Returns the ChildType of this node. Panics if this is a root node.
    fn child_type(&self) -> ChildType {
        match self {
            TreePosition::Child(_, ct) => *ct,
            _ => panic!("Root does not have a child type."),
        }
    }

    /// Returns the position of this node's sibling, which may be a leaf node. Panics if this is a root node.
    fn sibling(&self) -> TreePosition<'position, T> {
        match self {
            TreePosition::Child(ptr, ct) => TreePosition::Child(*ptr, ct.flip()),
            _ => panic!("Root does not have a sibling."),
        }
    }

    /// Returns the node in this position. Unsafe because the TreePosition does not own its node, but useful for
    /// getting nodes based on position.
    unsafe fn get(&self) -> Option<&mut RedBlackTreeNode<'position, T>> {
        match self {
            TreePosition::Child(ptr, ct) => (*ptr.as_ptr()).child_mut(*ct),
            TreePosition::Root(r) => (*r.as_ptr()).get_mut(),
        }
    }

    /// Returns a pointer to the container for the node referred to.
    unsafe fn get_container(&self) -> NodeContainerRef<'position, 'position, T> {
        match self {
            TreePosition::Child(ptr, ct) => (*ptr.as_ptr()).child_container_mut(*ct),
            TreePosition::Root(r) => &mut *r.as_ptr(),
        }
    }

    /// Sets the value of this container to the given RedBlackTree.
    fn set(&self, value: RedBlackTreeNode<'position, T>) {
        self.set_pinned(Some(Box::pin(value)));
    }

    /// Sets the value of this container to the given already-pinned RedBlackTree.
    fn set_pinned(&self, value: Option<Pin<Box<RedBlackTreeNode<'position, T>>>>) {
        let container = unsafe { self.get_container() };
        match value {
            Some(mut v) => {
                v.position = self.clone();
                container.value.replace(v);
            }
            None => {
                container.value.take();
            }
        };
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
    /// Returns an optional reference to the child of the type provided.
    #[allow(unused)]
    fn child(&self, child_type: ChildType) -> Option<&RedBlackTreeNode<'node, T>> {
        self.child_container(child_type).get()
    }

    /// Returns a mutable reference to the container of the requested child node.
    #[allow(unused)]
    fn child_container<'a>(&'a self, child_type: ChildType) -> &'a NodeContainer<'node, T> {
        match child_type {
            ChildType::Left => &self.left_child,
            ChildType::Right => &self.right_child,
        }
    }

    /// Returns an optional mutable reference to the child of the type provided.
    fn child_mut(&mut self, child_type: ChildType) -> Option<&mut RedBlackTreeNode<'node, T>> {
        self.child_container_mut(child_type).get_mut()
    }

    /// Returns a mutable reference to the container of the requested child node.
    fn child_container_mut<'a>(
        &'a mut self,
        child_type: ChildType,
    ) -> NodeContainerRef<'a, 'node, T> {
        match child_type {
            ChildType::Left => &mut self.left_child,
            ChildType::Right => &mut self.right_child,
        }
    }

    /// Sets the given child of this node to the given value. Assumes that this child
    /// is currently empty, but this is not checked because it is only used as a helper
    /// for `rotate`, which calls `take` in each case before `set_child`.
    fn set_child(
        &mut self,
        child: Option<Pin<Box<RedBlackTreeNode<'node, T>>>>,
        child_type: ChildType,
    ) {
        let position = TreePosition::Child(NonNull::new(self as *mut _).unwrap(), child_type);
        position.set_pinned(child);
    }

    /// Rotate this node in the given direction. If the direction is Right, the left child
    /// of this node becomes its parent. If the direction is Left, the right child of this
    /// node becomes its parent.
    fn rotate(&mut self, direction: ChildType) {
        let position = self.position.clone();
        let container = unsafe { self.position.get_container() };
        let mut new_root = self.child_container_mut(direction.flip()).take().unwrap();
        let pivot_child = new_root.child_container_mut(direction).take();

        self.set_child(pivot_child, direction.flip());
        new_root.set_child(container.take(), direction);
        position.set_pinned(Some(new_root));
    }

    /// Returns the color of an optional node. The `None` value here represents a leaf node,
    /// which is black by definition in a red-black tree.
    fn node_color(node: &Option<&mut RedBlackTreeNode<T>>) -> Color {
        match node {
            Some(v) => v.color,
            None => Color::Black,
        }
    }

    /// Repair the tree after a given newly inserted node.
    fn repair_tree(&mut self) {
        let parent_container = unsafe { self.position.parent() };

        if let Some(RedBlackTreeNode {
            color: Color::Black,
            ..
        }) = parent_container
        {
            // Insert case 2: parent is black, do nothing.
        } else if let Some(mut parent) = parent_container {
            let grandparent = unsafe { parent.position.parent() }.expect(
                "Parent node is red, so it should not be the root, but it does not have a parent.",
            );
            let sibling = parent.position.sibling();
            let uncle = unsafe { sibling.get() };
            let uncle_color = RedBlackTreeNode::node_color(&uncle);

            if uncle_color == Color::Red {
                // Insert case 3: parent and uncle are red; color both black and grandparent red.
                parent.color = Color::Black;
                uncle.unwrap().color = Color::Black;
                grandparent.color = Color::Red;
                grandparent.repair_tree();
            } else {
                // Insert case 4.
                let rotate_direction =
                    match (self.position.child_type(), parent.position.child_type()) {
                        (ChildType::Left, ChildType::Left) => ChildType::Right,
                        (ChildType::Right, ChildType::Right) => ChildType::Left,
                        (ChildType::Right, ChildType::Left) => {
                            parent.rotate(ChildType::Left);
                            parent = self;
                            ChildType::Right
                        }
                        (ChildType::Left, ChildType::Right) => {
                            parent.rotate(ChildType::Right);
                            parent = self;
                            ChildType::Left
                        }
                    };

                let grandparent = unsafe { parent.position.parent() }.expect(
                    "Parent node is red, so it should not be the root, but it does not have a parent.",
                );
                grandparent.rotate(rotate_direction);
                parent.color = Color::Black;
                grandparent.color = Color::Red;
            }
        } else {
            // Insert case 1: node is root; color black.
            self.color = Color::Black;
        }
    }
}

impl<'a, T: Debug> Debug for RedBlackTreeNode<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({:?} {:?} {:?} {:?})",
            self.key, self.color, self.left_child, self.right_child
        )
    }
}

/// Cursor that points to an existing node in the tree. Contains a
/// mutable reference to a tree's `NodeCache`, because it needs to be
/// updated for inserts and deletes. This means that only one NodeCursor
/// may exist at once, even though the `node` references might not conflict.
pub struct NodeCursor<'cursor, 'tree, T: Debug> {
    node: &'cursor mut RedBlackTreeNode<'tree, T>,
    node_cache: &'cursor mut NodeCache<'tree, T>,
}

impl<'cursor, 'tree, T: Debug> NodeCursor<'cursor, 'tree, T> {
    /// Convert into a cursor for the given child.
    fn child(self, child_type: ChildType) -> TreeCursor<'cursor, 'tree, T> {
        let position = TreePosition::Child(NonNull::new(self.node as *mut _).unwrap(), child_type);
        let container = self.node.child_container_mut(child_type);
        if container.empty() {
            TreeCursor::leaf_from_position(position, self.node_cache)
        } else {
            TreeCursor::from_node(container.get_mut().unwrap(), self.node_cache)
        }
    }

    /// Convert into a cursor to the left child.
    pub fn left_child(self) -> TreeCursor<'cursor, 'tree, T> {
        self.child(ChildType::Left)
    }

    /// Convert into a cursor for the right child.
    pub fn right_child(self) -> TreeCursor<'cursor, 'tree, T> {
        self.child(ChildType::Right)
    }

    /// Convert into an (optional) cursor for the parent node. Although we don't own a
    /// reference to the parent, this is safe because we own a mutable reference to the
    /// tree's `NodeCache`, and therefore another cursor referencing the parent can't
    /// exist in an overlapping lifetime.
    pub fn parent(self) -> Option<NodeCursor<'cursor, 'tree, T>> {
        match self.node.position {
            TreePosition::Root(_) => None,
            TreePosition::Child(parent, _) => Some(NodeCursor {
                node: unsafe { &mut *parent.as_ptr() },
                node_cache: self.node_cache,
            }),
        }
    }

    /// Return the key associated with the node at this cursor.
    pub fn key(&self) -> &T {
        self.node.key
    }

    /// Delete the node from the tree.
    pub fn delete(self) {
        let container = unsafe { self.node.position.get_container() };
        self.node_cache.remove(&(self.node.key as *const _));

        let replacement = if self.node.left_child.empty() {
            self.node.right_child.take()
        } else if self.node.right_child.empty() {
            self.node.left_child.take()
        } else {
            unimplemented!()
        };

        self.node.position.set_pinned(replacement);
        let node = container.get_mut();

        match node {
            Some(r) => {
                if self.node.position.is_root() {
                    r.color = Color::Black;
                } else {
                    unimplemented!()
                }
            }
            None => (),
        }
    }
}

/// Cursor that points to a leaf node in a tree, allowing insertion.
pub struct LeafCursor<'cursor, 'tree, T: Debug> {
    position: TreePosition<'tree, T>,
    nodes: &'cursor mut HashMap<*const T, NodePointer<'tree, T>>,
}

impl<'cursor, 'tree, T: Debug> LeafCursor<'cursor, 'tree, T> {
    /// Insert the key into this node's position in the tree. Consumes this
    /// `LeafCursor` and returns a `NodeCursor` to the inserted node.
    pub fn insert(self, key: &'tree T) -> NodeCursor<'cursor, 'tree, T> {
        let node = RedBlackTreeNode {
            key,
            color: Color::Red,
            position: self.position.clone(),
            left_child: NodeContainer::new(),
            right_child: NodeContainer::new(),
        };

        let container = unsafe { self.position.get_container() };

        self.position.set(node);
        self.nodes.insert(key, container.get_ptr().unwrap());

        let cur = NodeCursor {
            node: container.get_mut().unwrap(),
            node_cache: self.nodes,
        };

        cur.node.repair_tree();

        cur
    }
}

/// A cursor that points either to an existing node of the tree or
/// to a leaf.
pub enum TreeCursor<'cursor, 'tree, T: 'cursor + Debug> {
    Node(NodeCursor<'cursor, 'tree, T>),
    Leaf(LeafCursor<'cursor, 'tree, T>),
}

impl<'cursor, 'tree, T: Debug> TreeCursor<'cursor, 'tree, T> {
    /// Extract a `NodeCursor` from this. Panic if it is a leaf.
    pub fn unwrap_node(self) -> NodeCursor<'cursor, 'tree, T> {
        if let TreeCursor::Node(n) = self {
            n
        } else {
            panic!("Expected node, got leaf.")
        }
    }

    /// Extract a `LeafCursor` from this. Panic if it is a node.
    pub fn unwrap_leaf(self) -> LeafCursor<'cursor, 'tree, T> {
        if let TreeCursor::Leaf(n) = self {
            n
        } else {
            panic!("Expected leaf, got node.")
        }
    }

    /// Construct a `TreeCursor::Node` from a mutable node reference.
    fn from_node(
        node: &'cursor mut RedBlackTreeNode<'tree, T>,
        nodes: &'cursor mut NodeCache<'tree, T>,
    ) -> TreeCursor<'cursor, 'tree, T> {
        TreeCursor::Node(NodeCursor {
            node,
            node_cache: nodes,
        })
    }

    /// Construct a `TreeCursor::Leaf` from a container and position.
    fn leaf_from_position(
        position: TreePosition<'tree, T>,
        nodes: &'cursor mut NodeCache<'tree, T>,
    ) -> TreeCursor<'cursor, 'tree, T> {
        TreeCursor::Leaf(LeafCursor { position, nodes })
    }
}

/// A data structure which is both ordered and indexed by key, allowing a list
/// to be maintained with random inserts, swaps, and deletions.
/// The implementation combines a red-black tree with a hashmap.
pub struct RedBlackTree<'tree, T: Debug> {
    nodes: NodeCache<'tree, T>,
    root: NodeContainer<'tree, T>,
}

impl<'tree, T: Debug> RedBlackTree<'tree, T> {
    /// Construct an empty tree.
    pub fn new() -> RedBlackTree<'tree, T> {
        RedBlackTree {
            nodes: HashMap::new(),
            root: NodeContainer::new(),
        }
    }

    /// Returns a `NodeCursor` for the given node, if it is found in the tree.
    pub fn get<'cursor>(&'cursor mut self, key: *const T) -> Option<NodeCursor<'cursor, 'tree, T>> {
        let node = unsafe { &mut *self.nodes.get(&key)?.as_ptr() };
        Some(NodeCursor {
            node,
            node_cache: &mut self.nodes,
        })
    }

    /// Returns the root node of the tree, whether it is a node or a leaf.
    pub fn root<'cursor>(&'cursor mut self) -> TreeCursor<'cursor, 'tree, T> {
        if self.root.empty() {
            let mm = NonNull::from(&mut self.root);
            TreeCursor::leaf_from_position(TreePosition::Root(mm), &mut self.nodes)
        } else {
            let v = self.root.get_mut().unwrap();
            TreeCursor::from_node(v, &mut self.nodes)
        }
    }

    /// Swap the positions of the nodes associated with each key in the tree. Simply swaps the keys
    /// within the nodes; the tree structure exactly the same.
    pub fn swap(&mut self, key1: *const T, key2: *const T) {
        let node1 = unsafe { &mut *self.nodes.remove(&key1).unwrap().as_ptr() };
        let node2 = unsafe { &mut *self.nodes.remove(&key2).unwrap().as_ptr() };

        std::mem::swap(&mut node1.key, &mut node2.key);

        self.nodes.insert(
            node1.key,
            NonNull::new(node1 as *const RedBlackTreeNode<T> as *mut _).unwrap(),
        );
        self.nodes.insert(
            node2.key,
            NonNull::new(node2 as *const RedBlackTreeNode<T> as *mut _).unwrap(),
        );
    }
}

impl<'tree, T: Debug> Debug for RedBlackTree<'tree, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Color::Black, Color::Red, *};

    /// A representation of the "expected" shape of the tree resulting from operations.
    struct NodeExpectation {
        key: usize,
        left_child: Option<Box<NodeExpectation>>,
        right_child: Option<Box<NodeExpectation>>,
        color: Color,
    }

    /// A helper function to build a `NodeExpectation`.
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

    /// Recursively compare an expected node with the corresponding actual tree node,
    /// panic if there are any issues.
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
            assert!(actual_node.position == position);

            // Ensure that the value in the tree matches this node.
            {
                let actual_node_ptr = nodes
                    .remove(&(actual_node.key as *const usize))
                    .expect("Node should be in nodes cache, but isn't.")
                    .as_ptr() as *const _;
                let expected_node_ptr = actual_ptr as *const RedBlackTreeNode<_>;
                assert!(actual_node_ptr == expected_node_ptr);
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

    /// Compare the root of the tree with the given `NodeExpectation`.
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

    /// Check that the tree is valid. Returns the number of black nodes on the path to each descendent
    /// (which, according to rule 5, is the same for all descendant paths of a given node).
    ///
    /// The rules of a red-black tree (per Wikipedia) are:
    /// 1. Each node is either red or black.
    /// 2. The root is black.
    /// 3. All leaves (NIL) are black.
    /// 4. If a node is red, then both its children are black.
    /// 5. Every path from a given node to any of its descendant NIL nodes goes through the same number of black nodes.
    ///
    /// #1 is enforced by the type system; #3 is handled implicitly (we don't store the color of leaf nodes). The
    /// other properties are checked here, as well as these data structure invariants:
    ///
    /// 1. A node's position should match the position where we found it.
    /// 2. A node's location in the node cache should point to that node.
    /// 3. Every node in the node cache should be in the tree (i.e. we shouldn't have
    ///    dangling pointers in the node cache).
    ///
    /// #3 is tested by removing nodes from the node cache as they are reached; when called on the root node the resulting
    /// node cache should be empty. The others are tested explicitly.
    fn check_tree_node(
        nodes: &mut NodeCache<usize>,
        position: TreePosition<usize>,
        actual: &NodeContainer<usize>,
    ) -> usize {
        if let Some(node) = actual.get() {
            // Ensure that the node's reference to its position matches the position in the tree at which we
            // found it. (invariant #1)
            assert!(node.position == position);

            if let Some(parent) = unsafe { node.position.parent() } {
                // If this node has a red parent, this node should be black. (property #4)
                if parent.color == Color::Red {
                    assert!(node.color == Color::Black);
                }
            } else {
                // If this node is the root, it should be black. (property #2)
                assert_eq!(Color::Black, node.color);
            }

            // Ensure that the value in the tree matches this node. (invariant #2)
            {
                let node_ptr =
                    nodes.remove(&(node.key as *const usize)).unwrap().as_ptr() as *const _;
                let expected_node_ptr = node_ptr as *const RedBlackTreeNode<_>;
                assert!(node_ptr == expected_node_ptr);
            }

            // Recurse left child.
            let left_d = check_tree_node(
                nodes,
                TreePosition::Child(
                    NonNull::new(node as *const _ as *mut _).unwrap(),
                    ChildType::Left,
                ),
                &node.left_child,
            );
            // Recurse right child.
            let right_d = check_tree_node(
                nodes,
                TreePosition::Child(
                    NonNull::new(node as *const _ as *mut _).unwrap(),
                    ChildType::Right,
                ),
                &node.right_child,
            );

            // Ensure that the black distance to ancestors along the left and right children is the same. (property #5)
            assert_eq!(left_d, right_d);
            if node.color == Color::Black {
                // This node is black, so count it in the black distance to descendants.
                left_d + 1
            } else {
                // This node is red, so black distance to descendants is unchanged.
                left_d
            }
        } else {
            // Leaf node; return a black distance of 0 (we don't count the leaf node as being on the path).
            0
        }
    }

    /// Check a tree according to properties of a red black tree as well as invariants specific to our
    /// implementation.
    fn check_tree(actual: &RedBlackTree<usize>) {
        let mut nodes = actual.nodes.clone();
        check_tree_node(
            &mut nodes,
            TreePosition::Root(NonNull::new(&actual.root as *const _ as *mut _).unwrap()),
            &actual.root,
        );
        assert_eq!(0, nodes.len());
    }

    fn check_bst(node: &RedBlackTreeNode<usize>) -> (usize, usize) {
        let mut min = *node.key;
        let mut max = *node.key;
        if let Some(left) = node.child(ChildType::Left) {
            let (left_min, left_max) = check_bst(left);
            assert!(left_max < *node.key);
            min = left_min;
        }
        if let Some(right) = node.child(ChildType::Right) {
            let (right_min, right_max) = check_bst(right);
            assert!(right_min > *node.key);
            max = right_max;
        }
        (min, max)
    }

    #[test]
    fn test_root_insert() {
        // Insert case 1 on wikipedia.
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let leaf = tree.root().unwrap_leaf();

        leaf.insert(&4);

        expect_tree(&tree, &nd(4, Black, None, None));
    }

    #[test]
    fn test_insert_children() {
        // Insert case 2 on wikipedia.
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let leaf = tree.root().unwrap_leaf();
        let mut root = leaf.insert(&4);
        let result = root.left_child().unwrap_leaf().insert(&3);
        assert_eq!(&3, result.node.key);
        assert_eq!(Red, result.node.color);

        root = tree.root().unwrap_node();
        let five = root.right_child().unwrap_leaf().insert(&5);
        assert_eq!(Red, five.node.color);
        assert_eq!(&4, five.parent().unwrap().key());

        expect_tree(
            &tree,
            &nd(4, Black, nd(3, Red, None, None), nd(5, Red, None, None)),
        );
    }

    #[test]
    fn test_insert_case_three() {
        // Insert case 3 on wikipedia: insert under red parent with red uncle.
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let root = tree.root().unwrap_leaf().insert(&5);
        root.right_child().unwrap_leaf().insert(&6);
        let left = tree
            .root()
            .unwrap_node()
            .left_child()
            .unwrap_leaf()
            .insert(&4);
        left.left_child().unwrap_leaf().insert(&3);

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
    fn test_insert_rotate_right() {
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let mut c = tree.root().unwrap_leaf().insert(&5);
        c = c.left_child().unwrap_leaf().insert(&4);
        c.left_child().unwrap_leaf().insert(&3);

        expect_tree(
            &tree,
            &nd(
                4,
                Color::Black,
                nd(3, Color::Red, None, None),
                nd(5, Color::Red, None, None),
            ),
        );
    }

    #[test]
    fn test_insert_rotate_left() {
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let mut c = tree.root().unwrap_leaf().insert(&5);
        c = c.right_child().unwrap_leaf().insert(&6);
        c.right_child().unwrap_leaf().insert(&7);

        println!("{:?}", tree);

        expect_tree(
            &tree,
            &nd(
                6,
                Color::Black,
                nd(5, Color::Red, None, None),
                nd(7, Color::Red, None, None),
            ),
        );
    }

    #[test]
    fn test_two_rotation_insert() {
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let mut c = tree.root().unwrap_leaf().insert(&5);
        c = c.left_child().unwrap_leaf().insert(&3);
        c.right_child().unwrap_leaf().insert(&4);

        println!("{:?}", tree);

        expect_tree(
            &tree,
            &nd(
                4,
                Color::Black,
                nd(3, Color::Red, None, None),
                nd(5, Color::Red, None, None),
            ),
        );
    }

    #[test]
    fn test_two_rotation_insert_reverse() {
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let mut c = tree.root().unwrap_leaf().insert(&5);
        c = c.right_child().unwrap_leaf().insert(&7);
        c.left_child().unwrap_leaf().insert(&6);

        println!("{:?}", tree);

        expect_tree(
            &tree,
            &nd(6, Black, nd(5, Red, None, None), nd(7, Red, None, None)),
        );
    }

    #[test]
    fn stress_test() {
        let vals: Vec<usize> = vec![
            93, 11, 3, 31, 1, 78, 16, 14, 2, 58, 19, 44, 68, 97, 41, 15, 81, 49, 79, 40, 52, 98,
            91, 23, 95, 67, 30, 43, 62, 25, 96, 6, 100, 72, 37, 42, 38, 61, 74, 99, 39, 84, 50, 55,
            90, 64, 75, 69, 45, 54, 26, 56, 27, 4, 18, 13, 88, 66, 51, 32,
        ];

        let mut t = RedBlackTree::<usize>::new();
        for val in &vals {
            let mut c = t.root();
            while let TreeCursor::Node(nc) = c {
                if nc.key() > val {
                    c = nc.left_child();
                } else {
                    c = nc.right_child();
                }
            }
            let leaf = c.unwrap_leaf();
            leaf.insert(val);
        }

        check_tree(&t);
        check_bst(t.root.get_mut().unwrap());
    }

    #[test]
    fn swap_test() {
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();

        let v5: &usize = &5;
        let v6: &usize = &6;

        let c = tree.root().unwrap_leaf().insert(v5);
        c.right_child().unwrap_leaf().insert(v6);

        tree.swap(v5, v6);

        println!("{:?}", tree);

        expect_tree(&tree, &nd(6, Black, None, nd(5, Red, None, None)));
    }

    #[test]
    fn simple_delete() {
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();
        let mut c = tree.root().unwrap_leaf().insert(&7);
        c.left_child().unwrap_leaf().insert(&4);

        expect_tree(&tree, &nd(7, Black, nd(4, Red, None, None), None));

        c = tree.root().unwrap_node();
        c.delete();

        expect_tree(&tree, &nd(4, Black, None, None));
    }
}
