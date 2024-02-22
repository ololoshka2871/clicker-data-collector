#![feature(proc_macro_span)]
#![feature(track_path)]

use std::io;
use std::path::PathBuf;

use swc::{
    config::{IsModule, SourceMapsConfig},
    Compiler, PrintArgs,
};

use swc_common::{
    errors::Handler, source_map::SourceMap, sync::Lrc, FilePathMapping, Mark, GLOBALS,
};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::Syntax;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

use proc_macro::Span;
use quote::quote;
use syn::{parse_macro_input, LitStr};

lazy_static::lazy_static! {
    static ref REPLACER: regex::Regex = regex::Regex::new(r"<.+\.ts>").unwrap();
}

// https://stackoverflow.com/a/76828821
/// Transforms typescript to javascript. Returns tuple (js string, source map)
fn ts_to_js(filename: &str, ts_code: &str) -> (String, String) {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let compiler = Compiler::new(cm.clone());

    let source = cm.new_source_file(
        swc_common::FileName::Custom(filename.into()),
        ts_code.to_string(),
    );

    let handler = Handler::with_emitter_writer(Box::new(io::stderr()), Some(compiler.cm.clone()));

    return GLOBALS.set(&Default::default(), || {
        let res = compiler
            .parse_js(
                source,
                &handler,
                EsVersion::Es5,
                Syntax::Typescript(Default::default()),
                IsModule::Bool(true),
                Some(compiler.comments()),
            )
            .expect("parse_js failed");

        let module = res.module().unwrap();

        // Add TypeScript type stripping transform
        let top_level_mark = Mark::new();
        let module = module.fold_with(&mut strip(top_level_mark));

        // https://rustdoc.swc.rs/swc/struct.Compiler.html#method.print
        let mut args = PrintArgs::default();
        args.inline_sources_content = true;
        args.source_map = SourceMapsConfig::Bool(true);
        args.comments = Some(compiler.comments());
        args.emit_source_map_columns = false;
        args.codegen_config.minify = false;
        
        let ret = compiler.print(&module, args).expect("print failed");

        return (ret.code, ret.map.expect("No map generated"));
    });
}

fn include_ts<R: regex::Replacer>(
    ts_file_name: PathBuf,
    dest_file_nname: R,
) -> proc_macro::TokenStream {
    if !ts_file_name.exists() {
        panic!(
            "file '{:?}' in '{:?}' not found",
            ts_file_name,
            std::env::current_dir().unwrap()
        );
    }

    let ts_file_name_str = ts_file_name.to_str().unwrap().to_owned();
    let ts_code = std::fs::read_to_string(ts_file_name).expect("Failed to read file");
    let (js_code, map) = ts_to_js(&ts_file_name_str, &ts_code);

    let map = REPLACER.replace_all(&map, dest_file_nname);

    quote! {
        (#js_code, #map, #ts_code)
    }
    .into()
}

#[proc_macro]
pub fn include_ts_relative(file: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let span = Span::call_site();
    let source = span.source_file();

    let infile = parse_macro_input!(file as LitStr).value();
    let ts_file_name = source
        .path()
        .parent()
        .expect("Invalid path")
        .join(PathBuf::from(&infile));

    let ts_file_name = ts_file_name.canonicalize().unwrap();

    // Следим за файлом, если он изменится, то перекомпилируемся
    proc_macro::tracked_path::path(ts_file_name.to_str().unwrap());

    let in_file_name_only = PathBuf::from(&infile)
        .file_name()
        .expect("Invalid path")
        .to_str()
        .unwrap()
        .to_owned();

    include_ts(ts_file_name, in_file_name_only)
}

#[proc_macro]
pub fn include_ts_proj(file: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ts_file_name = parse_macro_input!(file as LitStr).value();
    let ts_file_path = PathBuf::from(&ts_file_name).canonicalize().unwrap();

    // Следим за файлом, если он изменится, то перекомпилируемся
    proc_macro::tracked_path::path(ts_file_path.to_str().unwrap());

    let in_file_name_only = PathBuf::from(&ts_file_name)
        .file_name()
        .expect("Invalid path")
        .to_str()
        .unwrap()
        .to_owned();

    include_ts(ts_file_path, in_file_name_only)
}
