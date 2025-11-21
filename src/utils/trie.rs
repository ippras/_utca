use radix_trie::{Trie, TrieCommon, iter::Children};

// Shortest unique (unambiguous) prefixes of list.
pub fn unique_prefixes(strings: &[String]) -> Vec<String> {
    // Create and fill Radix Trie.
    let mut trie = Trie::new();
    for key in strings {
        trie.insert(key.clone(), ());
    }
    let mut results = Vec::new();
    // For each line, we search for its unique prefix.
    for string in strings {
        // By default, if there is no unique prefix, we use the entire string.
        let mut unique_prefix = string.clone();
        // Iterate through all possible key prefixes, starting with the shortest one.
        for (index, _) in string.char_indices() {
            let prefix = &string[..index];
            println!("prefix: {:?}", prefix);
            // println!("trie.subtrie: {:#?}", trie.subtrie(prefix));
            // println!("trie.children: {:?}", trie.children().collect::<Vec<_>>());
            println!("prefix {:?}", str::from_utf8(trie.prefix().as_bytes()));
            // println!("trie.get: {:?}", trie.get(prefix));
            // println!("is_empty: {:?}", trie.is_empty());
            // println!("is_leaf: {:?}", trie.is_leaf());
            // We obtain a subtree for the current prefix.
            if trie.subtrie(prefix).is_none() {
                println!("subtrie prefix is_none: {:?}", prefix);
                unique_prefix = prefix.to_string();
                // We found it, let's get out of the inner loop.
                break;
            }
        }
        results.push(unique_prefix);
    }
    results
}

fn recursive(children: Children<'_, &str, ()>) {
    for subtrie in children {
        if subtrie.is_leaf() {
            println!("is_leaf {:?}, {:?}", subtrie.key(), subtrie.is_leaf());
        } else {
            println!(
                "is_branch {:?}, {:?}",
                subtrie.len(),
                // subtrie.key(),
                // subtrie.value(),
                subtrie.is_leaf()
            );
            recursive(subtrie.children());
        }
        // if let Some(&path) = trie.key() {
        //     let name = path.rsplit_once('/').map_or(path, |(_, suffix)| suffix);
        //     if trie.is_leaf() {
        //         if let Some(&url) = trie.value() {
        //             ui.horizontal(|ui| {
        //                 if ui.button(CLOUD_ARROW_DOWN).on_hover_text(url).clicked() {
        //                     load_blob(ui.ctx(), url);
        //                 }
        //                 ui.label(name);
        //             });
        //         }
        //     } else {
        //         ui.collapsing(RichText::new(name).heading(), |ui| {
        //             show_children(ui, trie.children());
        //         });
        //     }
        // } else {
        //     show_children(ui, trie.children());
        // }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test0() {
        let strings = vec!["Aaa", "Aab", "Aac", "Baa"];
        let mut trie = Trie::new();
        for key in &strings {
            trie.insert(key, ());
        }
        // t.insert("Stearic", ());
        // t.insert("Linoleic", ());

        // recursive(t.children());
        // for subtrie in t.children() {
        //     if subtrie.is_leaf() {
        //         println!("is_leaf {:?}, {:?}", subtrie.key(), subtrie.is_leaf());
        //     } else {
        //         subtrie.children()
        //     }
        //     // for item in subtrie.is_leaf().iter() {
        //     //     println!("None children {:?}", item);
        //     // }
        // }

        // if let Some(none) = t.subtrie("") {
        //     for subtrie in none.iter() {
        //         println!("None {:?}", subtrie);
        //     }
        //     // if let Some(none) = none.subtrie("Palmit") {
        //     //     for subtrie in none.iter() {
        //     //         println!("None Palmit {:?}", subtrie);
        //     //     }
        //     // }
        // }
        // if let Some(subtrie) = t.subtrie("Palmit") {
        //     for item in subtrie.iter() {
        //         println!("Palmit {:?}", item);
        //     }
        // }
        // println!("---");
        for string in &strings {
            // Iterate through all possible key prefixes, starting with the shortest one.
            for (index, _) in string.char_indices() {
                let prefix = &string.get(..index + 1).unwrap();
                println!(r#"---"{prefix}"---"#);
                if let Some(subtrie) = trie.subtrie(prefix) {
                    println!("prefix {:?}", str::from_utf8(subtrie.prefix().as_bytes()));
                    for item in subtrie.iter() {
                        println!("{prefix} {:?}", item);
                    }
                }
            }
        }
    }

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
