//! Built-in functions on ordinary values (01 §12, "Words and sets"), and the hooks through
//! which later parts of the engine (machines, languages) extend the interpreter.

use crate::diag::{Diagnostic, Span};
use crate::interp::{Flow, Interp, R, err};
use crate::value::{Sym, Value};
use std::collections::BTreeSet;
use std::rc::Rc;

/// Positional and named arguments of a call, with their spans.
pub type Args = Vec<(Value, Span)>;
pub type Named = Vec<(String, Value, Span)>;

fn arity(name: &str, args: &Args, n: usize, span: Span) -> R<()> {
    if args.len() != n {
        return err(
            "E507",
            span,
            format!(
                "`{name}` takes {n} argument{}, but {} {} given",
                if n == 1 { "" } else { "s" },
                args.len(),
                if args.len() == 1 { "was" } else { "were" }
            ),
        );
    }
    Ok(())
}

fn wrong(name: &str, v: &Value, span: Span) -> Flow {
    Flow::Error(Diagnostic::error(
        "E504",
        span,
        format!(
            "`{name}` doesn't apply to {} {}",
            v.type_name(),
            v.display()
        ),
    ))
}

impl Interp<'_> {
    // `'_` is an anonymous lifetime: "whatever lifetime Interp has".
    pub fn call_builtin(&mut self, name: &str, args: Args, named: Named, span: Span) -> R<Value> {
        if let Some(v) = self.call_builtin_extra(name, &args, &named, span)? {
            return Ok(v);
        }
        if let Some((n, _, s)) = named.first() {
            return err("E507", *s, format!("`{name}` has no named parameter `{n}`"));
        }
        let first = args.first().map(|(v, _)| v.clone()).unwrap_or(Value::None);
        let fspan = args.first().map(|(_, s)| *s).unwrap_or(span);
        match name {
            "len" => {
                arity(name, &args, 1, span)?;
                let n = match &first {
                    Value::Word(w) => w.len(),
                    Value::List(l) | Value::Tuple(l) => l.len(),
                    Value::Set(s) => s.len(),
                    Value::Map(m) => m.len(),
                    other => return Err(wrong(name, other, fspan)),
                };
                Ok(Value::Int(n as i64))
            }
            "count" => {
                arity(name, &args, 2, span)?;
                let x = args.get(1).map(|(v, _)| v.clone()).unwrap_or(Value::None);
                match (&first, &x) {
                    (Value::Word(w), Value::Sym(s)) => {
                        Ok(Value::Int(w.iter().filter(|c| *c == s).count() as i64))
                    }
                    (Value::List(l), x) => {
                        Ok(Value::Int(l.iter().filter(|c| *c == x).count() as i64))
                    }
                    (other, _) => Err(wrong(name, other, fspan)),
                }
            }
            "rev" => {
                arity(name, &args, 1, span)?;
                match &first {
                    Value::Word(w) => Ok(Value::Word(Rc::new(w.iter().rev().cloned().collect()))),
                    Value::List(l) => Ok(Value::list(l.iter().rev().cloned().collect())),
                    other => self.rev_extra(other.clone(), fspan),
                }
            }
            "set" => {
                arity(name, &args, 1, span)?;
                let items = self.items_of(&first, fspan)?;
                Ok(Value::set(items.into_iter().collect()))
            }
            "list" | "sort" => {
                arity(name, &args, 1, span)?;
                let mut items = self.items_of(&first, fspan)?;
                if name == "sort" {
                    items.sort();
                }
                Ok(Value::list(items))
            }
            "keys" | "values" => {
                arity(name, &args, 1, span)?;
                match &first {
                    Value::Map(m) if name == "keys" => Ok(Value::set(m.keys().cloned().collect())),
                    Value::Map(m) => Ok(Value::list(m.values().cloned().collect())),
                    other => Err(wrong(name, other, fspan)),
                }
            }
            "first" | "last" => {
                arity(name, &args, 1, span)?;
                let items = self.items_of(&first, fspan)?;
                let x = if name == "first" {
                    items.first()
                } else {
                    items.last()
                };
                Ok(x.cloned().unwrap_or(Value::None))
            }
            "min" | "max" | "sum" => {
                let items = if args.len() == 1 {
                    self.items_of(&first, fspan)?
                } else {
                    args.iter().map(|(v, _)| v.clone()).collect()
                };
                if name == "sum" {
                    let mut total: i64 = 0;
                    for v in &items {
                        match v {
                            Value::Int(n) => {
                                total = total.checked_add(*n).ok_or_else(|| {
                                    Flow::Error(Diagnostic::error(
                                        "E501",
                                        span,
                                        "integer overflow (int is 64-bit)",
                                    ))
                                })?
                            }
                            other => return Err(wrong(name, other, span)),
                        }
                    }
                    return Ok(Value::Int(total));
                }
                let x = if name == "min" {
                    items.iter().min()
                } else {
                    items.iter().max()
                };
                Ok(x.cloned().unwrap_or(Value::None))
            }
            "abs" => {
                arity(name, &args, 1, span)?;
                match first {
                    Value::Int(n) => n
                        .checked_abs()
                        .map(Value::Int)
                        .map_or_else(|| err("E501", span, "integer overflow (int is 64-bit)"), Ok),
                    other => Err(wrong(name, &other, fspan)),
                }
            }
            "prefixes" | "suffixes" | "substrings" => {
                arity(name, &args, 1, span)?;
                let Value::Word(w) = &first else {
                    return Err(wrong(name, &first, fspan));
                };
                let n = w.len();
                let mut out = BTreeSet::new();
                for i in 0..=n {
                    self.tick(span)?;
                    match name {
                        "prefixes" => {
                            out.insert(Value::Word(Rc::new(w.get(..i).unwrap_or(&[]).to_vec())));
                        }
                        "suffixes" => {
                            out.insert(Value::Word(Rc::new(w.get(i..).unwrap_or(&[]).to_vec())));
                        }
                        _ => {
                            for j in i..=n {
                                self.tick(span)?;
                                out.insert(Value::Word(Rc::new(
                                    w.get(i..j).unwrap_or(&[]).to_vec(),
                                )));
                            }
                        }
                    }
                }
                Ok(Value::set(out))
            }
            "power_set" => {
                arity(name, &args, 1, span)?;
                let items = self.items_of(&first, fspan)?;
                if items.len() > 20 {
                    return err(
                        "E504",
                        fspan,
                        "power_set of more than 20 elements is too large",
                    );
                }
                let mut out = BTreeSet::new();
                for mask in 0u32..(1u32 << items.len()) {
                    self.tick(span)?;
                    let sub: BTreeSet<Value> = items
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| mask & (1 << i) != 0)
                        .map(|(_, v)| v.clone())
                        .collect();
                    out.insert(Value::set(sub));
                }
                Ok(Value::set(out))
            }
            "symbols" => {
                arity(name, &args, 1, span)?;
                match &first {
                    Value::Word(w) => Ok(Value::set(
                        w.iter().map(|s| Value::Sym(s.clone())).collect(),
                    )),
                    other => Err(wrong(name, other, fspan)),
                }
            }
            "word" => {
                // word(list of symbols) or word(symbol)
                arity(name, &args, 1, span)?;
                match &first {
                    Value::Sym(s) => Ok(Value::Word(Rc::new(vec![s.clone()]))),
                    Value::Word(_) => Ok(first.clone()),
                    Value::List(l) => {
                        let mut w: Vec<Sym> = Vec::new();
                        for v in l.iter() {
                            match v {
                                Value::Sym(s) => w.push(s.clone()),
                                Value::Word(x) => w.extend(x.iter().cloned()),
                                other => return Err(wrong(name, other, fspan)),
                            }
                        }
                        Ok(Value::Word(Rc::new(w)))
                    }
                    other => Err(wrong(name, other, fspan)),
                }
            }
            "atom" => {
                arity(name, &args, 1, span)?;
                match &first {
                    Value::Word(w) => Ok(Value::Atom(Rc::from(w.concat().as_str()))),
                    Value::Sym(s) => Ok(Value::Atom(s.clone())),
                    Value::Atom(_) => Ok(first.clone()),
                    other => Err(wrong(name, other, fspan)),
                }
            }
            _ => err(
                "E509",
                span,
                format!("`{name}` is not available yet in this version of Delta 2"),
            ),
        }
    }
}
