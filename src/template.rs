use quote::quote;
pub struct TemplateData {
    pub title: String,
    pub test_data_1: String,
    pub test_result_1: String,
    pub test_data_2: String,
    pub test_result_2: String,
}
impl Default for TemplateData {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            test_data_1: "".to_string(),
            test_result_1: "".to_string(),
            test_data_2: "".to_string(),
            test_result_2: "".to_string(),
        }
    }
}

pub fn create_template(data: TemplateData) -> String {
    let title = data.title;
    let test_data_1 = data.test_data_1;
    let test_result_1 = data.test_result_1;
    let test_data_2 = data.test_data_2;
    let test_result_2 = data.test_result_2;

    let quote = quote! {
    #![doc=#title]

    pub fn part1(input: String) -> String {
        unimplemented!()
    }

    pub fn part2(input: String) -> String {
        unimplemented!()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        static INPUT1: &str = #test_data_1;
        static RESULT1: &str = #test_result_1;
        static INPUT2: &str = #test_data_2;
        static RESULT2: &str = #test_result_2;

        #[test]
        fn part1() {
            assert_eq!(super::part1(INPUT1.to_string()), RESULT1.to_string());
        }

        #[test]
        fn part2() {
            assert_eq!(super::part2(INPUT2.to_string()), RESULT2.to_string());
        }
    }
    };
    let syntax_tree = syn::parse_file(&quote.to_string()).unwrap();
    let text = prettyplease::unparse(&syntax_tree);
    text
}
