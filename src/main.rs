use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{Read, Write},
    process::{Command, Stdio},
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde::Deserialize;
use swc_common::BytePos;
use swc_ecma_parser::{lexer::Lexer, EsSyntax, Parser as ESParser, StringInput, Syntax};

mod command_utils;
mod globals;
mod transpiler;

#[cfg(test)]
mod test;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Path to clang++
    #[clap(long = "clang-path", default_value = "clang++", value_parser)]
    clang_path: String,

    /// Emit cpp code to stdout rather than compiling it
    #[clap(long = "emit-cpp", default_value_t = false, value_parser)]
    emit_cpp: bool,

    /// Target WebAssembly
    #[clap(long = "wasm", default_value_t = false, value_parser)]
    wasm: bool,

    /// Run as an internal tool
    #[clap(long = "itool", default_value_t = false, value_parser)]
    itool: bool,

    /// Output file
    #[clap(long = "output", short = 'o', value_parser)]
    out: Option<String>,

    /// Extra flags to path to clang++
    extra_flags: Vec<String>,
}

fn js_to_cpp<T: AsRef<str>>(mut transpiler: transpiler::Transpiler, input: T) -> Result<String> {
    let syntax = Syntax::Es(EsSyntax::default());
    let lexer = Lexer::new(
        syntax,
        swc_ecma_visit::swc_ecma_ast::EsVersion::latest(),
        StringInput::new(
            input.as_ref(),
            swc_common::BytePos(0),
            BytePos(input.as_ref().as_bytes().len().try_into().unwrap()),
        ),
        None,
    );
    let mut parser = ESParser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|err| anyhow!(format!("{:?}", err)))?;

    transpiler.globals.push(globals::io::io_global());
    transpiler.globals.push(globals::json::json_global());
    transpiler.globals.push(globals::symbol::symbol_global());
    transpiler.transpile_module(&module)
}

fn cpp_to_binary(
    code: String,
    outputname: String,
    clang_path: String,
    flags: &[String],
    // rtobjs: &[String],
) -> Result<()> {
    let cpp_file_name = format!("./{}.cpp", outputname);
    let mut tempfile = File::create(&cpp_file_name)?;
    tempfile.write_all(code.as_bytes())?;
    drop(tempfile);

    let args = flags
        .into_iter()
        .map(|i| i.as_ref())
        .chain(
            [
                "--std=c++20",
                "-o",
                outputname.as_ref(),
                cpp_file_name.as_ref(),
            ]
            .into_iter(),
        )
        .collect::<Vec<&str>>();

    let mut child = Command::new(&clang_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .args(args)
        .spawn()?;

    child.wait()?;
    std::fs::remove_file(cpp_file_name)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut input: String = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    if args.itool {
        let out = args.out.clone().context("in getting the runtime dir")?;
        #[derive(Deserialize)]
        struct Manifest {
            pub properties: BTreeSet<String>,
        }
        let j: Manifest = serde_json::from_str(&input)?;
        let mut base_props_inc = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(format!("{out}/js_primitives_props.hpp"))?;
        let mut base_props_access = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(format!("{out}/js_primitives_props_access.cpp"))?;
        write!(
            base_props_access,
            "static optional<JSValue> *prop(JSValue key,JSBase &out){{"
        )?;
        for p in j.properties {
            write!(base_props_inc, "optional<JSValue> $prop${p};")?;
            write!(
                base_props_access,
                " if(key.coerce_to_string() == \"{p}\")return &out.$prop${p};"
            )?;
        }
        write!(base_props_access, "return 0;}};")?;
    } else {
        let mut transpiler = transpiler::Transpiler::new();
        transpiler.feature_exceptions = !args.wasm;
        let cpp_code = js_to_cpp(transpiler, &input)?;

        if args.emit_cpp {
            let (_status, stdout, _stderr) = command_utils::pipe_through_shell::<String>(
                "clang-format",
                &[],
                cpp_code.as_bytes(),
            )?;
            let stdout = String::from_utf8(stdout)?;
            match args.out.as_deref() {
                None | Some("-") => {
                    println!("{stdout}");
                }
                Some(a) => {
                    std::fs::write(a, stdout)?;
                }
            }
        } else {
            let mut flags = args.extra_flags.clone();
            let mut extension = "".to_string();
            if args.wasm {
                flags.push("-fno-exceptions".to_string());
                flags.push("--target=wasm32-wasi".to_string());
                if let Ok(wasi_sdk_prefix) = std::env::var("WASI_SDK_PREFIX") {
                    flags.push(format!("--sysroot={}/share/wasi-sysroot", wasi_sdk_prefix));
                }
                extension = ".wasm".to_string();
            } else {
                flags.push("-DFEATURE_EXCEPTIONS".to_string());
            }
            cpp_to_binary(
                cpp_code,
                args.out
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| format!("output{}", extension)),
                args.clang_path,
                &flags,
            )?;
        }
    }
    Ok(())
}
