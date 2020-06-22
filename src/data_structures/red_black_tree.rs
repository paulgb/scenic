use crate::data_structures::expanding_vec::ExpandingVector;

#[derive(Clone, Copy)]
enum Color {
    Red,
    Black,
}

#[derive(Clone)]
pub struct RedBlackTreeNode<T> {
    color: Color,
    pub value: T,
}

pub struct RedBlackTree<T> {
    values: ExpandingVector<RedBlackTreeNode<T>>,
}

trait HasParent<'a, T> {
    fn parent(self) -> DataNodePointer<'a, T>;
}

pub struct LeafNodePointer<'a, T> {
    index: usize,
    tree: &'a mut RedBlackTree<T>,
}

impl<'a, T: Clone> HasParent<'a, T> for LeafNodePointer<'a, T> {
    fn parent(self) -> DataNodePointer<'a, T> {
        match self.tree.get_node(self.index >> 1) {
            NodePointer::Node(n) => n,
            _ => panic!("Expected parent of leaf node to be a data node."),
        }
    }
}

impl<'a, T: Clone> HasParent<'a, T> for DataNodePointer<'a, T> {
    fn parent(self) -> DataNodePointer<'a, T> {
        match self.tree.get_node(self.index >> 1) {
            NodePointer::Node(n) => n,
            _ => panic!("Expected parent of data node to be a data node."),
        }
    }
}

pub struct DataNodePointer<'a, T> {
    index: usize,
    tree: &'a mut RedBlackTree<T>,
}

impl<'a, T: Clone> DataNodePointer<'a, T> {
    pub fn data<'b>(&'b self) -> &T {
        &self.tree.get(self.index).unwrap()
    }
}

impl<'a, T: Clone> LeafNodePointer<'a, T> {
    pub fn insert(self, data: T) {
        if self.index == 1 {
            self.tree.values[1] = Some(RedBlackTreeNode {
                color: Color::Red,
                value: data,
            })
        } else {
            unimplemented!()
        }
    }
}

pub enum NodePointer<'a, T> {
    Leaf(LeafNodePointer<'a, T>),
    Node(DataNodePointer<'a, T>),
}

impl<'a, T: Clone> DataNodePointer<'a, T> {
    pub fn left_child(self) -> NodePointer<'a, T> {
        let index = self.index << 1;
        self.tree.get_node(index)
    }

    pub fn right_child(self) -> NodePointer<'a, T> {
        let index = (self.index << 1) + 1;
        self.tree.get_node(index)
    }
}

impl<'a, T: Clone> RedBlackTree<T> {
    pub fn new() -> RedBlackTree<T> {
        RedBlackTree {
            values: ExpandingVector::new(),
        }
    }

    fn get_node<'b>(&'b mut self, index: usize) -> NodePointer<'b, T> {
        if let Some(_) = self.get(index) {
            NodePointer::Node(DataNodePointer { index, tree: self })
        } else {
            NodePointer::Leaf(LeafNodePointer { index, tree: self })
        }
    }

    fn get(&'a self, index: usize) -> Option<&'a T> {
        match &self.values[index] {
            Some(x) => Some(&x.value),
            _ => None,
        }
    }

    pub fn root<'b>(&'b mut self) -> NodePointer<'b, T> {
        self.get_node(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_insert() {
        let mut tree: RedBlackTree<usize> = RedBlackTree::new();

        let ln = match tree.root() {
            NodePointer::Leaf(ln) => ln,
            NodePointer::Node(_) => panic!("Empty tree should have leaf root node."),
        };

        ln.insert(5);

        let np = match tree.root() {
            NodePointer::Node(np) => np,
            NodePointer::Leaf(_) => panic!("Root should be replaced with leaf."),
        };

        assert_eq!(&5, np.data());
    }
}
