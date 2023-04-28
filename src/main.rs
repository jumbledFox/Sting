use serde::Deserialize;
use walkdir::WalkDir;
use comrak::{markdown_to_html, ComrakOptions};
use std::env;
use std::fs;

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
    content: String
}

fn main() {
    // Clear the output path
    fs::remove_dir_all("output/").unwrap();
    fs::create_dir("output/").unwrap();

    let template = fs::read_to_string("res/template.html")
        .expect("Should have been able to read the file");

    for entry in WalkDir::new("res/pages").into_iter().filter_map(|e| e.ok()).filter(|d| if let Some(e) = d.path().extension() { e == "toml" } else {false}) {
        let directory_path = &entry.path().parent().to_owned().unwrap().to_str().unwrap()[9..];
        // Create the directories
        fs::create_dir_all("output/".to_owned() + &directory_path);
        // Read the toml file
        let contents = fs::read_to_string(entry.path())
            .expect("Should have been able to read the file");

        let page: Page = toml::from_str(&contents).unwrap();

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

        
        let markdown = markdown_to_html(&page_content, &options);
        
        let mut navbar = String::from("<div id=\"topnav\"><ul class=\"breadcrumb\"><li>‎</li></ul>\n</div>");
        if directory_path != "" {
            navbar = String::from("<div id=\"topnav\">\n<ul class=\"breadcrumb\">\n");
            let d_path_split: Vec<&str> = directory_path.split("\\").collect();
            let mut j = 2;
            for &i in &d_path_split[..d_path_split.len()-1] {
                if i == "" { navbar += "<li><a href=\"/output/index.html\">Home</a></li>"; continue; }
                let link = "<li><a href=\"/output".to_owned() + &d_path_split[0..j].join("/") + "/index.html\">" + &i + "</a></li>";
                navbar += &link;
                j+=1;
            }
            navbar += &("<li>".to_owned() + &page.meta.title + "</li>\n</ul></div>");
        }
        
        fs::write("output/".to_owned() + &directory_path + "/index.html", 
            template
            .replace("{content}", &markdown)
            .replace("{title}", &page.meta.title)
            .replace("{description}", &page.meta.description)
            .replace("{author}", &page.meta.author)
            .replace("{background}", &page.meta.background)
            .replace("{topnav}", &navbar))
            .expect("Unable to write file");
    }
}