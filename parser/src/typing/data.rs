use std::collections::HashMap;
use std::collections::HashSet;
use crate::ast::*;

use automata::read_error::ReadError;

pub type EnvironmentMap = HashMap<String, Vec<EnvVariable>>;
pub type FuncSignature = (StaticType, Vec<StaticType>);
pub type InternalTypingResult<'a> = Result<(), ReadError<'a>>;
pub type PartialTypingResult<'a> = Result<StaticType, ReadError<'a>>;
pub type TypingResult<'a> = Result<TypedDecls<'a>, ReadError<'a>>;

pub fn is_compatible(a: StaticType, b: StaticType) -> bool {
    a == StaticType::Any || b == StaticType::Any || a == b
}

pub fn is_builtin_function(name: &String) -> bool {
    match name.as_str() {
        "println" | "div" | "print" => true,
        _ => false
    }
}

#[derive(Debug)]
pub struct TypedDecls<'a> {
    pub functions: HashMap<String, Vec<Function<'a>>>,
    pub structures: HashMap<String, Structure<'a>>,
    pub global_expressions: Vec<Exp<'a>>
}

#[derive(Debug)]
pub struct GlobalEnvironmentState<'a> {
    pub structures: HashMap<String, Structure<'a>>,
    pub functions: HashMap<String, Vec<Function<'a>>>,
    pub function_sigs: HashMap<String, Vec<FuncSignature>>,
    pub structure_name_by_fields: HashMap<String, String>,
    pub all_structure_fields: HashMap<String, StaticType>,
    pub all_mutable_fields: HashSet<String>,
    pub global_variables: HashSet<String>,
    pub global_expressions: Vec<Exp<'a>>,
    pub known_types: HashSet<StaticType>,
}

impl<'a> GlobalEnvironmentState<'a> {
    pub fn init() -> Self {
        GlobalEnvironmentState {
            structures: HashMap::new(),
            functions: HashMap::new(),
            function_sigs: HashMap::new(),
            structure_name_by_fields: HashMap::new(),
            all_structure_fields: HashMap::new(),
            all_mutable_fields: HashSet::new(),
            global_variables: HashSet::new(),
            global_expressions: vec![],
            known_types: vec![StaticType::Any, StaticType::Str, StaticType::Bool, StaticType::Int64, StaticType::Nothing].into_iter().collect()
        }
    }
}

#[derive(Debug)]
pub struct EnvVariable {
    ty: StaticType,
    scope: Scope
}

impl EnvVariable {
    pub fn init() -> Self {
        EnvVariable { ty: StaticType::Any, scope: Scope::Global }
    }

    pub fn local() -> Self {
        EnvVariable { scope: Scope::Local, ..EnvVariable::init() }
    }

    pub fn typed(ty: StaticType) -> Self {
        EnvVariable { ty, ..EnvVariable::init() }
    }
}

#[derive(Debug)]
pub struct TypingContext<'a> {
    pub functions: HashMap<String, Vec<FuncSignature>>,
    pub structures: HashMap<String, Structure<'a>>,
    pub known_types: HashSet<StaticType>,
    pub mutable_fields: HashSet<String>,
    pub all_fields: HashMap<String, StaticType>,
    pub structure_name_by_fields: HashMap<String, String>,
    pub previous_scope: Scope,
    pub current_scope: Scope,
    pub environment: EnvironmentMap
}

impl<'a> TypingContext<'a> {
    pub fn field_exist_in(&self, structure_type: &StaticType, field_name: &String) -> bool {
        match structure_type {
            StaticType::Any => true,
            StaticType::Nothing | StaticType::Int64 | StaticType::Str | StaticType::Bool  => false,
            StaticType::Struct(s) => self.structures[s].fields.iter().any(|p| &p.name.name == field_name)
        }
    }

    pub fn get_potentially_unique_return_type_for_function(&self, name: &String) -> Option<StaticType> {
        match self.functions.get(name) {
            None => None,
            Some(list_of_matches) => match list_of_matches.len() > 1 {
                true => None,
                false => Some(list_of_matches.first().unwrap().0.clone())
            }
        }
    }

    pub fn enter_in_local_scope(&mut self) {
        self.previous_scope = self.current_scope;
        self.current_scope = Scope::Local;
    }

    pub fn restore_previous_scope(&mut self) {
        self.current_scope = self.previous_scope;
    }

    pub fn push_to_env(&mut self, ident: &LocatedIdent<'a>, ty: StaticType, scope: Scope) {
        self.environment
            .entry(ident.name.clone())
            .or_default()
            .push(EnvVariable { ty, scope });
    }

    pub fn push_local_to_env(&mut self, ident: &LocatedIdent<'a>) {
        self.push_to_env(&ident, StaticType::Any, Scope::Local);
    }

    pub fn extend_local_env(&mut self, idents: Vec<LocatedIdent<'a>>) -> Vec<LocatedIdent<'a>> {
        idents.iter().for_each(|var| self.push_local_to_env(&var));
        idents
    }

    pub fn pop_from_env(&mut self, ident: &LocatedIdent<'a>) {
        match self.environment.get_mut(&ident.name) {
            None => {
                println!("warning: Attempt to pop from environment the variable: {}", &ident.name);
            },
            Some(types) => {
                types.pop();

                if types.len() == 0 {
                    self.environment.remove(&ident.name);
                }
            }
        }
    }

    pub fn unextend_env(&mut self, idents: Vec<LocatedIdent<'a>>) {
        idents.iter().for_each(|var| self.pop_from_env(&var));
    }

    pub fn type_from_env_name(&mut self, name: &String) -> Option<StaticType> {
        match self.environment.get(name) {
            None => None,
            Some(vars) => match vars.last() {
                None => None,
                Some(var) => Some(var.ty.clone())
            }
        }
    }

    pub fn scope_from_env_name(&mut self, name: &String) -> Option<Scope> {
        match self.environment.get(name) {
            None => None,
            Some(vars) => match vars.last() {
                None => None,
                Some(var) => Some(var.scope)
            }
        }
    }

    pub fn is_alive_in_env(&self, ident: &LocatedIdent<'a>) -> bool {
        self.environment.get(&ident.name).is_some()
    }
}
