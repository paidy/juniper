use crate::{
    ast::{Field, Operation},
    parser::Spanning,
    validation::{ValidatorContext, Visitor},
    value::ScalarValue,
};

pub struct Aliases {
    alias_count: u8,
    max_allowed: u8,
}

pub fn factory<'a>() -> Aliases {
    Aliases {
        alias_count: 0,
        max_allowed: 1,
    }
}

impl<'a, S> Visitor<'a, S> for Aliases
where
    S: ScalarValue,
{
    fn enter_operation_definition(
        &mut self,
        _ctx: &mut ValidatorContext<'a, S>,
        _op: &'a Spanning<Operation<'a, S>>,
    ) {
        self.alias_count = 0; // Reset for each operation
    }

    fn enter_field(
        &mut self,
        ctx: &mut ValidatorContext<'a, S>,
        field: &'a Spanning<Field<'a, S>>,
    ) {
        let alias_name = &field.item.alias; // Get the Spanning<String> for the alias

        if let Some(alias) = alias_name {
            self.alias_count += 1;
            if self.alias_count > self.max_allowed {
                ctx.report_error(&error_message(&alias.item), &[alias.start]);
            }
        }
    }
}

fn error_message(alias_name: &str) -> String {
    format!("There can only be one alias, {} is not allowed", alias_name)
}

#[cfg(test)]
mod tests {
    use super::{error_message, factory};

    use crate::{
        parser::SourcePosition,
        validation::{expect_fails_rule, expect_passes_rule, RuleError},
        value::DefaultScalarValue,
    };

    #[test]
    fn single_alias_allowed() {
        expect_passes_rule::<_, _, DefaultScalarValue>(
            factory,
            r#"
            mutation CreateLiquidToken {
              liquidApplication: createLiquidSdkApplication {
                token
                applicantId
              }
            }
            "#,
        );

        expect_passes_rule::<_, _, DefaultScalarValue>(
            factory,
            r#"
          mutation UpdateLiquidToken {
              liquidApplication: createLiquidSdkUpdateApplication {
                token
                applicantId
              }
            }
            "#,
        );
    }

    #[test]
    fn multiple_field_aliases_not_allowed() {
        expect_fails_rule::<_, _, DefaultScalarValue>(
            factory,
            r#"
        query MyQuery {
            myField1: my_field,
            myField2: my_field,
            myField3: my_field
        }
        "#,
            &[
                RuleError::new(
                    &error_message("myField2"),
                    &[
                        SourcePosition::new(69, 3, 12), // includes whitespaces
                    ],
                ),
                RuleError::new(
                    &error_message("myField3"),
                    &[SourcePosition::new(101, 4, 12)],
                ),
            ],
        );
    }

    #[test]
    fn multiple_query_aliases_not_allowed() {
        expect_fails_rule::<_, _, DefaultScalarValue>(
            factory,
            r#"
          query Hero {
            empireHero: hero(episode: EMPIRE) {
              name
            }
            jediHero: hero(episode: JEDI) {
              name
            }
            republicHero: hero(episode: REPUBLIC) {
                name
            }
          }
        "#,
            &[
                RuleError::new(
                    &error_message("jediHero"),
                    &[SourcePosition::new(117, 5, 12)],
                ),
                RuleError::new(
                    &error_message("republicHero"),
                    &[SourcePosition::new(194, 8, 12)],
                ),
            ],
        );
    }

    #[test]
    fn no_aliases() {
        expect_passes_rule::<_, _, DefaultScalarValue>(
            factory,
            r#"
            query MyQuery {
                my_field
            }
            "#,
        );
    }
}
