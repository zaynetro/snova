use std::collections::HashMap;

use anyhow::{anyhow, Result};

/// Format the template to insert variables from the context.
/// Template example "Hello {name}". "name" should be present in the context.
pub fn format(template: impl AsRef<str>, context: HashMap<String, String>) -> Result<String> {
    let mut out = String::new();
    let mut var = None;

    for c in template.as_ref().chars() {
        match c {
            '{' => {
                var = Some("".to_string());
            }
            '}' => {
                match var {
                    Some(name) if name.is_empty() => {
                        return Err(anyhow!(
                            "You need to specify variable name in between '{' and '}' (e.g '{name}')"
                        ));
                    }
                    Some(name) => match context.get(&name) {
                        Some(value) => {
                            out.push_str(value);
                        }
                        None => {
                            return Err(anyhow!(
                                "Variable '{}' is not present in the context.",
                                name
                            ));
                        }
                    },
                    None => {
                        return Err(anyhow!("Unexpected '}'. Do you have an opening one?"));
                    }
                }

                var = None;
            }
            _ if var.is_some() => {
                var.as_mut().unwrap().push(c);
            }
            _ => {
                out.push(c);
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ok() {
        let template = "Hello {name}!";
        let mut ctx = HashMap::new();
        ctx.insert("name".to_string(), "Bond".to_string());

        let res = format(template, ctx);
        assert!(res.is_ok(), "Result {:?}", res);

        assert_eq!("Hello Bond!", res.unwrap());
    }
}
