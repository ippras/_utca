use polars::prelude::StringChunked;
// use radix_trie::{Trie, TrieCommon, iter::Children};
use radix_immutable::StringTrie;

// Shortest unique (unambiguous) prefixes of list.
pub fn unique_prefixes(strings: &StringChunked) -> Vec<&str> {
    // Create and fill Radix Trie.
    let mut trie = StringTrie::new();
    for string in strings {
        if let Some(value) = string {
            let suffix = value.trim_start_matches(['α', 'β', 'γ', '-']);
            if !trie.contains_key(&suffix) {
                println!(r#"trie !contains_key: "{suffix}""#);
                trie = trie.insert(suffix, value.chars().count());
            }
        }
    }
    let mut results = Vec::new();
    //
    for string in strings.into_no_null_iter() {
        // By default, if there is no unique prefix, we use the entire string.
        let mut unique = string;
        // Iterate through all possible suffixes, starting with the shortest one.
        for (index, _) in string.char_indices() {
            let suffix = string.trim_start_matches(['α', 'β', 'γ', '-']);
            let root = &suffix[..index + 1];
            let view_subtrie = trie.view_subtrie(root);
            println!(r#"-root: "{root}" ({})"#, view_subtrie.len());
            if view_subtrie.len() == 1 {
                unique = root;
                // We found it, let's get out of the inner loop.
                break;
            }
        }
        results.push(unique);
    }
    results
}

// string = string.trim_start_matches(['α', 'β', 'γ', '-']);
// .trim_start_matches("cis")
// .trim_start_matches("trans")

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test0() {
        let strings = vec!["Stearic", "Palmitic", "Palmitoleic", "Linoleic"];
        let mut trie = StringTrie::new();
        for &string in &strings {
            trie = trie.insert(string, ());
        }
        // Create a view of the "hel" prefix
        let view = trie.view_subtrie("Pal");
        println!("view.len() {}", view.len());

        assert!(view.exists());
        assert_eq!(view.len(), 2);
        assert!(!view.is_empty());

        for string in strings {
            // Iterate through all possible key prefixes, starting with the shortest one.
            for (index, _) in string.char_indices() {
                let prefix = &string.get(..index + 1).unwrap();
                println!(r#"---"{prefix}"---"#);
                let subtrie = trie.view_subtrie(prefix);
                println!("{prefix} {:?}", subtrie.len());
                // println!("prefix {:?}", str::from_utf8(subtrie.prefix().as_bytes()));
                // for item in subtrie.iter() {
                //     println!("{prefix} {:?}", item);
                // }
            }
        }
    }

    // #[test]
    // fn test() {
    //     let list = vec![
    //         "Stearic".to_string(),
    //         "Palmitic".to_string(),
    //         "Linoleic".to_string(),
    //     ];
    //     assert_eq!(unique_prefixes(&list), ["S", "P", "L"]);
    // }

    #[test]
    fn test1() {
        let list = StringChunked::from_iter([
            "Stearic",
            "Palmitic",
            "Palmitoleic",
            "Linoleic",
            "α-Linolenic",
            "γ-Linolenic",
        ]);
        println!("unique_prefixes: {:?}", unique_prefixes(&list));
        // assert_eq!(
        //     unique_prefixes(&list),
        //     [
        //         "S",
        //         "Palmiti",
        //         "Palmito",
        //         "Linolei",
        //         "α-Linolen",
        //         "γ-Linolen"
        //     ]
        // );
    }
}
