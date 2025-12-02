use crate::{
    TokenIter,
    tokens::{Register, Token},
};
use anyhow::{Context, Result, anyhow};

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Xor,
}

impl BinaryOp {
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Xor => 1,
            BinaryOp::Add | BinaryOp::Sub => 2,
            BinaryOp::Mul | BinaryOp::Div => 3,
        }
    }

    pub fn calculate(&self, lhs: u64, rhs: u64) -> u64 {
        match self {
            BinaryOp::Add => lhs.wrapping_add(rhs),
            BinaryOp::Sub => lhs.wrapping_sub(rhs),
            BinaryOp::Mul => lhs.wrapping_mul(rhs),
            BinaryOp::Div => (lhs as i64).wrapping_div(rhs as i64) as u64, // Do as signed division
            BinaryOp::Xor => lhs ^ rhs,
        }
    }
}

impl TryFrom<Token> for BinaryOp {
    type Error = ();
    fn try_from(value: Token) -> std::result::Result<Self, Self::Error> {
        use BinaryOp::*;
        match value {
            Token::Plus => Ok(Add),
            Token::Sub => Ok(Sub),
            Token::Mul => Ok(Mul),
            Token::Div => Ok(Div),
            Token::Caret => Ok(Xor),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
}

impl UnaryOp {
    pub fn calculate(&self, val: u64) -> u64 {
        match self {
            UnaryOp::Neg => val.wrapping_neg(),
        }
    }
}

impl TryFrom<Token> for UnaryOp {
    type Error = ();
    fn try_from(value: Token) -> std::result::Result<Self, Self::Error> {
        use UnaryOp::*;
        match value {
            Token::Sub => Ok(Neg),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    None,
    Immediate,
    Addr,
}

#[derive(Debug, Clone)]
pub enum Node {
    Constant(u64, Mode),
    Register(Register),
    Identifier(String, Mode),
    BinaryOp {
        op: BinaryOp,
        left: Box<Self>,
        right: Box<Self>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Self>,
    },
    Expression(Box<Self>),
}

pub fn parse_expr(tokens: &mut TokenIter) -> Result<Box<Node>> {
    let mut left = parse_constant(tokens)?;

    while let Some(token) = tokens.peek()? {
        let op: BinaryOp = match token.try_into() {
            Ok(op) => op,
            Err(_) => break,
        };

        // Consume the peeked token
        let _ = tokens.next().unwrap().unwrap();

        let right = parse_constant(tokens)?;

        insert_into_tree(&mut left, op, right);
    }

    Ok(left)
}

fn parse_constant(tokens: &mut TokenIter) -> Result<Box<Node>> {
    let mut mode = Mode::None;
    match tokens.peek()? {
        Some(Token::Mul) => mode = Mode::Addr,
        Some(Token::Dollar) => mode = Mode::Immediate,
        _ => {}
    }

    if mode != Mode::None {
        _ = tokens.next();
    }

    let node = match tokens.next()?.with_context(|| "Expected token")? {
        Token::Number(num) => Node::Constant(num, mode),
        Token::Register(reg) => {
            if mode != Mode::None {
                return Err(anyhow!("Cannot use mode specifier with registers"));
            }

            Node::Register(reg)
        }
        Token::Identifier(id) => Node::Identifier(id, mode),
        Token::LBrace => {
            if mode != Mode::None {
                return Err(anyhow!("Cannot use mode specifier before a sub-expression"));
            }
            // tokens.next().unwrap().unwrap();
            let expr = parse_expr(tokens)?;
            match tokens.next()? {
                Some(Token::RBrace) => Node::Expression(expr),
                _ => return Err(anyhow!("Expected closing brace")),
            }
        }
        unary_op => {
            if mode != Mode::None {
                return Err(anyhow!("Cannot use mode specifier before a unary operator"));
            }
            match UnaryOp::try_from(unary_op) {
                Ok(op) => Node::UnaryOp {
                    op,
                    expr: parse_constant(tokens)?,
                },
                _ => return Err(anyhow!("Invalid token while parsing expression")),
            }
        }
    };
    Ok(Box::new(node))
}

fn insert_into_tree(left: &mut Box<Node>, op: BinaryOp, right: Box<Node>) {
    /// Replaces `left` with a new Node::Binary with the operation `op` and the left and right
    /// fields being the old `left` value and the given `right` value
    fn insert_as_binary_op(left: &mut Box<Node>, op: BinaryOp, right: Box<Node>) {
        // Allocate the box here to prevent a panic from happening during the time there are
        // two Box's that "own" the same memory which causes memory unsafety
        let uninit = Box::new_uninit();
        let op = Node::BinaryOp {
            op,
            // SAFETY: the mutable reference `left` is never read after this and no code can panic between here and
            // when we ptr::write left to prevent double frees and other undefined behavior
            left: unsafe { std::ptr::read(left) },
            right,
        };
        // SAFETY: `left` is a reference so its garunteed to be a safe pointer
        unsafe { std::ptr::write(left, Box::write(uninit, op)) };
    }

    match &mut **left {
        // These expression nodes are always insertion points for any operator
        Node::Constant(_, _)
        | Node::Register(_)
        | Node::Identifier(_, _)
        | Node::Expression(_)
        | Node::UnaryOp { .. } => {
            insert_as_binary_op(left, op, right);
        }
        Node::BinaryOp {
            op: op2,
            left: _,
            right: right2,
        } => {
            if op.precedence() > op2.precedence() {
                // Precedence is greater so we descend further down the tree to find the right spot
                // for this operation to go
                insert_into_tree(right2, op, right);
            } else {
                // Precdence is less than or equal to the operator attemping to be inserted
                insert_as_binary_op(left, op, right);
            }
        }
    }
}
