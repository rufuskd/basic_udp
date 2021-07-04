use std::collections::HashSet;
use std::fmt;

pub struct Node
{
    i: usize,
    pub start: usize,
    pub end: usize,
    parent: Option<usize>,
    lefty: bool, //The relationship of this node to its parent
    left: Option<usize>,
    left_depth: usize,
    right: Option<usize>,
    right_depth: usize,
}

impl Node
{
    fn new(i: usize, parent: Option<usize>, start: usize, end: usize, lefty: bool) -> Self {
        Self {
            i,
            start,
            end,
            parent,
            lefty,
            left: None,
            left_depth: 0,
            right: None,
            right_depth: 0,
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
         .field("Start", &self.start)
         .field("End", &self.end)
         .finish()
    }
}

#[derive(Debug)]
pub struct RangeTree
{
    pub root: usize,
    pub intervals: HashSet<usize>,
    pub tree_vec: Vec<Node>,
}

impl RangeTree
{
    pub fn new(start: usize, end: usize) -> Self {
        //Push a new node that has the start and end specified
        let mut tree_vec: Vec<Node> = Vec::new();
        let rootnode: Node = Node::new(0,None,start,end,false);
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
        let rootnode: Node = Node::new(0,None,start,end,false);
        self.tree_vec.push(rootnode);
        self.intervals.clear();
        self.intervals.insert(0);
    }

    pub fn balance(&mut self, i: usize) {
        //Shrug the tree at a give node to balance it
        if self.tree_vec[i].left_depth > self.tree_vec[i].right_depth {
            //Shrug left case
            //Obtain all relevant node indexes
            let old_root = i;
            let new_root = self.tree_vec[i].left.unwrap();
            let parent = self.tree_vec[old_root].parent;
            let orientation = self.tree_vec[old_root].lefty;
            
            let replaced_child: Option<usize>;
            match self.tree_vec[new_root].right
            {
                Some(rc) => {
                    replaced_child = Some(rc);
                },
                None => {
                    replaced_child = None;
                }
            }

            
            //The old root adopts the replaced child and its parent is switched to the new root
            match replaced_child {
                Some(rc) => {
                    self.tree_vec[old_root].left = Some(rc);
                    self.tree_vec[rc].parent = Some(old_root);
                    self.tree_vec[rc].lefty = true;
                },
                None => {
                    //There is no replaced child to worry about, good to go
                }
            }
            self.tree_vec[old_root].parent = Some(new_root);
            self.tree_vec[old_root].lefty = false;

            self.tree_vec[new_root].parent = parent;
            self.tree_vec[new_root].right = Some(self.tree_vec[old_root].i);            

            match parent {
                Some(p) => {
                    if orientation {
                        self.tree_vec[p].left = Some(new_root);
                    } else {
                        self.tree_vec[p].right = Some(new_root);
                    }
                },
                None => {
                    self.root = new_root;
                }
            }
            
        } else if self.tree_vec[i].right_depth > self.tree_vec[i].left_depth {
            //Shrug right case
            //Obtain all relevant node indexes
            let old_root = i;
            let new_root = self.tree_vec[i].right.unwrap();
            let parent = self.tree_vec[old_root].parent;
            let orientation = self.tree_vec[old_root].lefty;
            
            let replaced_child: Option<usize>;
            match self.tree_vec[new_root].left
            {
                Some(rc) => {
                    replaced_child = Some(rc);
                },
                None => {
                    replaced_child = None;
                }
            }

            
            //The old root adopts the replaced child and its parent is switched to the new root
            match replaced_child {
                Some(rc) => {
                    self.tree_vec[old_root].right = Some(rc);
                    self.tree_vec[rc].parent = Some(old_root);
                    self.tree_vec[rc].lefty = false;
                },
                None => {
                    //There is no replaced child to worry about, good to go
                }
            }
            self.tree_vec[old_root].parent = Some(new_root);
            self.tree_vec[old_root].lefty = true;

            self.tree_vec[new_root].parent = parent;
            self.tree_vec[new_root].right = Some(self.tree_vec[old_root].i);            

            match parent {
                Some(p) => {
                    if orientation {
                        self.tree_vec[p].left = Some(new_root);
                    } else {
                        self.tree_vec[p].right = Some(new_root);
                    }
                },
                None => {
                    self.root = new_root;
                }
            }
        }
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
                            //We reached a split, tried to go right, but nothing is on the right
                            //This is a repeat packet on a completed interval and can be quietly ignored
                            break;
                        }
                    }
                } else if index ==  self.tree_vec[traverser].start {
                    //We reached a split, and we are that packet!
                    self.intervals.remove(&traverser);
                    break;
                }
            } else {
                //We are at a range, we will either narrow it, or split it
                if index == self.tree_vec[traverser].start {
                    self.tree_vec[traverser].start+=1;
                    break;
                } else if index == self.tree_vec[traverser].end {
                    self.tree_vec[traverser].end-=1;
                    break;
                } else if index > self.tree_vec[traverser].start && index < self.tree_vec[traverser].end{
                    //Turn the current node into a split, remove it from the intervals
                    self.intervals.remove(&traverser);
                    let lowside = self.tree_vec[traverser].start;
                    let highside = self.tree_vec[traverser].end;
                    self.tree_vec[traverser].start = index;
                    self.tree_vec[traverser].end = index;
                    
                    //Push a left and right node for the split
                    self.tree_vec[traverser].left_depth = 1;
                    self.tree_vec[traverser].left = Some(self.tree_vec.len());
                    self.intervals.insert(self.tree_vec.len());
                    self.tree_vec.push(Node::new(self.tree_vec.len(),Some(traverser),lowside,index-1,true));
                    
                    self.tree_vec[traverser].right_depth = 1;
                    self.tree_vec[traverser].right = Some(self.tree_vec.len());
                    self.intervals.insert(self.tree_vec.len());
                    self.tree_vec.push(Node::new(self.tree_vec.len(),Some(traverser),index+1,highside,false));

                    //Now adjust depth numbers all the way up
                    let mut depth_traverser = traverser;
                    loop {
                        match self.tree_vec[depth_traverser].parent {
                            Some(d) => {
                                if self.tree_vec[depth_traverser].lefty {
                                    //We are a left child of our parent
                                    //Update the parent's left depth, and then move to the parent
                                    self.tree_vec[d].left_depth+=1;
                                    if self.tree_vec[d].left_depth > self.tree_vec[d].right_depth+1 {
                                        println!("We have a possible rebalance left {:?} right {:?}",self.tree_vec[d].left_depth,self.tree_vec[d].right_depth);
                                    }
                                } else {
                                    //We are a right child of our parent
                                    //Update the parent's right depth, and then move to the parent
                                    self.tree_vec[d].right_depth+=1;
                                    if self.tree_vec[d].right_depth > self.tree_vec[d].left_depth+1 {
                                        println!("We have a possible rebalance left {:?} right {:?}",self.tree_vec[d].left_depth,self.tree_vec[d].right_depth);
                                    }
                                }
                                depth_traverser = d;
                            },
                            None => { break }
                        }
                    }

                    //Now rebalance the tree!  At any node where depths mismatch by 2 or more
                    
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