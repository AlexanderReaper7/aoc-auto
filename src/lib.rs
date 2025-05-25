//! This crate is used to generate the auto_import.rs and mod.rs files for each year/day/part.

use quote::quote;
use std::{
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};

pub mod template;

/// Run this function to generate the auto_import.rs and mod.rs files for each year folder and day file.
/// 
/// # Example
/// 
/// ```rust
/// use aoc_auto::aoc_auto;
/// fn main() {
///    aoc_auto();
/// }
/// ```
pub fn aoc_auto() {
    // get years to make mod files for from src/, each folder is a year formatted as y20XX/
    let years: Vec<String> = Path::new("src/")
        .read_dir()
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| {
            e.path().is_dir() &&
            e.file_name().to_str().unwrap().starts_with("y") &&
            // every character after y is a digit
            e.file_name().to_str().unwrap().chars().skip(1).all(|c| c.is_digit(10))
        })
        .map(|e| e.file_name().to_str().unwrap().to_owned())
        .collect();
    // for each year, get days to include into the mod files for, each day is formatted as dX.rs
    for year in &years {
        let days: Vec<String> = Path::new("src/")
            .join(year.clone())
            .read_dir()
            .unwrap()
            .map(|e| e.unwrap())
            .filter(|e| {
                let filename = e.file_name().into_string().unwrap();
                e.path().is_file() && filename.starts_with("d") && filename.ends_with(".rs") && 
                // every character after d is a digit except for the file extension
                filename.replace(".rs", "").chars().skip(1).all(|c| c.is_digit(10))
            })
            .map(|e| e.file_name().to_str().unwrap().to_owned())
            .collect();

        let days_expr: Vec<syn::Expr> = days
            .iter()
            .map(|e| {
                let d = e.replace(".rs", "");
                syn::parse_str::<syn::Expr>(&d).unwrap()
            })
            .collect();
        let days_num_expr: Vec<syn::Expr> = days
            .iter()
            .map(|e| e.replace("d", "").replace(".rs", ""))
            .map(|e| syn::parse_str::<syn::Expr>(&e).unwrap())
            .collect();
        let mod_code = quote! {
            //! Auto-generated file by build script, do not edit!
            #(pub mod #days_expr;)*

            /// Selects the function for the given day and part
            pub fn select_function(day: u32, part: u32) -> Result<fn(String) -> String, String> {
                match day {
                    #(#days_num_expr =>
                        match part {
                            1 => Ok(#days_expr::part1),
                            2 => Ok(#days_expr::part2),
                            _ => Err("Invalid part!".into()),
                        }
                    ),*
                    _ => Err("Invalid day!".into()),
                }
            }
        };

        let mut mod_file_path = Path::new("src/").join(year).join("mod.rs");
        write_and_format(mod_code.to_string(), &mut mod_file_path);
    }

    let years_expr: Vec<syn::Expr> = years
        .iter()
        .map(|e| syn::parse_str::<syn::Expr>(&e).unwrap())
        .collect();
    let auto_import_file = Path::new("src/auto_import.rs").to_owned();
    let years_mod: Vec<String> = years.iter().map(|e| format!("{}/mod.rs", e)).collect();
    let years_num_expr: Vec<syn::Expr> = years
        .iter()
        .map(|e| e.replace("y", ""))
        .map(|e| syn::parse_str::<syn::Expr>(&e).unwrap())
        .collect();

    let auto_import_code = quote! {
        //! Auto-generated file by build script, do not edit!
        #(
            #[path = #years_mod]
            pub mod #years_expr;
        )*
        /// Selects the function for the given year, day, and part
        pub fn select_function(year: u32, day: u32, part: u32) -> Result<fn(String) -> String, String> {
            match year {
                #(#years_num_expr => Ok(#years_expr::select_function(day, part)?),)*
                _ => Err("Invalid year!".into()),
            }
        }
    };

    write_and_format(auto_import_code.to_string(), &auto_import_file)
}

