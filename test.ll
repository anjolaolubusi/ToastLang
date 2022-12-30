; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define double @anonexpr_() local_unnamed_addr #0 {
entry:
  ret float select (i1 fcmp one (double uitofp (float 1.000000e+00 to double), float 0.000000e+00), float 1.000000e+01, float 0.000000e+00)
}