use radix_trie::{Trie, TrieCommon};

// Shortest unique (unambiguous) prefixes of list.
pub fn unique_prefixes(list: &[String]) -> Vec<String> {
    // Create and fill Radix Trie.
    let mut trie = Trie::new();
    for key in list {
        trie.insert(key.clone(), ());
    }
    let mut results = Vec::new();
    // For each line, we search for its unique prefix.
    for key in list {
        // By default, if there is no unique prefix, we use the entire string.
        let mut unique_prefix = key.clone();
        // Iterate through all possible key prefixes, starting with the shortest one.
        for (index, _) in key.char_indices() {
            let prefix = &key[..index];
            println!("prefix: {:?}", prefix);
            // println!("trie.get: {:?}", trie.get(prefix));
            println!("trie.subtrie: {:?}", trie.subtrie(prefix));
            // println!("is_empty: {:?}", trie.is_empty());
            // println!("is_leaf: {:?}", trie.is_leaf());
            // We obtain a subtree for the current prefix.
            if trie.subtrie(prefix).is_none() {
                println!("prefix: {:?}", prefix);
                unique_prefix = prefix.to_string();
                // We found it, let's get out of the inner loop.
                break;
            }
        }
        results.push(unique_prefix);
    }
    results
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let list = vec![
            "Stearic".to_string(),
            "Palmitic".to_string(),
            "Linoleic".to_string(),
        ];
        assert_eq!(unique_prefixes(&list), ["S", "P", "L"]);
    }

    #[test]
    fn test1() {
        let list = vec![
            "Stearic".to_string(),
            "Palmitic".to_string(),
            "Palmitoleic".to_string(),
            "Linoleic".to_string(),
        ];
        assert_eq!(unique_prefixes(&list), ["S", "Palmiti", "Palmito", "L"]);
    }
}
