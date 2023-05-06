use serde::Deserialize;
use walkdir::WalkDir;
use comrak::{markdown_to_html, ComrakOptions};
use std::borrow::Borrow;
use std::env;
use std::fs;
use std::collections::HashMap;
use titlecase::titlecase;

#[derive(Deserialize, Debug)]
struct Page {
    meta: Meta,
    boxes: Vec<Box>,
}

#[derive(Deserialize, Debug)]
struct Meta {
    title: String,
    description: String,
    author: String,
    background: String,
}

#[derive(Deserialize, Debug)]
struct Box {
    parts: Vec<Part>,
}

#[derive(Deserialize, Debug)]
struct Part {
    kind: String,
    content: String,
}

fn main() {
    // Look, this is really goofy probably, maybe I could go about it a better way.. but it works!!
    let mut path_fancy_names: HashMap<String, String> = HashMap::new();

    // Clear the output path
    for entry in WalkDir::new("../stingoutput").into_iter().filter_map(|e| e.ok()) {
        let directory_path = &entry.path().clone().to_str().unwrap();
        
        if directory_path == &"../stingoutput" || directory_path.starts_with("../stingoutput\\.git") {
            continue;
        }
        // Skip the entry if there's an error in the metadata
        if entry.metadata().is_err() {
            println!("Error reading metadata of file {:?}!!!! Q///Q", entry.path());
            continue;
        }
        let metadata = entry.metadata().unwrap();
        // If the entry is a directory, create that directory in the output folder
        if metadata.is_dir() {
            fs::remove_dir_all(directory_path).unwrap();
            continue;
        } else {
            fs::remove_file(directory_path).unwrap()
        }
    }
    //fs::remove_dir_all("output/").unwrap();
    //fs::create_dir("output/").unwrap();

    let html_template = fs::read_to_string("res/template.html")
        .expect("Should have been able to read the file");

    for entry in WalkDir::new("res/pages").into_iter().filter_map(|e| e.ok()) {
        // Skip the entry if there's an error in the metadata
        if entry.metadata().is_err() {
            println!("Error reading metadata of file {:?}!!!! Q///Q", entry.path());
            continue;
        }

        let directory_path = &entry.path().clone().to_str().unwrap()[9..];
        let d_path_split: Vec<&str> = directory_path.split("\\").collect();

        let metadata = entry.metadata().unwrap();
        // If the entry is a directory, create that directory in the output folder
        if metadata.is_dir() {
            path_fancy_names.insert(directory_path.to_string(), titlecase(&d_path_split[d_path_split.len()-1].replace("-", " ")));

            fs::create_dir_all("../stingoutput/".to_owned() + directory_path).ok();
            continue;
        }
        // If the entry is a file that isn't gonna be parsed, copy it over
        if !directory_path.ends_with("index.toml") {
            fs::copy(entry.path(), "../stingoutput/".to_owned() + directory_path).ok();
            continue;
        }
        // If we're dealing with a file needing to be parsed...
        // Read the toml file

        let contents = fs::read_to_string(entry.path())
            .expect("Should have been able to read the file");

        let page: Page = toml::from_str(&contents).unwrap();

        path_fancy_names.insert(d_path_split[..d_path_split.len()-1].join("\\").to_string(), page.meta.title.to_owned());

        let mut page_content = String::new();
        for b in page.boxes {
            page_content += "<div class=\"box\">\n";
            for p in b.parts {
                let j: String = "<div class=\"".to_owned() + &p.kind.to_string() + "\">\n\n" + &p.content + "\n</div>";
                page_content += &j;
            }
            page_content += "</div>\n";
        }
        // Options for markdown parser
        let mut options = ComrakOptions::default();
        options.render.unsafe_ = true;
        options.extension.strikethrough = true;
        
        let markdown: String = markdown_to_html(&page_content, &options);
        
        // Create the navigation bar / breadcrumbs
        let mut navbar = String::from("<div id=\"topnav\"><ul class=\"breadcrumb\"><li>Welcome to my website!</li></ul>\n</div>");
        if directory_path != "\\index.toml" {
            navbar = String::from("<div id=\"topnav\">\n<ul class=\"breadcrumb\">\n");
            let mut j = 2;
            for &i in &d_path_split[..d_path_split.len()-2] {
                if i == "" { navbar += "<li><a href=\"/\">Home</a></li>"; continue; }
                let link = "<li><a href=\"".to_owned() + &d_path_split[0..j].join("/") + "/\">" + path_fancy_names.get(&d_path_split[..j].join("\\").to_string()).unwrap_or(&"Q//Q".to_owned()) + "</a></li>";
                navbar += &link;
                j+=1;
            }
            navbar += &("<li>".to_owned() + &page.meta.title + "</li>\n</ul></div>");
        }
        
        fs::write("../stingoutput/".to_owned() + &directory_path[..directory_path.len()-10] + "/index.html", 
            html_template
            .replace("{content}", &markdown)
            .replace("{title}", &page.meta.title)
            .replace("{description}", &page.meta.description)
            .replace("{author}", &page.meta.author)
            .replace("{background}", &page.meta.background)
            .replace("{topnav}", &navbar)
            //.replace("<img", "<span class=\"imgborder\"><img")
            .replace("<imgrs50><img", "<img style='width: 50%; height: auto;'")
            .replace("<imgrsorig><img", "<img style='width: auto; height: auto;'"))
            .expect("Unable to write file");
    }
}