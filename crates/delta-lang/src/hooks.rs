//! Extension points of the interpreter for engine objects (machines, languages, regexes,
//! grammars, models). Each hook handles the object kinds that exist and reports the rest.

use crate::ast::*;
use crate::builtins::{Args, Named};
use crate::diag::Span;
use crate::interp::{Interp, R, err};
use crate::value::Value;
use serde_json::json;

fn not_yet<T>(what: &str, span: Span) -> R<T> {
    err(
        "E509",
        span,
        format!("{what} is not available yet in this version of Delta 2"),
    )
}

impl Interp<'_> {
    pub fn lookup_extra(&self, _name: &str) -> Option<Value> {
        None
    }

    pub fn items_of_extra(&mut self, v: &Value, span: Span) -> R<Vec<Value>> {
        err(
            "E504",
            span,
            format!("can't iterate over {} {}", v.type_name(), v.display()),
        )
    }

    pub fn binary_extra(&mut self, op: BinOp, l: Value, r: Value, span: Span) -> R<Value> {
        err(
            "E504",
            span,
            format!(
                "`{}` doesn't apply to {} and {} ({} and {})",
                op.text(),
                l.type_name(),
                r.type_name(),
                l.display(),
                r.display()
            ),
        )
    }

    pub fn member_of_extra(&mut self, x: &Value, coll: &Value, span: Span) -> R<bool> {
        err(
            "E504",
            span,
            format!(
                "can't test membership of {} in {} {}",
                x.display(),
                coll.type_name(),
                coll.display()
            ),
        )
    }

    pub fn power_extra(&mut self, v: Value, _n: i64, _upto: bool, span: Span) -> R<Value> {
        err(
            "E504",
            span,
            format!("`^` doesn't apply to {} {}", v.type_name(), v.display()),
        )
    }

    pub fn member_extra(&mut self, v: Value, name: &str, span: Span) -> R<Value> {
        err(
            "E505",
            span,
            format!("{} has no member `{name}`", v.type_name()),
        )
    }

    pub fn call_method(
        &mut self,
        recv: Value,
        name: &str,
        args: Args,
        named: Named,
        span: Span,
    ) -> R<Value> {
        if let Value::Record(r) = &recv
            && let Some(f) = r.get(name).cloned()
        {
            return self.call_value(&f, args, named, span);
        }
        err(
            "E505",
            span,
            format!("{} has no member `{name}`", recv.type_name()),
        )
    }

    pub fn call_builtin_extra(
        &mut self,
        _name: &str,
        _args: &Args,
        _named: &Named,
        _span: Span,
    ) -> R<Option<Value>> {
        Ok(None)
    }

    pub fn rev_extra(&mut self, v: Value, span: Span) -> R<Value> {
        err(
            "E504",
            span,
            format!("`rev` doesn't apply to {} {}", v.type_name(), v.display()),
        )
    }

    pub fn complement(&mut self, v: Value, span: Span) -> R<Value> {
        err(
            "E504",
            span,
            format!("`∁` doesn't apply to {} {}", v.type_name(), v.display()),
        )
    }

    pub fn star(&mut self, v: Value, span: Span, _plus: bool) -> R<Value> {
        match v {
            Value::Set(_) => err(
                "E205",
                span,
                "this set is infinite (Σ* has every word); it's allowed only inside `lang`",
            ),
            other => err(
                "E504",
                span,
                format!(
                    "`*` doesn't apply to {} {}",
                    other.type_name(),
                    other.display()
                ),
            ),
        }
    }

    pub fn values_equal(&mut self, l: &Value, r: &Value, _span: Span) -> R<bool> {
        Ok(l == r)
    }

    pub fn check(&mut self, s: &Stmt, subject: &Expr, kind: &CheckKind) -> R<()> {
        match kind {
            CheckKind::Holds(None) => {
                let v = self.eval(subject)?;
                let pass = self.truth(&v, subject.span)?;
                let line = self.line(s.span);
                let name = self.text(subject.span);
                self.emit(
                    s.span,
                    json!({ "kind": "test", "line": line, "name": name, "pass": pass }),
                )
            }
            _ => not_yet("`check` on machines and languages", s.span),
        }
    }

    pub fn run_stmt(&mut self, s: &Stmt, _m: &Expr, _w: &Expr) -> R<()> {
        not_yet("`run`", s.span)
    }

    pub fn lang_decl(&mut self, value: &Expr, _claim: Option<&(String, Span, Expr)>) -> R<Value> {
        not_yet("`lang`", value.span)
    }

    pub fn model_decl(&mut self, decl: &ModelDecl) -> R<()> {
        not_yet("`model`", decl.span)
    }

    pub fn machine_literal(&mut self, _m: &MachineLit, span: Span) -> R<Value> {
        not_yet("machines", span)
    }

    pub fn canonical_literal(&mut self, _c: &CanonicalLit, span: Span) -> R<Value> {
        not_yet("machines", span)
    }

    pub fn explore(&mut self, _x: &ExploreLit, span: Span) -> R<Value> {
        not_yet("`explore`", span)
    }

    pub fn regex_literal(
        &mut self,
        _text: &str,
        _text_span: Span,
        _over: Option<&Expr>,
        span: Span,
    ) -> R<Value> {
        not_yet("regexes", span)
    }

    pub fn cfg_literal(
        &mut self,
        _name: Option<&str>,
        _body: &str,
        _body_span: Span,
        span: Span,
    ) -> R<Value> {
        not_yet("grammars", span)
    }

    pub fn object_json(&self, v: &Value) -> serde_json::Value {
        json!({ "type": v.type_name(), "text": v.display() })
    }
}
