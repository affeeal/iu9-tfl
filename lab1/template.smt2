(set-logic QF_NIA)
(define-fun min_inf () Int -1)
(define-fun arc_add ((x Int) (y Int)) Int
  (ite (< x y) y x))
(define-fun arc_mul ((x Int) (y Int)) Int
  (ite (= x min_inf) x (ite (= y min_inf) y (+ x y))))
(define-fun arc_gt ((x Int) (y Int)) Bool
  (or (> x y) (= x y min_inf)))
; BODY
(check-sat)
(exit)
