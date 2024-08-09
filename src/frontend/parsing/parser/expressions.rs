use crate::{
    errors::{
        context::ErrorContextBuilder,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
    },
    frontend::{
        lexing::{ComplexPunctuationKind::*, SimplePunctuationKind::*, TokenKind::*},
        parsing::ast::expressions::{
            BinaryExpressionKind, ExpressionKind, ExpressionNode, ResolutionExpressionKind,
            UnaryExpressionKind,
        },
    },
};

use super::Parser;

impl Parser<'_> {
    pub(super) fn parse_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let token = self.current();
        if token.kind == EOF {
            return Err(SyntaxError::default()
                .ctx(
                    ErrorContextBuilder::col(token.loc.start_col)
                        .from_src_and_ln(&self.src, token.loc.ln)
                        .build(),
                )
                .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Expression)));
        }

        self.parse_assignment_expression()
    }

    fn parse_assignment_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let mut left = self.parse_binary_expression(BinaryExpressionKind::lowest())?;

        if let ComplexPunctuation(Assignment(kind)) = self.current().kind {
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

        while let ComplexPunctuation(punct_kind) = self.current().kind {
            if matches!(punct_kind, Assignment(_) | InlineBody) {
                break;
            }

            let token = self.current();

            let Ok(punct_bin_kind) = BinaryExpressionKind::from_punct(&punct_kind) else {
                return Err(SyntaxError::default()
                    .ctx(
                        ErrorContextBuilder::span(token.loc.start_col, token.loc.end_col)
                            .from_src_and_ln(&self.src, token.loc.ln)
                            .build(),
                    )
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

        if self.current().kind == SimplePunctuation(ParenOpen) {
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
        if self.current().kind == SimplePunctuation(ParenOpen) {
            expr = self.parse_call_expression(expr)?;
        }

        return Ok(expr);
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<ExpressionNode>, SyntaxError> {
        self.expect(SimplePunctuation(ParenOpen))?;
        let mut args = Vec::new();

        loop {
            self.advance();
            if matches!(self.current().kind, SimplePunctuation(ParenClose) | EOF) {
                break;
            }

            let expr = self.parse_expression()?;
            args.push(expr);

            if self.current().kind != SimplePunctuation(Comma) {
                break;
            }
        }

        self.expect(SimplePunctuation(ParenClose))?;
        return Ok(args);
    }

    fn parse_resolution_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let mut left = self.parse_primary_expression()?;

        loop {
            self.advance();

            if !matches!(
                self.current().kind,
                ComplexPunctuation(MemberSeparator) | ComplexPunctuation(NamespaceSeparator)
            ) {
                break;
            }

            let kind = match self.current().kind {
                ComplexPunctuation(MemberSeparator) => ResolutionExpressionKind::Member,
                ComplexPunctuation(NamespaceSeparator) => ResolutionExpressionKind::Namespace,
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
            Identifier => Ok(ExpressionNode {
                kind: ExpressionKind::Identifier {
                    name: token.value.unwrap(),
                },
            }),
            Literal(lit) => Ok(ExpressionNode {
                kind: ExpressionKind::Literal {
                    kind: lit,
                    value: token.value.unwrap(),
                },
            }),

            ComplexPunctuation(punct_kind) => {
                let Ok(un_kind) = UnaryExpressionKind::from_punct(&punct_kind) else {
                    return Err(SyntaxError::default()
                        .ctx(
                            ErrorContextBuilder::span(token.loc.start_col, token.loc.end_col)
                                .from_src_and_ln(&self.src, token.loc.ln)
                                .build(),
                        )
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
            SimplePunctuation(ParenOpen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(SimplePunctuation(ParenClose))?;

                Ok(expr)
            }

            _ => Err(SyntaxError::default()
                .ctx(
                    ErrorContextBuilder::span(token.loc.start_col, token.loc.end_col)
                        .from_src_and_ln(&self.src, token.loc.ln)
                        .build(),
                )
                .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Token))
                .msg("specified token cannot be used for expressions")),
        }
    }
}
