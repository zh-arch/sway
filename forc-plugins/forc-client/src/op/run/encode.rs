use sway_core::{
    language::ty::{TyDecl, TyFunctionDecl, TyProgram},
    Engines,
};

struct ScriptCallHelper {
    main_fn: TyFunctionDecl,
}

const MAIN_FUNCTION_KEYWORD: &str = "main";

impl ScriptCallHelper {
    /// Construct a scrip call handler from provided typed program.
    ///
    /// Returns an error if the typed program is missing a main function, the function declaration
    /// for main is fetched from declaration engine.
    fn from_ty_program(ty_program: TyProgram, engines: Engines) -> anyhow::Result<Self> {
        let decl = ty_program
            .declarations
            .iter()
            .find_map(|decl| match decl {
                TyDecl::FunctionDecl(function_decl) => {
                    if function_decl.name.as_str() == MAIN_FUNCTION_KEYWORD {
                        Some(function_decl)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("missing main function"))?;

        let de = engines.de();
        let main_fn = de.get_function(&decl.decl_id);
        Ok(Self { main_fn })
    }
}
