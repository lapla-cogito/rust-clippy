use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::is_in_const_context;
use clippy_utils::msrvs::{self, Msrv};
use clippy_utils::source::SpanRangeExt;
use clippy_utils::sugg::Sugg;
use rustc_errors::Applicability;
use rustc_hir::{Expr, QPath, TyKind};
use rustc_lint::LateContext;
use rustc_middle::ty::{self, FloatTy, IntTy, Ty, UintTy};
use rustc_span::hygiene;

use super::{CAST_LOSSLESS, utils};

pub(super) fn check(
    cx: &LateContext<'_>,
    expr: &Expr<'_>,
    cast_from_expr: &Expr<'_>,
    cast_from: Ty<'_>,
    cast_to: Ty<'_>,
    cast_to_hir: &rustc_hir::Ty<'_>,
    msrv: Msrv,
) {
    if !should_lint(cx, cast_from, cast_to, msrv)
        // Checking the expression, cast_from_expr and cast_to_hir share the same context.
        || [cast_from_expr.span, cast_to_hir.span]
            .iter()
            .any(|&span| !expr.span.eq_ctxt(span))
    {
        return;
    }

    span_lint_and_then(
        cx,
        CAST_LOSSLESS,
        expr.span,
        format!("casts from `{cast_from}` to `{cast_to}` can be expressed infallibly using `From`"),
        |diag| {
            diag.help("an `as` cast can become silently lossy if the types change in the future");
            let mut applicability = Applicability::MachineApplicable;
            let from_sugg = Sugg::hir_with_context(cx, cast_from_expr, expr.span.ctxt(), "<from>", &mut applicability);
            let Some(ty) = hygiene::walk_chain(cast_to_hir.span, expr.span.ctxt()).get_source_text(cx) else {
                return;
            };
            match cast_to_hir.kind {
                TyKind::Infer(()) => {
                    diag.span_suggestion_verbose(
                        expr.span,
                        "use `Into::into` instead",
                        format!("{}.into()", from_sugg.maybe_par()),
                        applicability,
                    );
                },
                // Don't suggest `A<_>::B::From(x)` or `macro!()::from(x)`
                kind if matches!(kind, TyKind::Path(QPath::Resolved(_, path))
                if path.segments.iter().any(|s| s.args.is_some()))
                    || !cast_to_hir.span.eq_ctxt(expr.span) =>
                {
                    diag.span_suggestion_verbose(
                        expr.span,
                        format!("use `<{ty}>::from` instead"),
                        format!("<{ty}>::from({from_sugg})"),
                        applicability,
                    );
                },
                _ => {
                    diag.span_suggestion_verbose(
                        expr.span,
                        format!("use `{ty}::from` instead"),
                        format!("{ty}::from({from_sugg})"),
                        applicability,
                    );
                },
            }
        },
    );
}

fn should_lint(cx: &LateContext<'_>, cast_from: Ty<'_>, cast_to: Ty<'_>, msrv: Msrv) -> bool {
    // Do not suggest using From in consts/statics until it is valid to do so (see #2267).
    if is_in_const_context(cx) {
        return false;
    }

    match (cast_from.kind(), cast_to.kind()) {
        (ty::Bool, ty::Uint(_) | ty::Int(_)) => msrv.meets(cx, msrvs::FROM_BOOL),
        (ty::Uint(_), ty::Uint(UintTy::Usize)) | (ty::Uint(UintTy::U8) | ty::Int(_), ty::Int(IntTy::Isize)) => {
            utils::int_ty_to_nbits(cast_from, cx.tcx) <= 16
        },
        // No `f16` to `f32`: https://github.com/rust-lang/rust/issues/123831
        (ty::Uint(UintTy::Usize) | ty::Int(IntTy::Isize), _)
        | (_, ty::Uint(UintTy::Usize) | ty::Int(IntTy::Isize))
        | (ty::Float(FloatTy::F16), ty::Float(FloatTy::F32)) => false,
        (ty::Uint(_) | ty::Int(_), ty::Int(_)) | (ty::Uint(_), ty::Uint(_)) => {
            utils::int_ty_to_nbits(cast_from, cx.tcx) < utils::int_ty_to_nbits(cast_to, cx.tcx)
        },
        (ty::Uint(_) | ty::Int(_), ty::Float(fl)) => utils::int_ty_to_nbits(cast_from, cx.tcx) < fl.bit_width(),
        (ty::Char, ty::Uint(_)) => utils::int_ty_to_nbits(cast_to, cx.tcx) >= 32,
        (ty::Float(fl_from), ty::Float(fl_to)) => fl_from.bit_width() < fl_to.bit_width(),
        _ => false,
    }
}
