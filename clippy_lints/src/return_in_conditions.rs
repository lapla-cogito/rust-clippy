use clippy_utils::contains_return;
use clippy_utils::diagnostics::span_lint;
use rustc_hir::{BlockCheckMode, Expr, ExprKind, MatchSource};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::declare_lint_pass;

declare_clippy_lint! {
    /// ### What it does
    ///
    /// Checks for `return` in conditions.
    ///
    /// ### Why is this bad?
    ///
    /// It is confusing to have a `return` in a condition. It is better to have a separate `if` statement.
    ///
    /// ### Example
    /// ```no_run
    ///
    /// ```
    /// Use instead:
    /// ```no_run
    /// // example code which does not raise clippy warning
    /// ```
    #[clippy::version = "1.87.0"]
    pub RETURN_IN_CONDITIONS,
    style,
    "return in condition"
}

declare_lint_pass!(ReturnInConditions => [RETURN_IN_CONDITIONS]);

impl LateLintPass<'_> for ReturnInConditions {
    fn check_expr(&mut self, cx: &LateContext<'_>, expr: &'_ Expr<'_>) {
        if expr.span.from_expansion() {
            return;
        }

        match expr.kind {
            ExprKind::If(cond, ..) => {
                if !cond.span.eq_ctxt(expr.span) {
                    return;
                }

                if contains_return(cond) {
                    span_lint(cx, RETURN_IN_CONDITIONS, cond.span, "return in condition");
                }
            },
            ExprKind::Match(cond, _, MatchSource::Normal) => {
                if !cond.span.eq_ctxt(expr.span) {
                    return;
                }

                if contains_return(cond) {
                    span_lint(cx, RETURN_IN_CONDITIONS, cond.span, "return in condition");
                }
            },
            _ => return,
        }
    }
}
