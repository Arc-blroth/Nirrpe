//! # 🗺️ Nirrpe

#![feature(type_alias_impl_trait)]
#![feature(int_roundings)]
#![feature(decl_macro)]
#![feature(adt_const_params)]
#![allow(incomplete_features)]
#![allow(clippy::type_complexity)]

use std::fmt::{Debug, Display};
use std::{env, fs};

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::error::Rich;
use chumsky::input::Input;
use chumsky::prelude::SimpleSpan;
use chumsky::Parser;

use crate::runtime::NirrpeRuntime;

pub mod parse;
pub mod runtime;

fn main() {
    let filename = env::args().nth(1).unwrap();
    let src = fs::read_to_string(filename.clone()).unwrap();

    let (tokens, errs) = parse::lexer::lexer().parse(&src).into_output_errors();

    if !errs.is_empty() {
        print_error_report(errs, &filename, &src);
        if let Some(tokens) = tokens {
            println!("LexResult {{ output: {:?} }}", tokens);
        }
    } else if let Some(tokens) = tokens {
        println!("LexResult {{ output: {:?} }}", tokens);

        let (program, errs) = parse::parser()
            .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
            .into_output_errors();

        if !errs.is_empty() {
            print_error_report(errs, &filename, &src);
        } else if let Some(program) = program {
            println!("ParseResult {{ {:?} }}", program);
            NirrpeRuntime::new().execute(program);
        }
    }
}

#[allow(clippy::ptr_arg)]
fn print_error_report<T, L>(errs: Vec<Rich<T, SimpleSpan, L>>, filename: &String, src: &String)
where
    T: Debug + Clone,
    L: Display + ToString + Clone,
{
    errs.into_iter()
        .map(|e| e.map_token(|c| format!("{:?}", c)))
        .for_each(|e| {
            Report::build(ReportKind::Error, filename.clone(), e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new((filename.clone(), e.span().into_range()))
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .with_labels(e.contexts().map(|(label, span)| {
                    Label::new((filename.clone(), span.into_range()))
                        .with_message(label.clone())
                        .with_color(Color::Yellow)
                }))
                .finish()
                .print(ariadne::sources([(filename.clone(), src.clone())]))
                .unwrap()
        });
}
