use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct Node {
    name: i32,
    edges: HashMap<char, Rc<RefCell<Edge>>>,
    suffix_link: Option<Rc<RefCell<Node>>>,
}

pub struct Edge {
    start_index: usize,
    end_index: Rc<RefCell<usize>>,
    node: Option<Rc<RefCell<Node>>>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct SuffixTree {
    root: Rc<RefCell<Node>>,
    repeated_substring: Vec<String>,
    string_vec: Vec<char>,
    string: String,
}

impl Edge {
    pub fn new(start_index: usize, end_index: Rc<RefCell<usize>>) -> Self {
        Self {
            start_index,
            end_index,
            node: None,
        }
    }

    pub fn add_new_node(&mut self, node: Rc<RefCell<Node>>) {
        self.node = Some(node);
    }
}

impl Node {
    pub fn new(suffix_link: Option<Rc<RefCell<Node>>>, name: i32) -> Self {
        Self {
            edges: HashMap::new(),
            suffix_link,
            name,
        }
    }

    pub fn add_new_edge(&mut self, edge: Rc<RefCell<Edge>>, char: char) {
        self.edges.insert(char, edge);
    }

    pub fn add_new_suffix_link(&mut self, node: Rc<RefCell<Node>>) {
        self.suffix_link = Some(node);
    }
}

impl SuffixTree {
    pub fn new() -> Self {
        let root = Rc::new(RefCell::new(Node::new(None, 0)));
        let mut inner_root = root.borrow_mut();
        inner_root.add_new_suffix_link(root.clone());
        Self {
            root: root.clone(),
            repeated_substring: vec![],
            string_vec: vec![],
            string: "".to_owned(),
        }
    }

    pub fn insert_string(&mut self, string: &str) {
        let mut active_node = self.root.clone();
        let mut active_edge: Option<char> = None;
        let mut active_length = 0;
        let mut remainder = 0;
        let end = Rc::new(RefCell::new(0));
        self.string_vec = string.chars().collect();
        self.string = string.to_owned();
        let mut name = 1;
        let mut repeated_substrings = HashSet::new();
        for i in 0..string.len() {
            *end.borrow_mut() += 1;
            remainder += 1;
            if remainder > 10 {
                repeated_substrings.insert(string[i - 10..i].to_string());
            }
            let mut prev_node: Option<Rc<RefCell<Node>>> = None;
            while remainder > 0 {
                if active_edge.is_none() {
                    let inner_active_node = active_node.borrow();
                    let edge_found = inner_active_node.edges.get(&self.string_vec[i]);
                    if edge_found.is_none() {
                        drop(inner_active_node);
                        self.new_leaf_branch_from_existing_node(
                            active_node.clone(),
                            self.string_vec[i],
                            i,
                            end.clone(),
                        );
                        let active_node_cloned = active_node.clone();
                        let inner_active_node = active_node_cloned.borrow();
                        active_node = inner_active_node.suffix_link.clone().unwrap();
                        remainder -= 1;
                        if active_node == self.root && remainder > 0 {
                            active_length = remainder - 1;
                            active_edge = Some(self.string_vec[i - active_length]);
                        }
                        if active_length == 0 {
                            active_edge = None;
                        }
                        // traverse
                        if active_edge.is_some() {
                            (active_node, active_edge) =
                                self.traverse_node(active_node, active_edge, &mut active_length, i);
                        }
                    } else {
                        active_edge = Some(self.string_vec[i]);
                        active_length += 1;
                        break;
                    }
                } else {
                    let active_node_cloned = active_node.clone();
                    let inner_active_node = active_node_cloned.borrow();
                    let edge = inner_active_node
                        .edges
                        .get(&active_edge.unwrap())
                        .unwrap()
                        .borrow();
                    let edge_length = *edge.end_index.borrow() - edge.start_index;
                    if edge_length == active_length {
                        active_node = edge.node.clone().unwrap();
                        active_length = 0;
                        active_edge = None;
                        continue;
                    }
                    if self.string_vec[edge.start_index + active_length] == self.string_vec[i] {
                        active_length += 1;
                        (active_node, active_edge) = self.traverse_node(
                            active_node.clone(),
                            active_edge,
                            &mut active_length,
                            i,
                        );
                        break;
                    } else {
                        let edge_length = *edge.end_index.borrow() - edge.start_index;
                        let next_node = edge.node.clone();
                        drop(edge);
                        drop(inner_active_node);

                        let new_node = if edge_length == active_length {
                            active_node = next_node.unwrap();
                            self.new_leaf_branch_from_existing_node(
                                active_node.clone(),
                                self.string_vec[i],
                                i,
                                end.clone(),
                            );
                            Some(active_node)
                        } else {
                            Some(self.break_branch(
                                active_node.clone(),
                                active_length,
                                active_edge,
                                end.clone(),
                                i,
                                name,
                            ))
                        };
                        name += 1;
                        let new_unwrap = new_node.clone().unwrap();
                        let mut new_borrowed = new_unwrap.borrow_mut();
                        new_borrowed.suffix_link = Some(self.root.clone());
                        drop(new_borrowed);
                        if let Some(prev) = prev_node.clone() {
                            prev.borrow_mut().suffix_link = new_node.clone();
                        }
                        prev_node = new_node;
                        let inner_active_node = active_node_cloned.borrow();
                        active_node = inner_active_node.suffix_link.clone().unwrap();
                        remainder -= 1;
                        if active_node == self.root {
                            active_length = remainder - 1;
                            active_edge = Some(self.string_vec[i - remainder + 1]);
                        }
                        (active_node, active_edge) = self.traverse_node(
                            active_node.clone(),
                            active_edge,
                            &mut active_length,
                            i,
                        );
                        if active_length == 0 {
                            active_edge = None;
                        }
                    }
                }
            }
        }
        self.repeated_substring = repeated_substrings.iter().cloned().collect();
    }

