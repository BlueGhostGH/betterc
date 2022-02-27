use chumsky::{
    error::Simple,
    primitive::{end, filter_map, just},
    Error, Parser,
};

mod span;
mod token;

use token::Delimiter;
pub use token::{lexer, Token};

#[derive(Debug)]
pub enum Lit {
    Int(i32),
    Str(String),
}

#[derive(Debug)]
pub struct Expr {
    kind: ExprKind,
}

#[derive(Debug)]
pub enum ExprKind {
    Lit(Lit),
}

#[derive(Debug)]
pub enum ItemKind {
    Struct {
        generics: Option<Vec<String>>,
        fields: Option<Vec<(String, String)>>,
    },
    Func {
        inputs: Vec<(String, String)>,
    },
}

#[derive(Debug)]
pub struct Block {
    stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Item {
    ident: String,
    kind: ItemKind,
}

#[derive(Debug)]
pub struct Stmt {
    kind: StmtKind,
}

#[derive(Debug)]
pub enum StmtKind {
    Item(Item),
    Expr(Expr),
}

pub fn parser() -> impl chumsky::Parser<Token, Vec<Item>, Error = Simple<Token>> {
    let ident = filter_map(|span, tok| {
        if let Token::Ident(id) = tok {
            Ok(id)
        } else {
            Err(Simple::expected_input_found(span, Vec::new(), Some(tok)))
        }
    });

    let int = filter_map(|span, tok| {
        if let Token::Int(mut int) = tok {
            int.remove_matches('_');
            if let Ok(int) = int.parse() {
                Ok(Lit::Int(int))
            } else {
                Err(Simple::custom(
                    span as std::ops::Range<usize>,
                    "invalid integer literal",
                ))
            }
        } else {
            Err(Simple::expected_input_found(span, Vec::new(), Some(tok)))
        }
    });
    let r#str = filter_map(|span, tok| {
        if let Token::Str(r#str) = tok {
            Ok(Lit::Str(r#str))
        } else {
            Err(Simple::expected_input_found(span, Vec::new(), Some(tok)))
        }
    });
    let lit = int.or(r#str).map(ExprKind::Lit).map(|kind| Expr { kind });

    let field = ident.then_ignore(just(Token::Colon)).then(ident);
    let fields = field
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(
            just(Token::Open(Delimiter::Brace)),
            just(Token::Close(Delimiter::Brace)),
        );

    let generics = ident
        .separated_by(just(Token::Comma))
        .delimited_by(just(Token::Lt), just(Token::Gt))
        .or_not();

    let r#struct = just(Token::Struct)
        .ignore_then(ident)
        .then(generics)
        .then(fields.map(Some).or(just(Token::Semicolon).to(None)))
        .map(|((name, generics), fields)| Item {
            ident: name,
            kind: ItemKind::Struct { generics, fields },
        });

    let arg = ident.then_ignore(just(Token::Colon)).then(ident);
    let args = arg.separated_by(just(Token::Comma)).delimited_by(
        just(Token::Open(Delimiter::Paren)),
        just(Token::Close(Delimiter::Paren)),
    );

    let block = just(Token::Open(Delimiter::Brace))
        .ignore_then(just(Token::Close(Delimiter::Brace)))
        .ignored();

    let func = just(Token::Func)
        .ignore_then(ident)
        .then(args)
        .then_ignore(block)
        .map(|(name, args)| Item {
            ident: name,
            kind: ItemKind::Func { inputs: args },
        });

    r#struct.or(func).repeated().then_ignore(end())
}
