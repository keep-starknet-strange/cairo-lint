use clap::Parser;

macro_rules! command {
    ($enum_name:ident ( $($module:ident,)+ )) => {
        $(mod $module;)+
        
        #[derive(::clap::Subcommand)]
        #[allow(non_camel_case_types)]
        enum $enum_name {
            $($module(crate::$module::Args),)+
        }

        impl $enum_name {
            fn main(self) {
                match self {
                    $(Self::$module(args) => crate::$module::main(args),)+
                }
            }
        }
    }
}

command!(Command(create_test,));

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    Args::parse().command.main();
}
