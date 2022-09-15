use nirrpe::parse::ast::Program;
use nirrpe::runtime::NirrpeRuntime;

fn main() {
    let mut runtime = NirrpeRuntime::new_with_std_extern_functions();
    let program = concat!(
        include_str!("../../nirrpe/src/std/std.nirrpe"),
        include_str!("zero.nirrpe")
    );
    match Program::parse(&mut runtime.rodeo, program) {
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
