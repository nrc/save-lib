#![feature(custom_derive, plugin)]
#![feature(question_mark)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;

use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::{Read, Write};

fn run(file_name: &str) {
    let mut file = File::open(file_name).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    let analysis: Analysis = match serde_json::from_str(&buf) {
        Ok(a) => a,
        Err(e) => {
            println!("Error: `{}`", e);
            return;
        }
    };

    let output: Output = analysis.into();
    let mut out_file = File::create(&format!("{}.out", file_name)).unwrap();
    out_file.write(serde_json::to_string(&output).unwrap().as_bytes()).unwrap();
}


fn main() {
    let args = env::args();
    for a in args {
        run(&a);
    }
}

#[derive(Deserialize, Debug)]
pub struct Analysis {
    pub prelude: Option<CratePreludeData>,
    pub defs: Vec<Def>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompilerId {
    pub krate: u32,
    pub index: u32,
}

#[derive(Deserialize, Debug)]
pub struct CratePreludeData {
    pub crate_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Def {
    pub kind: DefKind,
    pub id: CompilerId,
    pub span: SpanData,
    pub name: String,
    pub qualname: String,
    pub value: String,
}

#[derive(Serialize, Debug)]
pub enum DefKind {
    Enum,
    Tuple,
    Struct,
    Trait,
    Function,
    Macro,
    Mod,
    Type,
    Variable,
}

impl DefKind {
    // TODO prob needs to be on Def
    fn rustdoc_type_str(&self) -> &'static str {
        match *self {
            DefKind::Mod          => "mod",
            DefKind::Struct          => "struct",
            DefKind::Enum            => "enum",
            DefKind::Function        => "fn",
            DefKind::Type         => "type",
            DefKind::Trait           => "trait",
            DefKind::Macro           => "macro",

            // TODO Variable
            DefKind::Static          => "static",
            DefKind::Constant        => "constant",
            DefKind::StructField     => "structfield",

            // TODO method
            DefKind::TyMethod        => "tymethod",
            DefKind::Method          => "method",

            // TODO tuple/struct
            DefKind::Variant         => "variant",
        }
    }
}

// Custom impl to read rustc_serialize's format.
impl Deserialize for DefKind {
    fn deserialize<D>(deserializer: &mut D) -> Result<DefKind, D::Error>
        where D: serde::Deserializer,
    {
        let s = String::deserialize(deserializer)?;
        match &*s {
            "Enum" => Ok(DefKind::Enum),
            "Tuple" => Ok(DefKind::Tuple),
            "Struct" => Ok(DefKind::Struct),
            "Trait" => Ok(DefKind::Trait),
            "Function" => Ok(DefKind::Function),
            "Macro" => Ok(DefKind::Macro),
            "Mod" => Ok(DefKind::Mod),
            "Type" => Ok(DefKind::Type),
            "Variable" => Ok(DefKind::Variable),
            _ => Err(serde::de::Error::custom("unexpected def kind")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpanData {
    pub file_name: String,
    pub byte_start: u32,
    pub byte_end: u32,
    /// 1-based.
    pub line_start: usize,
    pub line_end: usize,
    /// 1-based, character offset.
    pub column_start: usize,
    pub column_end: usize,
}

#[derive(Serialize, Debug)]
struct Output {
    crate_name: String,
    defs: Vec<OutputDef>,
}

#[derive(Serialize, Debug)]
struct OutputDef {
    kind: DefKind,
    id: CompilerId,
    name: String,
    qualname: String,
    value: String,
    doc_url: String,
    src_url: String,
}

impl From<Analysis> for Output {
    fn from(analysis: Analysis) -> Output {
        let crate_name = analysis.prelude.map(|p| p.crate_name).unwrap_or("<unknown>".to_owned());
        Output {
            defs: analysis.defs.into_iter().map(|d| OutputDef::from_def(d, &crate_name)).collect(),
            crate_name: crate_name,
        }
    }
}

impl OutputDef {
    fn from_def(def: Def, crate_name: &str) -> OutputDef {
        let doc_url = make_doc_url(crate_name, &def);
        OutputDef {
            kind: def.kind,
            id: def.id,
            name: def.name,
            qualname: def.qualname,
            value: def.value,
            doc_url: doc_url,
            src_url: make_src_url(crate_name, &def.span),
        }
    }
}

const DOC_BASE: &'static str = "https://doc.rust-lang.org/nightly/";
const SRC_BASE: &'static str = "https://github.com/rust-lang/rust/tree/master/src/";

fn make_doc_url(crate_name: &str, def: &Def) -> String {
    // Synthesise a Rustdoc URL, see showResults in
    // https://github.com/rust-lang/rust/blob/master/src/librustdoc/html/static/main.js.
    let path = match def.kind {
        DefKind::Mod => {
            format!("{}/index.html", def.qualname.replace("::", "/"))
        }
        _ => {

        }
    };

    } else if (item.parent !== undefined) {
        var myparent = item.parent;
        var anchor = '#' + type + '.' + name;
        href = rootPath + item.path.replace(/::/g, '/') +
               '/' + itemTypes[myparent.ty] +
               '.' + myparent.name +
               '.html' + anchor;
    } else {
        href = rootPath + item.path.replace(/::/g, '/') +
               '/' + type + '.' + name + '.html';
    }

    "TODO".to_owned()
}

fn make_src_url(crate_name: &str, span: &SpanData) -> String {
    format!("{}lib{}/{}#L{}", SRC_BASE, crate_name, span.file_name, span.line_start)
}
