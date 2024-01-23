use crate::ukkonen::SuffixTree;

mod ukkonen;

fn main() {
    let mut suffix_tree = SuffixTree::new();
    suffix_tree.insert_string("ACCCTCCCACTTGGATGCCGCACGTGTCGACTAACCTTACATTGTCCCCCCACCTCCAGACGGTTAACTCTTGAAATGGGGGAATAGCTGCTTGCGCGTG$");

    suffix_tree.print_tree();
    println!("Repeated substrings: {:?}", suffix_tree.get_repeated_substring());
}
