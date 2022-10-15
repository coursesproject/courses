use crate::builder::Builder;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub title: String,
    pub version: String,
    pub build_path: PathBuf,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    title: String,
    id: String,
    doc: Document,
    sections: Vec<Section>,
    resources: Vec<ResourceFile>,
    code: Vec<CodeFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    id: String,
    doc: Document,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Format {
    Markdown,
    Notebook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub format: Format,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    Python,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFile {
    path: PathBuf,
    language: Language,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFile {
    src_path: PathBuf,
    static_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    title: String,
    version: String,
    build_path: String,
}

impl Format {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        match path.as_ref().extension().unwrap().to_str().unwrap() {
            "md" => Ok(Format::Markdown),
            "ipynb" => Ok(Format::Notebook),
            _ => Err(anyhow!("Invalid file extension")),
        }
    }
}

impl Document {
    fn new<P: AsRef<Path>>(section_path: P) -> Result<Self> {
        Ok(Document {
            path: section_path.as_ref().to_path_buf(),
            format: Format::from_path(section_path)?,
        })
    }
}

fn raw_file_name<P: AsRef<Path>>(path: P) -> Option<String> {
    Some(
        path.as_ref()
            .file_name()?
            .to_str()?
            .split(".")
            .into_iter()
            .next()?
            .to_string(),
    )
}

const EXT: [&str; 2] = ["md", "ipynb"];

fn extension_in(extension: &str) -> bool {
    EXT.iter().any(|e| e == &extension)
}

impl Section {
    fn new<P: AsRef<Path>>(section_path: P) -> Result<Self> {
        Ok(Section {
            id: raw_file_name(section_path.as_ref()).unwrap(),
            doc: Document::new(section_path)?,
        })
    }
}

impl Chapter {
    fn new<P: AsRef<Path>>(chapter_dir: P) -> Result<Self> {
        let section_dir = chapter_dir.as_ref().join("sections");

        let sections = match fs::read_dir(section_dir) {
            Ok(dir) => {
                let paths = dir
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry
                            .path()
                            .extension()
                            .filter(|e| extension_in(e.to_str().unwrap()))
                            .is_some()
                    })
                    .filter(|entry| entry.metadata().map(|meta| meta.is_file()).is_ok());

                paths
                    .map(|entry| Section::new(entry.path()))
                    .collect::<Result<Vec<Section>>>()?
            }
            Err(_) => Vec::new(),
        };

        let chapter_index = chapter_dir.as_ref().join("index.md");

        Ok(Chapter {
            title: "hej".to_string(),
            id: chapter_dir
                .as_ref()
                .file_name()
                .ok_or(anyhow::Error::msg("Invalid file name"))?
                .to_str()
                .unwrap()
                .to_string(),
            doc: Document::new(chapter_index)?,

            sections,
            resources: vec![],
            code: vec![],
        })
    }
}

impl Config {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
        let cfg_path = path.as_ref().join("config.yml");
        let cfg: ConfigFile = serde_yaml::from_reader(BufReader::new(File::open(cfg_path)?))?;

        let content_path = path.as_ref().join("content");
        let chapters = fs::read_dir(content_path)?
            .filter_map(|entry| {
                entry
                    .map(|entry| {
                        let m = fs::metadata(entry.path());
                        m.map(|m| m.is_dir().then_some(entry)).ok()?
                    })
                    .ok()?
            })
            .map(|entry| {
                let file_path = entry.path();
                Chapter::new(file_path)
            })
            .collect::<Result<Vec<Chapter>>>()?;

        Ok(Config {
            title: cfg.title,
            version: cfg.version,
            build_path: path.as_ref().join(cfg.build_path),
            chapters: chapters,
        })
    }

    pub fn build(&self, builder: &mut Builder) -> Result<()> {
        fs::create_dir_all(self.build_path.as_path())?;

        for chapter in &self.chapters {
            println!("Building chapter {}", chapter.id);
            let chapter_build_path = self.build_path.as_path().join(chapter.id.clone());
            fs::create_dir_all(chapter_build_path.as_path())?;
            for section in &chapter.sections {
                let section_build_path = chapter_build_path.join(format!("{}.html", section.id));
                let section_meta_path =
                    chapter_build_path.join(format!("{}_meta.json", section.id));
                let section_solution_path =
                    chapter_build_path.join(format!("{}_solution.py", section.id));
                let content = builder.parse_pd(section.doc.clone())?;
                // let content = parse(section.doc.clone())?;
                let result = builder.render_section(&self, content.html)?;
                fs::write(section_build_path, result)?;

                let f = File::create(section_meta_path)?;
                let writer = BufWriter::new(f);
                serde_json::to_writer(writer, &content.split_meta)?;

                fs::write(section_solution_path, content.raw_solution)?;
            }
        }

        Ok(())
    }
}
