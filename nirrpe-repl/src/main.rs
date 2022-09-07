use nirrpe::parse::ast::Program;
use nirrpe::runtime::NirrpeRuntime;

fn main() {
    let mut runtime = NirrpeRuntime::default();
    match Program::parse(&mut runtime.rodeo, include_str!("zero.nirrpe")) {
        Ok(program) => {
            println!("{:#?}", runtime.rodeo);
            println!("{:#?}", program);
            println!();
            if let Err(err) = runtime.execute(program) {
                eprintln!("{:?}", err);
            }
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
