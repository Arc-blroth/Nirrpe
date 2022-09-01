use nirrpe::parse::ast::Program;

fn main() {
    match Program::parse(r#"print("hello\sworld!")"#) {
        Ok((rodeo, program)) => {
            println!("{:#?}", rodeo);
            println!("{:#?}", program);
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
