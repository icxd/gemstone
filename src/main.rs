use std::collections::HashMap;

use colored::Colorize;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    LeftParen,
    RightParen,
    LeftArrow, // ->
    LeftCurly,
    RightCurly,
    String(String),
    Semicolon,
    Colon,
    Comma,
    Int(i32),
    Float(f32),
    Equal,
    Plus,
    Minus,
    Star,
    Slash,
    Dot,
}

#[derive(Debug, Clone, PartialEq)]
enum Type {
    Int,
    Float,
    String,
    Bool,
    Void,
    Char,
    Class(String),
    Pointer(Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
enum AccessModifier {
    Public,
    Private,
}

#[derive(Debug, Clone)]
struct Class {
    pub name: String,
    pub base_class: Option<String>,
    pub methods: Vec<Expr>,
}

#[derive(Debug, Clone)]
struct ClassFunction {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Box<Expr>,
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_external: bool,
    pub access: AccessModifier,
}

#[derive(Debug, Clone)]
struct Function {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
struct Block {
    pub exprs: Vec<Expr>,
}

#[derive(Debug, Clone)]
struct FunctionCall {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
struct VariableDeclaration {
    pub name: String,
    pub value: Box<Expr>,
    pub var_type: Type,
    pub constant: bool,
}

#[derive(Debug, Clone)]
struct New {
    pub class_name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
enum Expr {
    Class(Class),
    ClassFunction(ClassFunction),
    Function(Function),
    Block(Block),
    FunctionCall(FunctionCall),
    InternalFunctionCall(FunctionCall),
    Int(i32),
    String(String),
    Variable(String),
    Return(Box<Expr>),
    VariableDeclaration(VariableDeclaration),
    New(New),
    BinaryOp(Box<Expr>, Token, Box<Expr>),
    Member(Box<Expr>, String),
    MemberFunctionCall(Box<Expr>, FunctionCall),
    Empty,
}

#[derive(Debug, Clone)]
struct Gemstone {
    internal_functions: Vec<String>,
    classes: HashMap<String, Class>,
    variables: HashMap<String, VariableDeclaration>,
}

impl Gemstone {
    pub fn new() -> Gemstone {
        Gemstone {
            internal_functions: vec![
                "print".to_string(),
                "println".to_string(),
            ],
            classes: HashMap::new(),
            variables: HashMap::new(),
        }
    }
    pub fn lex(&mut self, contents: &String) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];
        let mut index: usize = 0;

        while index < contents.len() {
            let token: Token = match contents.chars().nth(index).unwrap().clone() {
                ' ' | '\n' | '\t' | '\r' => { index += 1; continue; }
                '(' => { index += 1; Token::LeftParen }
                ')' => { index += 1; Token::RightParen }
                '-' => {
                    index += 1;
                    if contents.chars().nth(index).unwrap().clone() == '>' {
                        index += 1;
                        Token::LeftArrow
                    } else {
                        Token::Minus
                    }
                }
                '{' => { index += 1; Token::LeftCurly }
                '}' => { index += 1; Token::RightCurly }
                ';' => { index += 1; Token::Semicolon }
                ':' => { index += 1; Token::Colon }
                ',' => { index += 1; Token::Comma }
                '=' => { index += 1; Token::Equal }
                '+' => { index += 1; Token::Plus }
                '*' => { index += 1; Token::Star }
                '/' => { index += 1; Token::Slash }
                '.' => { index += 1; Token::Dot }
                'a'..='z' | 'A'..='Z' => {
                    let mut word: String = String::new();
                    while index < contents.len() && contents.chars().nth(index).unwrap().clone().is_alphanumeric() || contents.chars().nth(index).unwrap().clone() == '_' {
                        word += contents.chars().nth(index).unwrap().clone().to_string().as_str();
                        index += 1;
                    }
                    Token::Word(word)
                }
                '0'..='9' => {
                    let mut number: String = String::new();
                    while index < contents.len() && contents.chars().nth(index).unwrap().clone().is_numeric() {
                        number += contents.chars().nth(index).unwrap().clone().to_string().as_str();
                        index += 1;
                    }
                    if index < contents.len() && contents.chars().nth(index).unwrap().clone() == '.' {
                        number += contents.chars().nth(index).unwrap().clone().to_string().as_str();
                        index += 1;
                        while index < contents.len() && contents.chars().nth(index).unwrap().clone().is_numeric() {
                            number += contents.chars().nth(index).unwrap().clone().to_string().as_str();
                            index += 1;
                        }
                        Token::Float(number.parse::<f32>().unwrap())
                    } else {
                        Token::Int(number.parse::<i32>().unwrap())
                    }
                }
                '"' => {
                    let mut string: String = String::new();
                    index += 1;
                    while index < contents.len() && contents.chars().nth(index).unwrap().clone() != '"' {
                        string += contents.chars().nth(index).unwrap().clone().to_string().as_str();
                        index += 1;
                    }
                    index += 1;
                    Token::String(string)
                }
                _ => panic!("invalid character: '{}'", contents.chars().nth(index).unwrap().clone())
            };
            tokens.push(token);
        }

        tokens
    }
    pub fn parse(&mut self, tokens: &Vec<Token>) -> Vec<Expr> {
        let mut exprs: Vec<Expr> = vec![];
        let mut index: usize = 0;

        while index < tokens.len() {
            exprs.push(self.parse_token(tokens, &mut index));
        }

        exprs
    }
    fn parse_token(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        match tokens.get(*index).unwrap().clone() {
            Token::Word(word) => {
                match word.as_str() {
                    "class" => self.parse_class_def(tokens, index),
                    "function" => self.parse_function(tokens, index),
                    "return" => self.parse_return(tokens, index),
                    "var" => self.parse_variable(tokens, index, false),
                    "const" => self.parse_variable(tokens, index, true),
                    "new" => self.parse_new(tokens, index),
                    _ => {
                        if *index < tokens.len() - 1 && tokens.get(*index + 1).unwrap().clone() == Token::LeftParen {
                            self.parse_function_call(tokens, index)
                        } else {
                            self.parse_additive(tokens, index)
                        }
                    }
                }
            }
            Token::LeftCurly => self.parse_block(tokens, index),
            _ => self.parse_additive(tokens, index)
        }
    }
    fn parse_class_def(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        *index += 1;
        let name = match tokens.get(*index).unwrap().clone() {
            Token::Word(word) => word,
            _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
        };
        *index += 1;
        let mut base_class: Option<String> = None;
        if tokens.get(*index).unwrap().clone() == Token::Colon {
            *index += 1;
            base_class = Some(match tokens.get(*index).unwrap().clone() {
                Token::Word(word) => word,
                _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
            });
            *index += 1;
        }
        let mut methods: Vec<Expr> = vec![];
        *index += 1;
        while *index < tokens.len() && tokens.get(*index).unwrap().clone() != Token::RightCurly {
            methods.push(self.parse_class_function_def(tokens, index));
        }
        *index += 1;
        self.classes.insert(name.clone(), Class { name: name.clone(), base_class, methods });
        Expr::Class(self.classes.get(&name.clone()).unwrap().clone())
    }
    fn parse_class_function_def(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        let mut access_modifier: AccessModifier = AccessModifier::Private;
        if tokens.get(*index).unwrap().clone() == Token::Word("public".to_string()) {
            access_modifier = AccessModifier::Public;
            *index += 1;
        } else if tokens.get(*index).unwrap().clone() == Token::Word("private".to_string()) {
            access_modifier = AccessModifier::Private;
            *index += 1;
        }
        let mut is_virtual: bool = false;
        let mut is_override: bool = false;
        let mut is_external: bool = false;
        if tokens.get(*index).unwrap().clone() == Token::Word("virtual".to_string()) {
            is_virtual = true;
            *index += 1;
        } else if tokens.get(*index).unwrap().clone() == Token::Word("override".to_string()) {
            is_override = true;
            *index += 1;
        } else if tokens.get(*index).unwrap().clone() == Token::Word("external".to_string()) {
            is_external = true;
            *index += 1;
        }
        if is_virtual && is_override {
            panic!("cannot be both virtual and override");
        } else if is_virtual && is_external {
            panic!("cannot be both virtual and external");
        } else if is_override && is_external {
            panic!("cannot be both override and external");
        }
        let function: Expr = self.parse_function(tokens, index);
        if is_external {
            match function.clone() {
                Expr::Function(function) => {
                    match *function.body {
                        Expr::Block(_) => panic!("external function cannot have a body"),
                        Expr::Empty => {}
                        _ => panic!("expected block, got {:?}", function.body)
                    }
                }
                _ => panic!("expected function, got {:?}", function)
            }
        }
        match function {
            Expr::Function(function) => Expr::ClassFunction(ClassFunction {
                name: function.name,
                args: function.args,
                return_type: function.return_type,
                body: function.body,
                is_virtual,
                is_override,
                is_external,
                access: access_modifier,
            }),
            _ => panic!("expected function, got {:?}", function)
        }
    }
    fn parse_function(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        if tokens.get(*index).unwrap().clone() == Token::Word("function".to_string()) {
            *index += 1;
        } else {
            panic!("expected 'function', got {:?}", tokens.get(*index).unwrap().clone());
        }
        let name: String = match tokens.get(*index).unwrap().clone() {
            Token::Word(word) => word,
            _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
        };
        *index += 1;
        self._match(tokens, index, &Token::LeftParen);
        let mut args: Vec<(String, Type)> = vec![];
        while *index < tokens.len() && tokens.get(*index).unwrap().clone() != Token::RightParen {
            let arg_name: String = match tokens.get(*index).unwrap().clone() {
                Token::Word(word) => word,
                _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
            };
            *index += 1; // skip to :
            self._match(tokens, index, &Token::Colon);
            let arg_type: Type = self.parse_type(tokens, index);
            args.push((arg_name, arg_type));
            if *index < tokens.len() && tokens.get(*index).unwrap().clone() == Token::Comma {
                self._match(tokens, index, &Token::Comma);
            }
        }
        self._match(tokens, index, &Token::RightParen);
        self._match(tokens, index, &Token::LeftArrow);
        let return_type: Type = self.parse_type(tokens, index);
        if tokens.get(*index).unwrap().clone() == Token::Semicolon {
            self._match(tokens, index, &Token::Semicolon);
            return Expr::Function(Function { 
                name,
                args,
                return_type,
                body: Box::new(Expr::Empty),
            });
        }
        self._match(tokens, index, &Token::LeftCurly);
        let body: Expr = self.parse_block(tokens, index);
        self._match(tokens, index, &Token::RightCurly);
        Expr::Function(Function { 
            name,
            args,
            return_type,
            body: Box::new(body),
        })
    }
    fn parse_block(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        let mut exprs: Vec<Expr> = vec![];
        while *index < tokens.len() && tokens.get(*index).unwrap().clone() != Token::RightCurly {
            exprs.push(self.parse_token(tokens, index));
        }
        Expr::Block(Block { exprs })
    }
    fn parse_function_call(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        let name: String = match tokens.get(*index).unwrap().clone() {
            Token::Word(word) => word,
            _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
        };
        *index += 1;
        self._match(tokens, index, &Token::LeftParen);
        let mut args: Vec<Expr> = vec![];
        while *index < tokens.len() && tokens.get(*index).unwrap().clone() != Token::RightParen {
            args.push(self.parse_token(tokens, index));
            if *index < tokens.len() && tokens.get(*index).unwrap().clone() == Token::Comma {
                self._match(tokens, index, &Token::Comma);
            }
        }
        self._match(tokens, index, &Token::RightParen);
        self._match(tokens, index, &Token::Semicolon);
        if self.internal_functions.contains(&name) {
            Expr::InternalFunctionCall(FunctionCall { name, args })
        } else {
            Expr::FunctionCall(FunctionCall { name, args })
        }
    }
    fn parse_return(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        *index += 1;
        let value: Expr = self.parse_token(tokens, index);
        self._match(tokens, index, &Token::Semicolon);
        Expr::Return(Box::new(value))
    }
    fn parse_variable(&mut self, tokens: &Vec<Token>, index: &mut usize, constant: bool) -> Expr {
        *index += 1; // skip to name
        let name: String = match tokens.get(*index).unwrap().clone() {
            Token::Word(word) => word,
            _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
        };
        *index += 1;
        self._match(tokens, index, &Token::Colon);
        let var_type: Type = self.parse_type(tokens, index);
        self._match(tokens, index, &Token::Equal);
        let value: Expr = self.parse_token(tokens, index);
        self._match(tokens, index, &Token::Semicolon);
        self.variables.insert(name.clone(), VariableDeclaration { name: name.clone(), value: Box::new(value), var_type, constant });
        Expr::VariableDeclaration(self.variables.get(&name).unwrap().clone())
    }
    fn parse_new(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        *index += 1; // skip new
        let mut class_name: String = match tokens.get(*index).unwrap().clone() {
            Token::Word(word) => word,
            _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
        };
        *index += 1; // skip name
        *index += 1; // skip (
        *index += 1; // skip )
        let mut args: Vec<Expr> = vec![];
        Expr::New(New { class_name, args })
    }
    fn parse_type(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Type {
        let mut type_: Type = match tokens.get(*index).unwrap().clone() {
            Token::Word(word) => match word.as_str() {
                "int" => Type::Int,
                "float" => Type::Float,
                "void" => Type::Void,
                "bool" => Type::Bool,
                "string" => Type::String,
                "char" => Type::Char,
                _ => Type::Class(word)
            },
            _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
        };
        *index += 1;
        while *index < tokens.len() && tokens.get(*index).unwrap().clone() == Token::Star {
            type_ = Type::Pointer(Box::new(type_));
            self._match(tokens, index, &Token::Star);
        }
        type_
    }
    fn parse_additive(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        let mut expr: Expr = self.parse_multiplicative(tokens, index);
        while *index < tokens.len() && (tokens.get(*index).unwrap().clone() == Token::Plus || tokens.get(*index).unwrap().clone() == Token::Minus) {
            let op: Token = tokens.get(*index).unwrap().clone();
            *index += 1;
            let mut right: Expr = self.parse_multiplicative(tokens, index);
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        expr
    }
    fn parse_multiplicative(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        let mut expr: Expr = self.parse_member(tokens, index);
        while *index < tokens.len() && (tokens.get(*index).unwrap().clone() == Token::Star || tokens.get(*index).unwrap().clone() == Token::Slash) {
            let op: Token = tokens.get(*index).unwrap().clone();
            *index += 1;
            let mut right: Expr = self.parse_member(tokens, index);
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        expr
    }
    fn parse_member(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        let mut expr: Expr = self.parse_member_function_call(tokens, index);
        while *index < tokens.len() && tokens.get(*index).unwrap().clone() == Token::Dot {
            *index += 1;
            let mut name: String = match tokens.get(*index).unwrap().clone() {
                Token::Word(word) => word,
                _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
            };
            *index += 1;
            expr = Expr::Member(Box::new(expr), name);
        }
        expr
    }
    fn parse_member_function_call(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        let mut expr: Expr = self.parse_primary(tokens, index);
        while *index < tokens.len() && tokens.get(*index).unwrap().clone() == Token::LeftArrow {
            *index += 1;
            let mut name: String = match tokens.get(*index).unwrap().clone() {
                Token::Word(word) => word,
                _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
            };
            *index += 1;
            let mut args: Vec<Expr> = vec![];
            self._match(tokens, index, &Token::LeftParen);
            while *index < tokens.len() && tokens.get(*index).unwrap().clone() != Token::RightParen {
                args.push(self.parse_token(tokens, index));
                if *index < tokens.len() && tokens.get(*index).unwrap().clone() == Token::Comma {
                    *index += 1;
                }
            }
            self._match(tokens, index, &Token::RightParen);
            expr = Expr::MemberFunctionCall(Box::new(expr), FunctionCall { name, args });
        }
        expr
    }
    fn parse_primary(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        match tokens.get(*index).unwrap().clone() {
            Token::Word(word) => {
                if word == "new" {
                    self.parse_new(tokens, index)
                } else {
                    self.parse_var(tokens, index)
                }
            },
            Token::Int(number) => {
                *index += 1;
                Expr::Int(number)
            },
            Token::String(string) => {
                *index += 1;
                Expr::String(string)
            },
            Token::LeftParen => {
                *index += 1;
                let mut expr: Expr = self.parse_token(tokens, index);
                *index += 1;
                expr
            },
            Token::LeftCurly => {
                self.parse_block(tokens, index)
            },
            Token::Semicolon => {
                *index += 1;
                Expr::Empty
            },
            _ => panic!("expected primary, got {:?}",
                tokens.get(*index).unwrap().clone(),
            )
        }
    }
    fn parse_var(&mut self, tokens: &Vec<Token>, index: &mut usize) -> Expr {
        let mut name: String = match tokens.get(*index).unwrap().clone() {
            Token::Word(word) => word,
            _ => panic!("expected Word, got {:?}", tokens.get(*index).unwrap().clone())
        };
        
        *index += 1; // skip to ;
        Expr::Variable(name)
    }
    fn _match(&mut self, tokens: &Vec<Token>, index: &mut usize, token: &Token) -> Token {
        if tokens.get(*index).unwrap().clone() == *token {
            *index += 1;
            return token.clone();
        }
        panic!("expected {:?}, got {:?} (before: {:?}, after: {:?})", token.clone(), tokens.get(*index).unwrap().clone(), tokens.get(*index - 1).unwrap().clone(), tokens.get(*index + 1).unwrap().clone());
    }
    pub fn type_checker(&mut self, exprs: &Vec<Expr>) {
        for expr in exprs {
            self.type_check_expr(expr);
        }
    }
    fn type_check_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Function(function) => self.type_check_function(function),
            Expr::Block(block) => self.type_checker(&block.clone().exprs),
            Expr::VariableDeclaration(variable_declaration) => self.type_check_variable_declaration(variable_declaration),
            _ => {}
        }
    }
    fn type_check_function(&mut self, function: &Function) {
        let body: Expr = *function.clone().body.clone();
        self.type_check_expr(&body);
    }
    fn type_check_variable_declaration(&mut self, variable_declaration: &VariableDeclaration) {
        let var_type: Type = variable_declaration.clone().var_type.clone();
        let value: Expr = *variable_declaration.clone().value.clone();
        let value_type: Type = self.type_checker_get_type(&value);
        self.type_checker_rename_me(&var_type, &value_type);
    }
    fn type_checker_rename_me(&mut self, first: &Type, second: &Type) {
        match first {
            Type::Pointer(first) => {
                match second {
                    Type::Pointer(second) => {
                        self.type_checker_rename_me(&*first, &*second);
                    },
                    _ => panic!("type_checker_rename_me: mismatched types {:?} {:?}", first, second)
                }
            },
            Type::Class(first) => {
                match second {
                    Type::Class(second) => {
                        let first_class: Class = self.classes.get(first).unwrap().clone();
                        let second_class: Class = self.classes.get(second).unwrap().clone();
                        let first_base_class: Option<String> = first_class.clone().base_class.clone();
                        let second_base_class: Option<String> = second_class.clone().base_class.clone();
                        if second_base_class.is_some() {
                            self.type_checker_rename_me(&Type::Class(first.clone()), &Type::Class(second_base_class.unwrap()));
                        }
                        if first_base_class.is_some() {
                            self.type_checker_rename_me(&Type::Class(first_base_class.unwrap()), &Type::Class(second.clone()));
                        }
                    },
                    _ => panic!("type_checker_rename_me: mismatched types {:?} {:?}", first, second)
                }
            },
            _ => panic!("type_checker_rename_me: unimplemented {:?} {:?}", first, second)
        }
    }
    fn type_checker_get_type(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Int(_) => Type::Int,
            Expr::String(_) => Type::String,
            Expr::New(new) => {
                let class_name: String = new.clone().class_name.clone();
                Type::Pointer(Box::new(Type::Class(class_name)))
            },
            Expr::MemberFunctionCall(parent, function_call) => {
                todo!("type_checker_get_type: MemberFunctionCall")
            },
            _ => panic!("type_checker_get_type: unimplemented {:?}", expr)
        }
    }
    pub fn compile(&mut self, exprs: &Vec<Expr>) -> String {
        let mut output: String = String::new();
        output.push_str("#include <stdio.h>\n");
        output.push_str("#include <string>\n");
        for expr in exprs {
            output.push_str(&self.compile_expr(expr));
        }
        output
    }
    fn compile_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Class(class) => self.compile_class(class),
            Expr::InternalFunctionCall(function_call) => self.compile_internal_function_call(function_call),
            Expr::String(string) => self.compile_string(string),
            Expr::Int(int) => self.compile_int(int),
            Expr::Function(function) => self.compile_function(function),
            Expr::Return(return_expr) => self.compile_return(return_expr),
            Expr::VariableDeclaration(variable_declaration) => self.compile_variable_declaration(variable_declaration),
            Expr::New(new) => self.compile_new(new),
            Expr::Variable(name) => name.clone().to_string(),
            Expr::FunctionCall(function_call) => self.compile_function_call(function_call),
            Expr::BinaryOp(left, op, right) => self.compile_binary_op(left, op, right),
            Expr::MemberFunctionCall(left, right) => self.compile_member_function_call(left, right),
            Expr::Member(left, name) => self.compile_member(left, name),
            Expr::Empty => String::new(),
            _ => panic!("invalid or unimplemented expr: {:?}", expr)
        }
    }
    fn compile_class(&mut self, class: &Class) -> String {
        let mut output: String = String::new();

        let mut public_methods: Vec<&ClassFunction> = vec![];
        let mut private_methods: Vec<&ClassFunction> = vec![];
        for method in &class.methods {
            match method {
                Expr::ClassFunction(class_function) => {
                    match class_function.access {
                        AccessModifier::Public => public_methods.push(class_function),
                        AccessModifier::Private => private_methods.push(class_function),
                    }
                }
                _ => panic!("expected ClassFunction, got {:?}", method)
            }
        }

        output.push_str(&format!("class {}", class.name));
        if class.base_class.is_some() {
            output.push_str(&format!(": public {}", class.base_class.clone().unwrap()));
        }
        output.push_str(" {\n");
        if public_methods.len() > 0 {
            output.push_str("public:\n");
        }
        for method in public_methods {
            output.push_str(&self.compile_class_function(method));
        }
        if private_methods.len() > 0 {
            output.push_str("private:\n");
        }
        for method in private_methods {
            output.push_str(&self.compile_class_function(method));
        }
        output.push_str("};\n");
        output
    }
    fn compile_class_function(&mut self, class_function: &ClassFunction) -> String {
        let mut output: String = String::new();

        if class_function.is_virtual {
            output.push_str("virtual ");
        }

        output.push_str(&format!("{} {}(", self.compile_type(&class_function.return_type), class_function.name));

        let mut args: Vec<String> = vec![];
        for arg in &class_function.args {
            args.push(format!("{} {}", self.compile_type(&arg.1), arg.0));
        }
        let args: String = args.join(", ");

        output.push_str(&args);
        output.push_str(") ");
        if class_function.is_override {
            output.push_str("override ");
        }
        match &*class_function.body {
            Expr::Block(block) => output.push_str(&self.compile_block(block)),
            Expr::Empty => {
                if class_function.is_override {
                    panic!("override function must have a body");
                }
                output.push_str("= 0;\n");
                return output;
            },
            _ => panic!("expected Block or Empty, got {:?}", class_function.body)
        }

        output
    }
    fn compile_function(&mut self, function: &Function) -> String {
        let mut output: String = String::new();
        output.push_str(&format!("{} {}(", self.compile_type(&function.return_type), function.name));

        let mut args: Vec<String> = vec![];
        for arg in &function.args {
            args.push(format!("{} {}", self.compile_type(&arg.1), arg.0));
        }
        let args: String = args.join(", ");

        output.push_str(&args);
        output.push_str(") ");
        output.push_str(&self.compile_block(match &*function.body {
            Expr::Block(block) => block,
            _ => panic!("expected Block, got {:?}", function.body)
        }));

        output
    }
    fn compile_return(&mut self, return_expr: &Box<Expr>) -> String {
        let mut output: String = String::new();
        output.push_str("return ");
        output.push_str(&self.compile_expr(&*return_expr));
        output.push_str(";\n");
        output
    }
    fn compile_variable_declaration(&mut self, variable_declaration: &VariableDeclaration) -> String {
        let mut output: String = String::new();
        output.push_str(&format!("{} ", self.compile_type(&variable_declaration.var_type)));
        if variable_declaration.constant {
            output.push_str("const ")
        }
        output.push_str(&format!("{} = ", variable_declaration.name));
        output.push_str(&self.compile_expr(&variable_declaration.value));
        output.push_str(";\n");
        output
    }
    fn compile_new(&mut self, new: &New) -> String {
        let mut output: String = String::new();
        output.push_str(&format!("new {}(", new.class_name));
        let mut args: Vec<String> = vec![];
        for arg in &new.args {
            args.push(self.compile_expr(arg));
        }
        let args: String = args.join(", ");
        output.push_str(&args);
        output.push_str(")");
        output
    }
    fn compile_type(&mut self, type_: &Type) -> String {
        match type_ {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::String => "std::string".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
            Type::Char => "char".to_string(),
            Type::Class(class) => class.clone(),
            Type::Pointer(pointer) => format!("{}*", self.compile_type(pointer)),
        }
    }
    fn compile_block(&mut self, block: &Block) -> String {
        let mut output: String = String::new();
        output.push_str("{\n");
        for expr in &block.exprs {
            output.push_str(&self.compile_expr(expr));
        }
        output.push_str("}\n");
        output
    }
    fn compile_internal_function_call(&mut self, function_call: &FunctionCall) -> String {
        let mut output: String = String::new();
        match function_call.name.as_str() {
            "print" => {
                output.push_str(&format!("printf({});\n", self.compile_expr(&function_call.args[0])));
            }
            "println" => {
                output.push_str(&format!("printf({}+\"\\n\");\n", self.compile_expr(&function_call.args[0])));
            }
            _ => panic!("invalid or unimplemented internal function: {}", function_call.name)
        }
        output
    }
    fn compile_function_call(&mut self, function_call: &FunctionCall) -> String {
        let mut output: String = String::new();
        output.push_str(&format!("{}(", function_call.name));
        let mut args: Vec<String> = vec![];
        for arg in &function_call.args {
            args.push(self.compile_expr(arg));
        }
        let args: String = args.join(", ");
        output.push_str(&args);
        output.push_str(")");
        output
    }
    fn compile_string(&mut self, string: &String) -> String {
        format!("\"{}\"", string)
    }
    fn compile_int(&mut self, int: &i32) -> String {
        format!("{}", int)
    }
    fn compile_binary_op(&mut self, lhs: &Box<Expr>, op: &Token, rhs: &Box<Expr>) -> String {
        let mut output: String = String::new();
        output.push_str(&self.compile_expr(&*lhs));
        output.push_str(&format!(" {} ", match op {
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Star => "*",
            Token::Slash => "/",
            _ => panic!("invalid binary op: {:?}", op)
        }));
        output.push_str(&self.compile_expr(&*rhs));
        output
    }
    fn compile_member_function_call(&mut self, left: &Box<Expr>, function_call: &FunctionCall) -> String {
        let mut output: String = String::new();
        output.push_str(&self.compile_expr(&*left));
        output.push_str("->");
        output.push_str(&self.compile_function_call(function_call));
        output
    }
    fn compile_member(&mut self, left: &Box<Expr>, member: &String) -> String {
        let mut output: String = String::new();
        output.push_str(&self.compile_expr(&*left));
        output.push_str(&format!(".{}", member));
        output
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path: String = args.next().expect("failed to get file path.");
    let contents: String = std::fs::read_to_string(path.clone()).expect("failed to read file.");

    println!("{:>12} {}...", "Compiling".green().bold(), &path);

    let mut gemstone: Gemstone = Gemstone::new();
    let tokens: Vec<Token> = gemstone.lex(&contents);
    let exprs: Vec<Expr> = gemstone.parse(&tokens);
    // TODO: gemstone.type_checker(&exprs);
    let output: String = gemstone.compile(&exprs);

    std::fs::write(&path.replace(".gem", ".cpp").as_str(), &output).expect("failed to write to file.");
    let output = std::process::Command::new("g++")
        .arg(&path.replace(".gem", ".cpp"))
        .arg("-o")
        .arg(&path.replace(".gem", ".out"))
        .output()
        .expect("failed to run c++ file.");

    if &output.status.code().unwrap() != &0 {
        println!("{:>12} {}", "Failed".red().bold(), &path.replace(".gem", ".cpp"));
        println!("{}", String::from_utf8_lossy(&output.stderr));
        return;
    }

    let output = std::process::Command::new(&path.replace(".gem", ".out"))
        .output()
        .expect("failed to run c++ file.");

    if &output.status.code().unwrap() != &0 {
        println!("{:>12} {}", "Failed".red().bold(), &path.replace(".gem", ".out"));
        println!("{}", String::from_utf8_lossy(&output.stderr));
    } else {
        println!("{:>12} {} (exit code: {})", "Running".green().bold(), &path.replace(".gem", ".out"), &output.status.code().unwrap());
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if std::env::args().any(|arg| arg == "--clean") {
        std::process::Command::new("rm")
            .arg("-rf")
            .arg("examples/*/*.out")
            .arg("examples/*/*.cpp")
            .output()
            .expect("failed to run c++ file.");
    }
    
}
