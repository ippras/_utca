// #![feature(hash_set_entry)]
// #![feature(debug_closure_helpers)]
#![feature(box_patterns)]
#![feature(debug_closure_helpers)]
#![feature(decl_macro)]
#![feature(extend_one)]
#![feature(if_let_guard)]
#![feature(result_option_map_or_default)]

pub use app::App;

mod app;
mod r#const;
mod export;
mod localization;

mod macros;
mod presets;
mod text;

// mod properties;
// mod widgets;

mod utils;

#[cfg(test)]
mod test {
    use radix_trie::{Trie, TrieCommon};

    #[test]
    fn test() {
        let my_list = vec![
            "apple".to_string(),
            "apply".to_string(),
            "apricot".to_string(),
            "api".to_string(),
        ];
        let shortened_list = shorten_with_radix_trie(&my_list);
        println!("{:?}", shortened_list);
        // Вывод: ["appl", "apply", "apr", "api"]

        let another_list = vec![
            "test".to_string(),
            "testing".to_string(),
            "tester".to_string(),
        ];
        let shortened_list2 = shorten_with_radix_trie(&another_list);
        println!("{:?}", shortened_list2);
        // Вывод: ["test", "testi", "teste"]
    }

    fn shorten_with_radix_trie(strings: &[String]) -> Vec<String> {
        // 1. Создаем и заполняем Radix Trie.
        // В качестве значения можно использовать что угодно, например `()`.
        let mut trie = Trie::new();
        for s in strings {
            trie.insert(s, ());
        }

        let mut results = Vec::new();

        // 2. Для каждой строки ищем ее уникальный префикс.
        for s in strings {
            let mut unique_prefix = s.clone(); // По умолчанию, если уникального префикса нет, используем всю строку.

            // Вручную итерируемся по всем возможным префиксам строки `s`,
            // начиная с самого короткого.
            for index in 1..=s.len() {
                let prefix = s[..index].to_string();

                // 3. Получаем поддерево для текущего префикса.
                if let Some(subtrie) = trie.subtrie(&prefix) {
                    // 4. Если в этом поддереве всего один элемент,
                    // значит, мы нашли самый короткий уникальный префикс.
                    if subtrie.len() == 1 {
                        unique_prefix = prefix.to_string();
                        break; // Нашли, выходим из внутреннего цикла.
                    }
                }
            }
            results.push(unique_prefix);
        }
        results
    }
}
