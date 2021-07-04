use std::collections::HashSet;

#[derive(Debug)]
struct Node
{
    i: usize,
    start: usize,
    end: usize,
    parent: Option<usize>,
    lefty: Option<bool>, //The relationship of this node to its parent
    left: Option<usize>,
    right: Option<usize>,
}

impl Node
{
    fn new(i: usize, start: usize, end: usize, lefty: Option<bool>) -> Self {
        Self {
            i,
            start,
            end,
            parent: None,
            lefty,
            left: None,
            right: None,
        }
    }
}

#[derive(Debug)]
pub struct RangeTree
{
    root: usize,
    intervals: HashSet<usize>,
    tree_vec: Vec<Node>,
}

impl RangeTree
{
    pub fn new(start: usize, end: usize) -> Self {
        //Push a new node that has the start and end specified
        let mut tree_vec: Vec<Node> = Vec::new();
        let rootnode: Node = Node::new(0,start,end,None);
        tree_vec.push(rootnode);
        let mut intervals = HashSet::<usize>::new();
        intervals.insert(0);

        Self{
            root: 0,
            intervals: intervals,
            tree_vec: tree_vec,
        }
    }

    pub fn reinit(&mut self, start: usize, end: usize) {
        self.tree_vec.clear();
        let rootnode: Node = Node::new(0,start,end,None);
        self.tree_vec.push(rootnode);
        self.intervals.clear();
        self.intervals.insert(0);
    }

    //Check a packet, traverse and make new stuff as needed
    pub fn add_packet(&mut self, index: usize) {
        //Check a packet, a few cases
        let mut traverser: usize = self.root;
        loop {
            //We are either at a split, or a range
            match self.tree_vec.get_mut(traverser) {
                Some(_) => {
                    //We're good to proceed!
                },
                None => {
                    println!("Somehow attempted to traverse to a non-existent node!");
                    break;
                }
            }
            if self.tree_vec[traverser].start == self.tree_vec[traverser].end {
                //We are at a split, traverse the correct direction
                if index < self.tree_vec[traverser].start {
                    match self.tree_vec[traverser].left {
                        Some(ln) => {
                            traverser = ln;
                        },
                        None => {
                            //We reached a split, tried to go left, but nothing is on the left
                            //This is a repeat packet on a completed interval and can be quietly ignored
                            break;
                        }
                    }
                } else if index > self.tree_vec[traverser].start {
                    match self.tree_vec[traverser].right {
                        Some(rn) => {
                            traverser = rn;
                        },
                        None => {
                            //We reached a split, tried to go left, but nothing is on the left
                            //This is a repeat packet on a completed interval and can be quietly ignored
                            break;
                        }
                    }
                } else {
                    //We are at a split, but we aren't greater of less, it's a repeat packet
                    //This can be quietly ignored, and will happen
                    break;
                }
            } else {
                //We are at a range, we will either narrow it, or split it
                if index == self.tree_vec[traverser].start {
                    self.tree_vec[traverser].start+=1;
                    if self.tree_vec[traverser].start == self.tree_vec[traverser].end {
                        self.intervals.remove(&traverser);
                    }
                    break;
                } else if index == self.tree_vec[traverser].end {
                    self.tree_vec[traverser].start-=1;
                    if self.tree_vec[traverser].start == self.tree_vec[traverser].end {
                        self.intervals.remove(&traverser);
                    }
                    break;
                } else if index > self.tree_vec[traverser].start && index < self.tree_vec[traverser].end{
                    //Turn the current node into a split, remove it from the intervals
                    self.intervals.remove(&traverser);
                    let lowside = self.tree_vec[traverser].start;
                    let highside = self.tree_vec[traverser].end;
                    self.tree_vec[traverser].start = index;
                    self.tree_vec[traverser].end = index;
                    //Push a left and right node for the split
                    self.tree_vec[traverser].left = Some(self.tree_vec.len());
                    self.tree_vec.push(Node::new(self.tree_vec.len(),lowside,index-1,Some(true)));
                    self.tree_vec[traverser].right = Some(self.tree_vec.len());
                    self.tree_vec.push(Node::new(self.tree_vec.len(),index+1,highside,Some(false)));
                    break;
                } else {
                    //We are at an interval, but the packet does not fall within the interval
                    //This can be quiety ignored, and may happen
                    break;
                }
            }
        }
    }
}