use std::collections::HashMap;

// TreePosition: either "root" or child relationship.
// Data/Leaf cursor: pointer to tree node allowing insert/swap.
// TreeNode: enum of either data/leaf node.

// Tree owns nodes; hash table is a cache of pointers to nodes.

#[derive(Clone, Copy)]
enum Color {
    Red,
    Black,
}

#[derive(Clone, Copy)]
enum ChildType {
    Left,
    Right
}

// Nullable holder for node.
type NodeContainer<'node, T> = Option<Box<RedBlackTreeNode<'node, T>>>;

// Pointer to non-null node.

type NodePointer<'pointer, T> = *mut RedBlackTreeNode<'pointer, T>;
type NodeContainerRef<'pointer, 'node, T> = &'pointer mut NodeContainer<'node, T>;

type NodeCache<'keys, T> = HashMap<*const T, NodePointer<'keys, T>>;

/*
#[derive(Clone, Copy)]
enum TreePosition<'position, T> {
    Child(NodePointer<'position, T>, ChildType),
    Root()
}
*/

#[derive(Clone)]
struct RedBlackTreeNode<'node, T> {
    key: &'node T,
    color: Color,
    //position: TreePosition<'node, T>,
    pub left_child: NodeContainer<'node, T>,
    pub right_child: NodeContainer<'node, T>,
}

struct NodeCursor<'cursor, 'tree, T> {
    node: &'cursor mut RedBlackTreeNode<'tree, T>,
    nodes: &'cursor mut HashMap<*const T, NodePointer<'tree, T>>,
}

impl<'cursor, 'tree, T> NodeCursor<'cursor, 'tree, T> {
    pub fn left_child(self) -> TreeCursor<'cursor, 'tree, T> {
        if self.node.left_child.is_none() {
            TreeCursor::leaf_from_position(&mut self.node.left_child, self.nodes)
        } else {
            let v = &mut **self.node.left_child.as_mut().unwrap();
            TreeCursor::from_node(v, self.nodes)
        }
    }
}

struct LeafCursor<'cursor, 'tree, T> {
    position: NodeContainerRef<'cursor, 'tree, T>,
    nodes: &'cursor mut HashMap<*const T, NodePointer<'tree, T>>,
}

impl<'cursor, 'tree, T> LeafCursor<'cursor, 'tree, T> {
    pub fn insert(&mut self, key: &'tree T) {
        let node = RedBlackTreeNode {
            key,
            color: Color::Red,
            left_child: None,
            right_child: None
        };

        *self.position = Some(Box::new(node));

        /*
        self.nodes.insert(key, &mut node);
        let k: *const T = key;
        //let ptr: *const RedBlackTreeNode<T> = &self.tree.nodes[&k];
        let ptr: *mut RedBlackTreeNode<T> = *self.nodes.get(&k).unwrap();
        */

        /*
        match self.position {
            TreePosition::Root => self.tree.root = Some(ptr),
            TreePosition::Child(p, a) => {
                let mut parent = unsafe {&mut *p};
                
                match a {
                    ChildType::Left => parent.left_child = Some(ptr),
                    ChildType::Right => parent.right_child = Some(ptr),
                }
            }
        };
        */
    }
}

enum TreeCursor<'cursor, 'tree, T: 'cursor> {
    Node(NodeCursor<'cursor, 'tree, T>),
    Leaf(LeafCursor<'cursor, 'tree, T>)
}

impl<'cursor, 'tree, T> TreeCursor<'cursor, 'tree, T> {
    pub fn from_node(node: &'cursor mut RedBlackTreeNode<'tree, T>, nodes: &'cursor mut NodeCache<'tree, T>) -> TreeCursor<'cursor, 'tree, T> {
        TreeCursor::Node(
            NodeCursor {
                node,
                nodes
            }
        )
    }

    pub fn leaf_from_position(position: NodeContainerRef<'cursor, 'tree, T>, nodes: &'cursor mut NodeCache<'tree, T>) -> TreeCursor<'cursor, 'tree, T> {
        TreeCursor::Leaf(
            LeafCursor {
                position,
                nodes
            }
        )
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            TreeCursor::Node(node) => {
                let n: &RedBlackTreeNode<'tree, T> = &*node.node;
                Some(n.key)
            },
            _ => None
        }
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

    pub fn get<'cursor>(&'cursor mut self, key: *const T) -> Option<TreeCursor<'cursor, 'tree, T>> {
        let node = unsafe { &mut **self.nodes.get(&key)? };
        Some(TreeCursor::Node(NodeCursor {node, nodes: &mut self.nodes}))
    }

    pub fn root<'cursor>(&'cursor mut self) -> TreeCursor<'cursor, 'tree, T> {
        if self.root.is_none() {
            TreeCursor::leaf_from_position(&mut self.root, &mut self.nodes)
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
        let mut root = tree.root();
        
        let leaf = match &mut root {
            TreeCursor::Leaf(leaf) => leaf,
            _ => panic!("Expected leaf.")
        };
        
        leaf.insert(&4);
        assert_eq!(&4, tree.root().value().unwrap());
    }
}