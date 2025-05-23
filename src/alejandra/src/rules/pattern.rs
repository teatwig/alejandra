pub(crate) fn rule(
    build_ctx: &crate::builder::BuildCtx,
    node: &rnix::SyntaxNode,
) -> std::collections::LinkedList<crate::builder::Step> {
    let mut steps = std::collections::LinkedList::new();

    let pattern = crate::parsers::pattern::parse(build_ctx, node);

    let has_comments_between_curly_b =
        pattern.arguments.iter().any(|argument| {
            argument.comment_after.is_some()
                || !argument.comments_before.is_empty()
        });

    let has_comments = has_comments_between_curly_b
        || !pattern.comments_after_initial_at.is_empty()
        || !pattern.comments_before_end_at.is_empty();

    let has_ellipsis = pattern.arguments.iter().any(|argument| {
        if argument.item.is_some() {
            argument.item.as_ref().unwrap().kind()
                == rnix::SyntaxKind::TOKEN_ELLIPSIS
        } else {
            false
        }
    });

    let arguments_count = pattern.arguments.len();

    let arguments_count_for_tall = if has_ellipsis { 2 } else { 1 };

    let vertical = has_comments
        || arguments_count > arguments_count_for_tall
        || (arguments_count > 0 && has_comments_between_curly_b)
        || build_ctx.vertical;

    // x @
    if let Some(element) = &pattern.initial_at {
        let element = element.clone();
        if vertical {
            steps.push_back(crate::builder::Step::FormatWider(element));
        } else {
            steps.push_back(crate::builder::Step::Format(element));
        }
    }

    // /**/
    if !pattern.comments_after_initial_at.is_empty() {
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
        for text in pattern.comments_after_initial_at {
            steps.push_back(crate::builder::Step::Comment(text));
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        }
    } else if pattern.initial_at.is_some() {
        steps.push_back(crate::builder::Step::Whitespace);
    }

    // {
    steps.push_back(crate::builder::Step::Token(
        rnix::SyntaxKind::TOKEN_L_BRACE,
        "{".to_string(),
    ));
    if vertical {
        steps.push_back(crate::builder::Step::Indent);
    }

    // arguments
    for (index, argument) in pattern.arguments.into_iter().enumerate() {
        if vertical {
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        } else {
            steps.push_back(crate::builder::Step::Whitespace);
        }

        // /**/
        if !argument.comments_before.is_empty() {
            for text in argument.comments_before {
                steps.push_back(crate::builder::Step::Comment(text));
                steps.push_back(crate::builder::Step::NewLine);
                steps.push_back(crate::builder::Step::Pad);
            }
        }

        // argument
        let element = argument.item.unwrap();
        let element_kind = element.kind();
        if vertical {
            steps.push_back(crate::builder::Step::FormatWider(element));
        } else {
            steps.push_back(crate::builder::Step::Format(element));
        };

        // ,
        if vertical {
            if !matches!(element_kind, rnix::SyntaxKind::TOKEN_ELLIPSIS) {
                steps.push_back(crate::builder::Step::Token(
                    rnix::SyntaxKind::TOKEN_COMMA,
                    ",".to_string(),
                ));
            }
        } else if index + 1 < arguments_count {
            steps.push_back(crate::builder::Step::Token(
                rnix::SyntaxKind::TOKEN_COMMA,
                ",".to_string(),
            ));
        };

        // possible inline comment
        if let Some(text) = argument.comment_after {
            if text.starts_with('#') {
                steps.push_back(crate::builder::Step::Whitespace);
            } else {
                steps.push_back(crate::builder::Step::NewLine);
                steps.push_back(crate::builder::Step::Pad);
            }
            steps.push_back(crate::builder::Step::Comment(text));
        }
    }

    // /**/
    let has_comments_before_curly_b_close =
        !pattern.comments_before_curly_b_close.is_empty();
    for text in pattern.comments_before_curly_b_close {
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
        steps.push_back(crate::builder::Step::Comment(text));
    }

    // }
    if vertical {
        steps.push_back(crate::builder::Step::Dedent);
        if arguments_count > 0 || has_comments_before_curly_b_close {
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        }
    } else if arguments_count > 0 {
        steps.push_back(crate::builder::Step::Whitespace);
    }
    steps.push_back(crate::builder::Step::Token(
        rnix::SyntaxKind::TOKEN_R_BRACE,
        "}".to_string(),
    ));

    // /**/
    if pattern.comments_before_end_at.is_empty() {
        if pattern.end_at.is_some() {
            steps.push_back(crate::builder::Step::Whitespace);
        }
    } else {
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
        for text in pattern.comments_before_end_at {
            steps.push_back(crate::builder::Step::Comment(text));
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        }
    }

    // @ x
    if let Some(element) = pattern.end_at {
        if vertical {
            steps.push_back(crate::builder::Step::FormatWider(element));
        } else {
            steps.push_back(crate::builder::Step::Format(element));
        }
    }

    steps
}
