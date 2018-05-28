use nom::types::CompleteStr;

use helpers::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SmallStatement {
    // TODO
    Del(Vec<Name>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Statement {
    Simple(Vec<SmallStatement>),
    Compound(Box<CompoundStatement>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompoundStatement {
    // TODO
    If(Vec<(Test, Vec<Statement>)>, Option<Vec<Statement>>),
}


named_args!(pub statement(first_indent: usize, indent: usize) <CompleteStr, Statement>,
  alt!(
    call!(simple_stmt, first_indent) => { |stmts| Statement::Simple(stmts) }
  | call!(compound_stmt, first_indent, indent) => { |stmt| Statement::Compound(Box::new(stmt)) }
  )
);

named_args!(simple_stmt(indent: usize) <CompleteStr, Vec<SmallStatement>>,
  do_parse!(
    count!(char!(' '), indent) >>
    first_stmts: many0!(terminated!(call!(small_stmt), semicolon)) >>
    last_stmt: small_stmt >>
    opt!(semicolon) >> ({
      let mut stmts = first_stmts;
      stmts.push(last_stmt);
      stmts
    })
  )
);

named_args!(block(indent: usize) <CompleteStr, Vec<Statement>>,
  do_parse!(
    newline >>
    new_indent: do_parse!(
      count!(char!(' '), indent) >>
      new_spaces: many1!(char!(' ')) >> ({
        indent + new_spaces.len()
      })
    ) >>
    stmt: many1!(call!(statement, 0, new_indent)) >> (
      stmt
    )
  )
);

named_args!(cond_and_block(indent: usize) <CompleteStr, (String, Vec<Statement>)>,
  do_parse!(
    cond: ws2!(tag!("foo")) >>
    ws2!(char!(':')) >>
    block: call!(block, indent) >> (
      (cond.to_string(), block)
    )
  )
);

named_args!(compound_stmt(first_indent: usize, indent: usize) <CompleteStr, CompoundStatement>,
  do_parse!(
    count!(char!(' '), first_indent) >>
    content: alt!(
      tuple!(
        preceded!(tag!("if "), call!(cond_and_block, indent)),
        many0!(
          preceded!(
            tuple!(newline, count!(char!(' '), indent), tag!("elif ")),
            call!(cond_and_block, indent)
          )
        ),
        opt!(
          preceded!(
            tuple!(newline, count!(char!(' '), indent), tag!("else"), ws2!(char!(':'))),
            call!(block, indent)
          )
        )
      ) => { |(if_block, elif_blocks, else_block)| {
        let mut blocks: Vec<_> = elif_blocks;
        blocks.insert(0, if_block);
        CompoundStatement::If(blocks, else_block)
      }}
    ) >> (
      content
    )
  )
);

named!(small_stmt<CompleteStr, SmallStatement>,
  alt!(
    del_stmt => { |atoms| SmallStatement::Del(atoms) }
    // TODO
  )
);

named!(del_stmt<CompleteStr, Vec<String>>,
  preceded!(tag!("del "), ws2!(many1!(name)))
  // TODO
);

#[cfg(test)]
mod tests {
    use super::*;
    use nom::types::CompleteStr as CS;

    #[test]
    fn test_statement_indent() {
        assert_eq!(statement(CS("del foo"), 0, 0), Ok((CS(""), Statement::Simple(vec![SmallStatement::Del(vec!["foo".to_string()])]))));
        assert_eq!(statement(CS(" del foo"), 1, 1), Ok((CS(""), Statement::Simple(vec![SmallStatement::Del(vec!["foo".to_string()])]))));
        assert!(statement(CS("del foo"), 1, 1).is_err());
        assert!(statement(CS(" del foo"), 0, 0).is_err());
    }

    #[test]
    fn test_block() {
        assert_eq!(block(CS("\n del foo"), 0), Ok((CS(""), vec![Statement::Simple(vec![SmallStatement::Del(vec!["foo".to_string()])])])));
        assert_eq!(block(CS("\n  del foo"), 1), Ok((CS(""), vec![Statement::Simple(vec![SmallStatement::Del(vec!["foo".to_string()])])])));
        assert_eq!(block(CS("\n      del foo"), 1), Ok((CS(""), vec![Statement::Simple(vec![SmallStatement::Del(vec!["foo".to_string()])])])));
        assert!(block(CS("\ndel foo"), 0).is_err());
        assert!(block(CS("\ndel foo"), 1).is_err());
        assert!(block(CS("\n del foo"), 1).is_err());
    }

    #[test]
    fn test_if() {
        assert_eq!(compound_stmt(CS("if foo:\n del bar"), 0, 0), Ok((CS(""),
            CompoundStatement::If(
                vec![
                    (
                        "foo".to_string(),
                        vec![
                            Statement::Simple(vec![
                                SmallStatement::Del(vec!["bar".to_string()])
                            ])
                        ]
                    ),
                ],
                None
            )
        )));
    }

    #[test]
    fn test_elif() {
        assert_eq!(compound_stmt(CS("if foo:\n del bar\nelif foo:\n del baz"), 0, 0), Ok((CS(""),
            CompoundStatement::If(
                vec![
                    (
                        "foo".to_string(),
                        vec![
                            Statement::Simple(vec![
                                SmallStatement::Del(vec!["bar".to_string()])
                            ])
                        ]
                    ),
                    (
                        "foo".to_string(),
                        vec![
                            Statement::Simple(vec![
                                SmallStatement::Del(vec!["baz".to_string()])
                            ])
                        ]
                    ),
                ],
                None
            )
        )));
    }

    #[test]
    fn test_else() {
        assert_eq!(compound_stmt(CS("if foo:\n del bar\nelse:\n del qux"), 0, 0), Ok((CS(""),
            CompoundStatement::If(
                vec![
                    (
                        "foo".to_string(),
                        vec![
                            Statement::Simple(vec![
                                SmallStatement::Del(vec!["bar".to_string()])
                            ])
                        ]
                    ),
                ],
                Some(
                    vec![
                        Statement::Simple(vec![
                            SmallStatement::Del(vec!["qux".to_string()])
                        ])
                    ]
                )
            )
        )));
    }

    #[test]
    fn test_elif_else() {
        assert_eq!(compound_stmt(CS("if foo:\n del bar\nelif foo:\n del baz\nelse:\n del qux"), 0, 0), Ok((CS(""),
            CompoundStatement::If(
                vec![
                    (
                        "foo".to_string(),
                        vec![
                            Statement::Simple(vec![
                                SmallStatement::Del(vec!["bar".to_string()])
                            ])
                        ]
                    ),
                    (
                        "foo".to_string(),
                        vec![
                            Statement::Simple(vec![
                                SmallStatement::Del(vec!["baz".to_string()])
                            ])
                        ]
                    ),
                ],
                Some(
                    vec![
                        Statement::Simple(vec![
                            SmallStatement::Del(vec!["qux".to_string()])
                        ])
                    ]
                )
            )
        )));
    }

    #[test]
    fn test_nested_if() {
        assert_eq!(compound_stmt(CS("if foo:\n if foo:\n  del bar"), 0, 0), Ok((CS(""),
            CompoundStatement::If(
                vec![
                    (
                        "foo".to_string(),
                        vec![
                            Statement::Compound(Box::new(
                              CompoundStatement::If(
                                  vec![
                                      (
                                          "foo".to_string(),
                                          vec![
                                              Statement::Simple(vec![
                                                  SmallStatement::Del(vec!["bar".to_string()])
                                              ])
                                          ]
                                      ),
                                  ],
                                  None
                                )
                            ))
                        ]
                    ),
                ],
                None
            )
        )));
    }

    #[test]
    fn test_dangling_else_1() {
        assert_eq!(compound_stmt(CS("if foo:\n if foo:\n  del bar\nelse:\n del qux"), 0, 0), Ok((CS(""),
            CompoundStatement::If(
                vec![
                    (
                        "foo".to_string(),
                        vec![
                            Statement::Compound(Box::new(
                              CompoundStatement::If(
                                  vec![
                                      (
                                          "foo".to_string(),
                                          vec![
                                              Statement::Simple(vec![
                                                  SmallStatement::Del(vec!["bar".to_string()])
                                              ])
                                          ]
                                      ),
                                  ],
                                  None
                                )
                            ))
                        ]
                    ),
                ],
                Some(
                    vec![
                        Statement::Simple(vec![
                            SmallStatement::Del(vec!["qux".to_string()])
                        ])
                    ]
                )
            )
        )));
    }

    #[test]
    fn test_dangling_else_2() {
        assert_eq!(compound_stmt(CS("if foo:\n if foo:\n  del bar\n else:\n  del qux"), 0, 0), Ok((CS(""),
            CompoundStatement::If(
                vec![
                    (
                        "foo".to_string(),
                        vec![
                            Statement::Compound(Box::new(
                              CompoundStatement::If(
                                  vec![
                                      (
                                          "foo".to_string(),
                                          vec![
                                              Statement::Simple(vec![
                                                  SmallStatement::Del(vec!["bar".to_string()])
                                              ])
                                          ]
                                      ),
                                  ],
                                  Some(
                                      vec![
                                          Statement::Simple(vec![
                                              SmallStatement::Del(vec!["qux".to_string()])
                                          ])
                                      ]
                                  )
                                )
                            ))
                        ]
                    ),
                ],
                None
            )
        )));
    }
}
