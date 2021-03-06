use crate::consts::{constant_context, Constant};
use crate::utils::{in_macro, is_type_diagnostic_item, snippet, span_lint_and_sugg, walk_ptrs_ty};
use if_chain::if_chain;
use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint_pass, declare_tool_lint};

declare_clippy_lint! {
    /// **What it does:** Checks for usage of `.repeat(1)` and suggest the following method for each types.
    /// - `.to_string()` for `str`
    /// - `.clone()` for `String`
    /// - `.to_vec()` for `slice`
    ///
    /// **Why is this bad?** For example, `String.repeat(1)` is equivalent to `.clone()`. If cloning the string is the intention behind this, `clone()` should be used.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// fn main() {
    ///     let x = String::from("hello world").repeat(1);
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// fn main() {
    ///     let x = String::from("hello world").clone();
    /// }
    /// ```
    pub REPEAT_ONCE,
    complexity,
    "using `.repeat(1)` instead of `String.clone()`, `str.to_string()` or `slice.to_vec()` "
}

declare_lint_pass!(RepeatOnce => [REPEAT_ONCE]);

impl<'tcx> LateLintPass<'tcx> for RepeatOnce {
    fn check_expr(&mut self, cx: &LateContext<'_>, expr: &'tcx Expr<'_>) {
        if_chain! {
            if let ExprKind::MethodCall(ref path, _, ref args, _) = expr.kind;
            if path.ident.name == sym!(repeat);
            if let Some(Constant::Int(1)) = constant_context(cx, cx.tables()).expr(&args[1]);
            if !in_macro(args[0].span);
            then {
                let ty = walk_ptrs_ty(cx.tables().expr_ty(&args[0]));
                if ty.is_str() {
                    span_lint_and_sugg(
                        cx,
                        REPEAT_ONCE,
                        expr.span,
                        "calling `repeat(1)` on str",
                        "consider using `.to_string()` instead",
                        format!("{}.to_string()", snippet(cx, args[0].span, r#""...""#)),
                        Applicability::MachineApplicable,
                    );
                } else if ty.builtin_index().is_some() {
                    span_lint_and_sugg(
                        cx,
                        REPEAT_ONCE,
                        expr.span,
                        "calling `repeat(1)` on slice",
                        "consider using `.to_vec()` instead",
                        format!("{}.to_vec()", snippet(cx, args[0].span, r#""...""#)),
                        Applicability::MachineApplicable,
                    );
                } else if is_type_diagnostic_item(cx, ty, sym!(string_type)) {
                    span_lint_and_sugg(
                        cx,
                        REPEAT_ONCE,
                        expr.span,
                        "calling `repeat(1)` on a string literal",
                        "consider using `.clone()` instead",
                        format!("{}.clone()", snippet(cx, args[0].span, r#""...""#)),
                        Applicability::MachineApplicable,
                    );
                }
            }
        }
    }
}
