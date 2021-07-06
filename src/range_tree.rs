use std::collections::HashSet;
use std::fmt;

pub struct Node {
    pub start: usize,
    pub end: usize,
    parent: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
    depth: usize,
}

impl Node {
    fn new(parent: Option<usize>, start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            parent,
            left: None,
            right: None,
            depth: 1,
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
pub struct RangeTree {
    pub root: usize,
    pub intervals: HashSet<usize>,
    pub tree_vec: Vec<Node>,
}

impl RangeTree
{
    pub fn new(start: usize, end: usize) -> Self {
        //Push a new node that has the start and end specified
        let mut tree_vec: Vec<Node> = Vec::new();
        let rootnode: Node = Node::new(None,start,end);
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
        let rootnode: Node = Node::new(None,start,end);
        self.root = 0;
        self.tree_vec.push(rootnode);
        self.intervals.clear();
        self.intervals.insert(0);
    }

    pub fn balance(&mut self, i: usize) -> usize{
        let old_root: usize = i;
        let new_root: usize;
        let replaced_child: Option<usize>;
        let left_depth: usize;
        let right_depth: usize;
        //Shrug the tree at a given node to balance it
        match self.tree_vec[i].left {
            Some(ld) => {left_depth = self.tree_vec[ld].depth},
            None => {left_depth = 0}
        }
        match self.tree_vec[i].right {
            Some(rd) => {right_depth = self.tree_vec[rd].depth},
            None => {right_depth = 0}
        }
        if left_depth > right_depth {
            //Shrug left case
            //Obtain all relevant node indexes
            match self.tree_vec[i].left {
                Some(nr) => {
                    new_root = nr;
                },
                None => {
                    println!("Attempting to balance favoring a node that doesn't exist!");
                    return i
                }
            }
            let parent = self.tree_vec[old_root].parent;
            
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
                },
                None => {
                    //There is no replaced child to worry about, good to go
                }
            }

            self.tree_vec[new_root].parent = parent;
            self.tree_vec[new_root].right = Some(old_root);            

            match parent {
                Some(p) => {
                    if self.tree_vec[new_root].end < self.tree_vec[p].start {
                        self.tree_vec[p].left = Some(new_root);
                    } else {
                        self.tree_vec[p].right = Some(new_root);
                    }
                },
                None => {
                    self.root = new_root;
                }
            }

            self.tree_vec[old_root].parent = Some(new_root);
 
            //Just need to fix the depth numbers on the old root, the new root will still be right
            let old_root_left_depth: usize;
            let old_root_right_depth: usize;

            match self.tree_vec[old_root].left {
                Some(ld) => {old_root_left_depth = self.tree_vec[ld].depth},
                None => {old_root_left_depth = 0}
            }
            match self.tree_vec[old_root].right {
                Some(rd) => {old_root_right_depth = self.tree_vec[rd].depth},
                None => {old_root_right_depth = 0}
            }

            if old_root_left_depth > old_root_right_depth {
                self.tree_vec[old_root].depth = old_root_left_depth+1;
            } else {
                self.tree_vec[old_root].depth = old_root_right_depth+1;
            }

            return new_root

        } else if left_depth < right_depth {
            //Shrug left case
            //Obtain all relevant node indexes
            match self.tree_vec[i].right {
                Some(nr) => {
                    new_root = nr;
                },
                None => {
                    println!("Attempting to balance favoring a node that doesn't exist!");
                    return i
                }
            }
            let parent = self.tree_vec[old_root].parent;
            
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
                },
                None => {
                    //There is no replaced child to worry about, good to go
                }
            }

            self.tree_vec[new_root].parent = parent;
            self.tree_vec[new_root].left = Some(old_root);            

            match parent {
                Some(p) => {
                    if self.tree_vec[new_root].start > self.tree_vec[p].end {
                        self.tree_vec[p].right = Some(new_root);
                    } else {
                        self.tree_vec[p].left = Some(new_root);
                    }
                },
                None => {
                    self.root = new_root;
                }
            }

            self.tree_vec[old_root].parent = Some(new_root);

            //Just need to fix the depth numbers on the old root, the new root will still be right
            let old_root_left_depth: usize;
            let old_root_right_depth: usize;

            match self.tree_vec[old_root].left {
                Some(ld) => {old_root_left_depth = self.tree_vec[ld].depth},
                None => {old_root_left_depth = 0}
            }
            match self.tree_vec[old_root].right {
                Some(rd) => {old_root_right_depth = self.tree_vec[rd].depth},
                None => {old_root_right_depth = 0}
            }

            if old_root_left_depth > old_root_right_depth {
                self.tree_vec[old_root].depth = old_root_left_depth+1;
            } else {
                self.tree_vec[old_root].depth = old_root_right_depth+1;
            }

            return new_root
        }

        i
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
                    println!("Somehow attempted to traverse to a non-existent node {:?}",traverser);
                    println!("Root's got {:?} {:?}",self.tree_vec[self.root].start,self.tree_vec[self.root].end);
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
                    //self.tree_vec[traverser].depth += 1;

                    self.tree_vec[traverser].left = Some(self.tree_vec.len());
                    self.intervals.insert(self.tree_vec.len());
                    self.tree_vec.push(Node::new(Some(traverser),lowside,index-1));
                    
                    self.tree_vec[traverser].right = Some(self.tree_vec.len());
                    self.intervals.insert(self.tree_vec.len());
                    self.tree_vec.push(Node::new(Some(traverser),index+1,highside));

                    //Now adjust depths and check for rebalance opportunities all the way up
                    let mut depth_traverser = traverser;
                    loop {
                        let left_depth: usize;
                        let right_depth: usize;
                        let parent: Option<usize>;
                        parent = self.tree_vec[depth_traverser].parent;
                        match self.tree_vec[depth_traverser].left {
                            Some(ld) => {left_depth = self.tree_vec[ld].depth},
                            None => {left_depth = 0}
                        }
                        match self.tree_vec[depth_traverser].right {
                            Some(rd) => {right_depth = self.tree_vec[rd].depth},
                            None => {right_depth = 0}
                        }

                        if left_depth > right_depth {
                            self.tree_vec[depth_traverser].depth = left_depth+1;
                        } else {
                            self.tree_vec[depth_traverser].depth = right_depth+1;
                        }

                        if left_depth > right_depth+1 || right_depth > left_depth+1 {
                            /*println!("We have a chance to rebalance - Left: {:?} Right: {:?}",left_depth,right_depth);
                            let troot = self.balance(depth_traverser);
                            match self.tree_vec[troot].left {
                                Some(ld) => {println!("Left {:?}",self.tree_vec[ld].depth)},
                                None => {println!("Left is 0")}
                            }
                            match self.tree_vec[troot].right {
                                Some(rd) => {println!("Right {:?}",self.tree_vec[rd].depth)},
                                None => {println!("Right is 0")}
                            }*/
                            self.balance(depth_traverser);
                        }
                        match parent {
                            Some(d) => {
                                depth_traverser = d;
                            },
                            None => { break }
                        }
                    }

                    break;
                } else {
                    //We are at an interval, but the packet does not fall within the interval, This can be quiety ignored, and may happen
                    break;
                }
            }
        }
    }
}