fn write_and_format(file: String, path: &PathBuf) {
    let syntax_tree = syn::parse_file(&file).unwrap();
    let text = prettyplease::unparse(&syntax_tree);
    let mut file: File = File::create(&path).unwrap();
    file.write_all(text.as_bytes()).unwrap();
}

/// Automatically fill empty day files with the template.
/// # Usage
/// Simply call this function in your build script to automatically fill empty day files with the template.
pub fn auto_template() {
    // Get years to check for empty day files
    let years: Vec<String> = Path::new("src/")
        .read_dir()
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| {
            e.path().is_dir() &&
            e.file_name().to_str().unwrap().starts_with("y") &&
            // every character after y is a digit
            e.file_name().to_str().unwrap().chars().skip(1).all(|c| c.is_digit(10))
        })
        .map(|e| e.file_name().to_str().unwrap().to_owned())
        .collect();

    // For each year, check day files
    for year in &years {
        let days: Vec<PathBuf> = Path::new("src/")
            .join(year.clone())
            .read_dir()
            .unwrap()
            .map(|e| e.unwrap())
            .filter(|e| {
                let filename = e.file_name().into_string().unwrap();
                e.path().is_file() && filename.starts_with("d") && filename.ends_with(".rs") && 
                // every character after d is a digit except for the file extension
                filename.replace(".rs", "").chars().skip(1).all(|c| c.is_digit(10))
            })
            .map(|e| e.path())
            .collect();

        // Check each day file
        for day_path in days {
            // Check if file is empty
            let metadata = std::fs::metadata(&day_path).unwrap();
            if metadata.len() == 0 {
                // File is empty, fill with template
                let day_name = day_path.file_name().unwrap().to_str().unwrap();
                let day_number = day_name.replace("d", "").replace(".rs", "");
                
                // Parse year and day as numbers
                let year_number = year.replace("y", "");
                let year_num: u32 = year_number.parse().unwrap_or(0);
                let day_num: u32 = day_number.parse().unwrap_or(0);
                
                // Try to get title from website, fall back to default if unavailable
                let title = get_title(year_num, day_num)
                    .unwrap_or_else(|| format!("--- Day {} ---", day_number));
                
                // Use template data with real title if available
                let template_data = template::TemplateData {
                    title,
                    ..Default::default()
                };
                
                // Generate template content
                let template_content = template::create_template(template_data);
                
                // Write template to file
                let mut file = File::create(&day_path).unwrap();
                file.write_all(template_content.as_bytes()).unwrap();
                
                println!("Filled empty file: {:?}", day_path);
            }
        }
    }
}

fn get_title(year: u32, day: u32) -> Option<String> {
    // Construct the URL for the day's challenge
    let url = format!("https://adventofcode.com/{}/day/{}", year, day);
    
    // Make an HTTP request to the URL
    let body = match reqwest::blocking::get(&url) {
        Ok(response) => {
            if !response.status().is_success() {
                return None;
            }
            match response.text() {
                Ok(text) => text,
                Err(_) => return None,
            }
        }
        Err(_) => return None,
    };
    
    // Parse the HTML
    let document = scraper::Html::parse_document(&body);
    
    // Select the article with class "day-desc"
    let article_selector = match scraper::Selector::parse("article.day-desc") {
        Ok(selector) => selector,
        Err(_) => return None,
    };
    
    // Select the first h2 within the article
    let h2_selector = match scraper::Selector::parse("h2") {
        Ok(selector) => selector,
        Err(_) => return None,
    };
    
    // Extract the title
    document.select(&article_selector)
        .next()
        .and_then(|article| article.select(&h2_selector).next())
        .map(|h2| {
            // Get the text content of the h2 tag
            let title = h2.text().collect::<Vec<_>>().join("");
            // Return the full title including the "---" parts, with a space prefix
            format!(" {}", title)
        })
}