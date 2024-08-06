use crate::{
    errors::{
        context::ErrorContext,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
    },
    frontend::{
        lexing::{Punctuations, Tokens},
        parsing::ast::expressions::{
            BinaryExpressionKind, ExpressionKind, ExpressionNode, ResolutionExpressionKind,
            UnaryExpressionKind,
        },
    },
    source::SourcePoint,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let token = self.current();
        if token.kind == Tokens::EOF {
            return Err(SyntaxError::default()
                .ctx(ErrorContext::point(
                    token.span.start.ln,
                    token.span.start.col,
                    self.src.clone(),
                ))
                .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Expression)));
        }

        self.parse_assignment_expression()
    }

    fn parse_assignment_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let mut left = self.parse_binary_expression(BinaryExpressionKind::lowest())?;

        if let Tokens::Punctuation(Punctuations::Assignment(kind)) = self.current().kind {
            self.advance();
            let right = self.parse_assignment_expression()?;

            left = ExpressionNode {
                kind: ExpressionKind::Assignment {
                    left: Box::new(left),
                    op: kind,
                    right: Box::new(right),
                },
            };
        }

        return Ok(left);
    }

    fn parse_binary_expression(
        &mut self,
        bin_kind: BinaryExpressionKind,
    ) -> Result<ExpressionNode, SyntaxError> {
        let mut expr = self.parse_binary_expression_operand(&bin_kind)?;

        while let Tokens::Punctuation(punct_kind) = self.current().kind {
            if matches!(
                punct_kind,
                Punctuations::Assignment(_) | Punctuations::InlineBody
            ) {
                break;
            }

            let token = self.current();

            let Ok(punct_bin_kind) = BinaryExpressionKind::from_punct(&punct_kind) else {
                return Err(SyntaxError::default()
                    .ctx(ErrorContext::point(
                        token.span.start.ln,
                        token.span.start.col,
                        self.src.clone(),
                    ))
                    .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Token))
                    .msg("specified token is not a binary operator"));
            };

            if punct_bin_kind != bin_kind {
                break;
            }

            self.advance();

            let right = self.parse_binary_expression_operand(&bin_kind)?;
            expr = ExpressionNode {
                kind: ExpressionKind::Binary {
                    kind: punct_bin_kind.clone(),
                    left: Box::new(expr),
                    op: punct_kind,
                    right: Box::new(right),
                },
            }
        }

        return Ok(expr);
    }

    fn parse_binary_expression_operand(
        &mut self,
        bin_kind: &BinaryExpressionKind,
    ) -> Result<ExpressionNode, SyntaxError> {
        let expr = if let Ok(more_precedence) =
            BinaryExpressionKind::from_precedence(bin_kind.precedence() + 1)
        {
            self.parse_binary_expression(more_precedence)?
        } else {
            self.parse_call_or_resolution_expression()?
        };

        return Ok(expr);
    }

    fn parse_call_or_resolution_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let res = self.parse_resolution_expression()?;

        if self.current().kind == Tokens::Punctuation(Punctuations::ParenOpen) {
            return Ok(self.parse_call_expression(res)?);
        }

        return Ok(res);
    }

    fn parse_call_expression(
        &mut self,
        res: ExpressionNode,
    ) -> Result<ExpressionNode, SyntaxError> {
        let args = self.parse_call_arguments()?;

        let mut expr = ExpressionNode {
            kind: ExpressionKind::Call {
                res: Box::new(res),
                args,
            },
        };

        self.advance();
        if self.current().kind == Tokens::Punctuation(Punctuations::ParenOpen) {
            expr = self.parse_call_expression(expr)?;
        }

        return Ok(expr);
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<ExpressionNode>, SyntaxError> {
        self.expect(Tokens::Punctuation(Punctuations::ParenOpen))?;
        let mut args = Vec::new();

        loop {
            self.advance();
            if matches!(
                self.current().kind,
                Tokens::Punctuation(Punctuations::ParenClose) | Tokens::EOF
            ) {
                break;
            }

            let expr = self.parse_expression()?;
            args.push(expr);

            if self.current().kind != Tokens::Punctuation(Punctuations::Comma) {
                break;
            }
        }

        self.expect(Tokens::Punctuation(Punctuations::ParenClose))?;
        return Ok(args);
    }

    fn parse_resolution_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let mut left = self.parse_primary_expression()?;

        loop {
            self.advance();

            if !matches!(
                self.current().kind,
                Tokens::Punctuation(Punctuations::MemberSeparator)
                    | Tokens::Punctuation(Punctuations::NamespaceSeparator)
            ) {
                break;
            }

            let kind = match self.current().kind {
                Tokens::Punctuation(Punctuations::MemberSeparator) => {
                    ResolutionExpressionKind::Member
                }
                Tokens::Punctuation(Punctuations::NamespaceSeparator) => {
                    ResolutionExpressionKind::Namespace
                }
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_primary_expression()?;

            left = ExpressionNode {
                kind: ExpressionKind::Resolution {
                    left: Box::new(left),
                    right: Box::new(right),
                    kind,
                },
            };
        }

        return Ok(left);
    }

    fn parse_primary_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let token = self.current();

        match token.kind {
            Tokens::Identifier => Ok(ExpressionNode {
                kind: ExpressionKind::Identifier {
                    name: token.value.unwrap(),
                },
            }),
            Tokens::Literal(lit) => Ok(ExpressionNode {
                kind: ExpressionKind::Literal {
                    kind: lit,
                    value: token.value.unwrap(),
                },
            }),

            Tokens::Punctuation(punct_kind) => {
                if punct_kind == Punctuations::ParenOpen {
                    self.advance();
                    let expr = self.parse_expression()?;
                    self.expect(Tokens::Punctuation(Punctuations::ParenClose))?;

                    return Ok(expr);
                }

                let Ok(un_kind) = UnaryExpressionKind::from_punct(&punct_kind) else {
                    return Err(SyntaxError::default()
                        .ctx(ErrorContext::span(
                            SourcePoint::new(token.span.start.ln, token.span.start.col),
                            SourcePoint::new(token.span.start.ln, token.span.end.col),
                            self.src.clone(),
                        ))
                        .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Token))
                        .msg("specified token is not a unary operator"));
                };

                self.advance();
                let right = self.parse_primary_expression()?;

                Ok(ExpressionNode {
                    kind: ExpressionKind::Unary {
                        kind: un_kind,
                        op: punct_kind,
                        right: Box::new(right),
                    },
                })
            }

            _ => Err(SyntaxError::default()
                .ctx(ErrorContext::span(
                    SourcePoint::new(token.span.start.ln, token.span.start.col),
                    SourcePoint::new(token.span.start.ln, token.span.end.col),
                    self.src.clone(),
                ))
                .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Token))
                .msg("specified token cannot be used for expressions")),
        }
    }
}
