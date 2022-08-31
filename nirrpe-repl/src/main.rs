use nirrpe::parse::ast::Program;

fn main() {
    match Program::parse("print()") {
        Ok((rodeo, program)) => {
            println!("{:#?}", rodeo);
            println!("{:#?}", program);
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}