    fn new_leaf_branch_from_existing_node(
        &mut self,
        active_node: Rc<RefCell<Node>>,
        char: char,
        start_index: usize,
        end_index: Rc<RefCell<usize>>,
    ) {
        let new_edge = Edge::new(start_index, end_index);
        active_node
            .borrow_mut()
            .add_new_edge(Rc::new(RefCell::new(new_edge)), char);
    }

    fn break_branch(
        &mut self,
        active_node: Rc<RefCell<Node>>,
        active_length: usize,
        active_edge: Option<char>,
        index: Rc<RefCell<usize>>,
        i: usize,
        name: i32,
    ) -> Rc<RefCell<Node>> {
        let binding = active_node.borrow_mut();
        let existing_edge = binding.edges.get(&active_edge.unwrap()).unwrap().clone();
        let mut existing_edge_borrow = existing_edge.borrow_mut();
        let start_index = existing_edge_borrow.start_index;
        let new_edge = Edge::new(i, index);
        let mut broken_upper_edge = Edge::new(
            start_index,
            Rc::new(RefCell::new(start_index + active_length)),
        );
        existing_edge_borrow.start_index = start_index + active_length;
        let mut new_node = Node::new(None, name);
        new_node.add_new_edge(Rc::new(RefCell::new(new_edge)), self.string_vec[i]);
        new_node.add_new_edge(
            existing_edge.clone(),
            self.string_vec[start_index + active_length],
        );
        let new_node = Rc::new(RefCell::new(new_node));
        drop(binding);
        let mut binding_2 = active_node.borrow_mut();
        broken_upper_edge.add_new_node(new_node.clone());
        binding_2.add_new_edge(
            Rc::new(RefCell::new(broken_upper_edge)),
            active_edge.unwrap(),
        );
        new_node
    }

    fn traverse_node(
        &self,
        mut active_node: Rc<RefCell<Node>>,
        mut active_edge: Option<char>,
        active_length: &mut usize,
        i: usize,
    ) -> (Rc<RefCell<Node>>, Option<char>) {
        let active_node_cloned = active_node.clone();
        let active_borrowed = active_node_cloned.borrow();
        let mut edge = active_borrowed.edges.get(&active_edge.unwrap()).cloned();
        if let Some(e) = edge.clone() {
            let edge_borrowed = e.borrow();
            let mut edge_length = *edge_borrowed.end_index.borrow() - edge_borrowed.start_index;
            while active_length > &mut edge_length {
                let edge_cloned = edge.clone();
                let edge_unwrap = edge_cloned.unwrap();
                let edge_borrowed = edge_unwrap.borrow();
                active_node = edge_borrowed.node.clone().unwrap();
                *active_length -= edge_length;
                active_edge = Some(self.string_vec[i - *active_length]);
                let active_node_cloned = active_node.clone();
                let active_borrowed = active_node_cloned.borrow();
                edge = active_borrowed.edges.get(&active_edge.unwrap()).cloned();
                match edge.clone() {
                    Some(e) => {
                        let edge_borrowed = e.borrow();
                        edge_length = *edge_borrowed.end_index.borrow() - edge_borrowed.start_index;
                    }
                    None => todo!(),
                };
            }
        };
        (active_node, active_edge)
    }

    pub fn print_tree(&self) {
        println!("{}", self.root.borrow().name);
        self.print_sub_tree(&"".to_string(), self.root.clone());
        println!();
    }

    fn print_sub_tree(&self, prefix: &String, node: Rc<RefCell<Node>>) {
        let node_borrowed = node.borrow();
        if node_borrowed.edges.is_empty() {
            return;
        }
        print!("{prefix}");
        let n_children = node_borrowed.edges.len();
        let edges: Vec<Rc<RefCell<Edge>>> = node_borrowed.edges.values().cloned().collect();
        if n_children > 1 {
            print!("├── ");
        } else {
            print!("");
        }

        for (i, _) in edges.iter().enumerate().take(n_children) {
            let c = &edges[i];
            let c_borrowed = c.borrow();
            let c_node = c_borrowed.node.clone();
            if i < n_children - 1 {
                if i > 0 {
                    print!("{}├── ", prefix);
                }
                let print_strand = n_children > 1 && c_node.is_none();
                let new_prefix = if print_strand {
                    prefix.to_owned() + "|\t"
                } else {
                    prefix.to_owned() + "\t"
                };
                print!(
                    "{}",
                    &self.string[c_borrowed.start_index..*c_borrowed.end_index.borrow()]
                );
                match c_node.clone() {
                    Some(node) => println!(" ── {}", node.borrow().name),
                    None => println!(),
                }
                if let Some(n) = c_node {
                    self.print_sub_tree(&new_prefix, n);
                } else {
                    continue;
                }
            } else {
                if n_children > 1 {
                    print!("{}└── ", prefix);
                } else {
                    print!("└── ");
                }
                print!(
                    "{}",
                    &self.string[c_borrowed.start_index..*c_borrowed.end_index.borrow()]
                );
                match c_node.clone() {
                    Some(node) => println!(" ── {}", node.borrow().name),
                    None => println!(),
                }
                if let Some(n) = c_node {
                    self.print_sub_tree(&format!("{}\t", prefix), n);
                } else {
                    continue;
                }
            }
        }
    }

    pub fn get_repeated_substring(&self) -> Vec<String> {
        self.repeated_substring.clone()
    }
}
