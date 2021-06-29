#[derive(Debug,Ord)]

struct Node<T>
where T: Ord
{
    i: usize,
    start: T,
    end: T,
    parent: Option<usize>,
    lefty: Option<bool>, //The relationship of this node to its parent
    left: usize,
    right: usize,
}

#[derive(Debug)]
pub struct RangeTree<Node<T>>
where T: Ord
{
    tree_vec: Vec<Node<T>>,
}

impl<T> Node<T>
where T: Ord
{
   fn new(i: usize, start: T, end: T) -> Self {
       Self {
           i,
           start,
           end,
           parent: None,
           lefty: None,
           left: None,
           right: None,
       }
   }
   fn add_left(&self, i: usize, nexti: usize, packet: T) -> Self {
        Self {
            i: nexti,
            start: self.left,
            end: packet,
            parent: Some(i),
            lefty: Some(true),
            left: None,
            right: None,
        }
   }
   fn add_right(&self, i: usize, nexti: usize, packet: T) -> Self {
        Self {
            i: nexti,
            start: packet,
            end: self.right,
            parent: Some(i),
            lefty: Some(false),
            left: None,
            right: None,
        }
    }
}


impl<T> RangeTree<T>
where T: Ord
{
    fn new(start: usize, end: usize) {
        //Push a new node that has the start and end specified
        Self {
            tree_vec.push(Node::new(0,start,end,None));
        }
    }
    //TODO check a packet, traverse and make new stuff as needed
    fn add_packet(&mut self, index: T) {
        //Check a packet, a few cases
        //If the tree is empty,
        if tree_vec.len() == 0 {

        }
    }
}